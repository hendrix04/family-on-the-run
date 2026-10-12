use super::calendar_feeds::FeedEventInput;
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, TimeZone};
use ical::{parser::ical::component::IcalEvent, property::Property};
use rrule::{RRuleSet, Tz};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::BufReader,
};

fn property<'a>(event: &'a IcalEvent, name: &str) -> Option<&'a Property> {
    event.properties.iter().find(|p| p.name == name)
}
fn value<'a>(event: &'a IcalEvent, name: &str) -> Option<&'a str> {
    property(event, name)?.value.as_deref()
}
fn parameter<'a>(p: &'a Property, key: &str) -> Option<&'a str> {
    p.params
        .as_ref()?
        .iter()
        .find(|(name, _)| name == key)?
        .1
        .first()
        .map(String::as_str)
}
fn date(p: &Property, default_zone: Tz) -> Result<DateTime<Tz>, &'static str> {
    let raw = p.value.as_deref().ok_or("parse")?;
    let zone = if raw.ends_with('Z') {
        Tz::UTC
    } else {
        parameter(p, "TZID")
            .map(|name| {
                name.parse::<chrono_tz::Tz>()
                    .map(Tz::Tz)
                    .map_err(|_| "timezone")
            })
            .transpose()?
            .unwrap_or(default_zone)
    };
    let naive = if raw.len() == 8 {
        NaiveDate::parse_from_str(raw, "%Y%m%d")
            .map_err(|_| "parse")?
            .and_hms_opt(0, 0, 0)
            .ok_or("parse")?
    } else {
        NaiveDateTime::parse_from_str(raw.trim_end_matches('Z'), "%Y%m%dT%H%M%S")
            .map_err(|_| "parse")?
    };
    zone.from_local_datetime(&naive)
        .earliest()
        .ok_or("timezone")
}
fn line(p: &Property) -> String {
    let mut result = p.name.clone();
    for (name, values) in p.params.iter().flatten() {
        result.push_str(&format!(";{name}={}", values.join(",")));
    }
    format!("{result}:{}", p.value.as_deref().unwrap_or_default())
}
// The recurrence library uses timed DTSTART internally, including for DATE events.
// Convert Google's date-only UNTIL to an inclusive local-day endpoint for that representation.
fn recurrence_line(p: &Property, initial: DateTime<Tz>) -> Result<String, &'static str> {
    let mut normalized = p.clone();
    if let Some(raw) = &p.value {
        let mut parts = Vec::new();
        for part in raw.split(';') {
            if let Some(until) = part.strip_prefix("UNTIL=").filter(|v| v.len() == 8) {
                let next = NaiveDate::parse_from_str(until, "%Y%m%d")
                    .map_err(|_| "parse")?
                    .succ_opt()
                    .ok_or("parse")?
                    .and_hms_opt(0, 0, 0)
                    .ok_or("parse")?;
                let end = initial
                    .timezone()
                    .from_local_datetime(&next)
                    .earliest()
                    .ok_or("timezone")?
                    - Duration::seconds(1);
                parts.push(format!(
                    "UNTIL={}",
                    end.with_timezone(&chrono::Utc).format("%Y%m%dT%H%M%SZ")
                ));
            } else {
                parts.push(part.to_owned());
            }
        }
        normalized.value = Some(parts.join(";"));
    }
    Ok(line(&normalized))
}

fn text(raw: &str) -> String {
    // RFC 5545 TEXT escaping, after the library has unfolded content lines.
    let mut result = String::new();
    let mut chars = raw.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n' | 'N') => result.push('\n'),
                Some(next) => result.push(next),
                None => return result,
            }
        } else {
            result.push(ch);
        }
    }
    result.chars().take(200).collect()
}
fn instance(
    event: &IcalEvent,
    uid: &str,
    original: i64,
    start: DateTime<Tz>,
    zone: Tz,
) -> Result<FeedEventInput, &'static str> {
    let initial = date(property(event, "DTSTART").ok_or("parse")?, zone)?;
    let all_day = property(event, "DTSTART").is_some_and(|p| {
        parameter(p, "VALUE") == Some("DATE") || p.value.as_ref().is_some_and(|v| v.len() == 8)
    });
    let end = if let Some(p) = property(event, "DTEND") {
        date(p, zone)?
    } else if let Some(raw) = value(event, "DURATION") {
        initial + duration(raw)?
    } else {
        initial
            + if all_day {
                Duration::days(1)
            } else {
                Duration::milliseconds(1)
            }
    };
    let end = if all_day {
        let days = (end.date_naive() - initial.date_naive()).num_days();
        let next = start
            .date_naive()
            .checked_add_signed(Duration::days(days))
            .ok_or("parse")?
            .and_hms_opt(0, 0, 0)
            .ok_or("parse")?;
        start
            .timezone()
            .from_local_datetime(&next)
            .earliest()
            .ok_or("timezone")?
    } else {
        start + (end - initial)
    };
    if end <= start || start.timestamp_millis() < 0 {
        return Err("parse");
    }
    Ok(FeedEventInput {
        key: format!("{original}:{:x}", Sha256::digest(uid.as_bytes())),
        title: text(value(event, "SUMMARY").unwrap_or("Untitled event")),
        start_at: start.timestamp_millis(),
        end_at: end.timestamp_millis(),
        all_day,
        starts_on: if all_day {
            start.format("%Y-%m-%d").to_string()
        } else {
            String::new()
        },
        ends_on: if all_day {
            end.format("%Y-%m-%d").to_string()
        } else {
            String::new()
        },
        location: text(value(event, "LOCATION").unwrap_or_default()),
    })
}
fn duration(raw: &str) -> Result<Duration, &'static str> {
    let mut seconds = 0i64;
    let mut number = String::new();
    if !raw.starts_with('P') {
        return Err("parse");
    }
    for ch in raw[1..].chars() {
        if ch.is_ascii_digit() {
            number.push(ch);
            continue;
        }
        if ch == 'T' {
            continue;
        }
        let multiplier = match ch {
            'W' => 604800,
            'D' => 86400,
            'H' => 3600,
            'M' => 60,
            'S' => 1,
            _ => return Err("parse"),
        };
        let amount: i64 = number.parse().map_err(|_| "parse")?;
        seconds = seconds
            .checked_add(amount.checked_mul(multiplier).ok_or("parse")?)
            .ok_or("parse")?;
        number.clear();
    }
    if !number.is_empty() {
        return Err("parse");
    }
    Duration::try_seconds(seconds).ok_or("parse")
}

pub fn parse_calendar(ics: &str, now: i64) -> Result<Vec<FeedEventInput>, &'static str> {
    if ics.len() > 5 * 1024 * 1024 {
        return Err("large");
    }
    if !ics.lines().any(|l| l.trim() == "BEGIN:VCALENDAR")
        || !ics.lines().any(|l| l.trim() == "END:VCALENDAR")
    {
        return Err("parse");
    }
    let mut calendars = ical::IcalParser::new(BufReader::new(ics.as_bytes()));
    let calendar = calendars.next().ok_or("parse")?.map_err(|_| "parse")?;
    if calendars.next().is_some() {
        return Err("parse");
    }
    let zone: Tz = calendar
        .properties
        .iter()
        .find(|p| p.name == "X-WR-TIMEZONE")
        .and_then(|p| p.value.as_deref())
        .unwrap_or("UTC")
        .parse::<chrono_tz::Tz>()
        .map(Tz::Tz)
        .map_err(|_| "timezone")?;
    let from = now;
    let to = now + 365 * 86400000;
    let mut groups = BTreeMap::<String, Vec<&IcalEvent>>::new();
    for event in &calendar.events {
        groups
            .entry(value(event, "UID").ok_or("parse")?.to_owned())
            .or_default()
            .push(event);
    }
    let mut output = Vec::new();
    for (uid, events) in groups {
        let base = events
            .iter()
            .find(|e| property(e, "RECURRENCE-ID").is_none())
            .copied();
        let mut overrides = BTreeMap::new();
        for event in &events {
            if let Some(p) = property(event, "RECURRENCE-ID") {
                if parameter(p, "RANGE").is_some() {
                    return Err("unsupported");
                }
                overrides.insert(date(p, zone)?.timestamp_millis(), *event);
            }
        }
        let mut dates = BTreeSet::new();
        if let Some(base) = base {
            if value(base, "STATUS") == Some("CANCELLED") {
                continue;
            }
            let initial = date(property(base, "DTSTART").ok_or("parse")?, zone)?;
            dates.insert(initial.timestamp_millis());
            if base
                .properties
                .iter()
                .any(|p| matches!(p.name.as_str(), "RRULE" | "RDATE"))
            {
                let start_line = if initial.timezone() == Tz::UTC {
                    format!("DTSTART:{}", initial.format("%Y%m%dT%H%M%SZ"))
                } else {
                    format!(
                        "DTSTART;TZID={}:{}",
                        initial.timezone(),
                        initial.format("%Y%m%dT%H%M%S")
                    )
                };
                let mut rule = start_line;
                for p in &base.properties {
                    if matches!(p.name.as_str(), "RRULE" | "EXRULE") {
                        rule.push('\n');
                        rule.push_str(&recurrence_line(p, initial)?);
                    }
                }
                if base.properties.iter().any(|p| p.name == "RRULE") {
                    let set: RRuleSet = rule.parse().map_err(|_| "parse")?;
                    let earliest = from;
                    let result = set
                        .after(
                            Tz::UTC
                                .timestamp_millis_opt(earliest)
                                .single()
                                .ok_or("parse")?,
                        )
                        .before(Tz::UTC.timestamp_millis_opt(to).single().ok_or("parse")?)
                        .all(10001);
                    if result.limited || result.dates.len() > 10000 {
                        return Err("large");
                    }
                    dates.extend(result.dates.iter().map(DateTime::timestamp_millis));
                }
            }
            for p in &base.properties {
                if matches!(p.name.as_str(), "RDATE" | "EXDATE") {
                    for raw in p.value.as_deref().ok_or("parse")?.split(',') {
                        let mut single = p.clone();
                        single.value = Some(raw.to_owned());
                        let time = date(&single, zone)?.timestamp_millis();
                        if p.name == "RDATE" {
                            dates.insert(time);
                        } else {
                            dates.remove(&time);
                        }
                    }
                }
            }
            for original in dates {
                if overrides.contains_key(&original) {
                    continue;
                }
                let start = initial
                    .timezone()
                    .timestamp_millis_opt(original)
                    .single()
                    .ok_or("parse")?;
                if original < from || original >= to {
                    continue;
                }
                let event = instance(base, &uid, original, start, zone)?;
                if event.start_at >= from && event.start_at < to {
                    output.push(event);
                }
            }
        }
        // A moved occurrence can enter the window even when its original date was outside it.
        for (original, event) in overrides {
            if value(event, "STATUS") == Some("CANCELLED") {
                continue;
            }
            let start = date(property(event, "DTSTART").ok_or("parse")?, zone)?;
            if start.timestamp_millis() < from || start.timestamp_millis() >= to {
                continue;
            }
            let result = instance(event, &uid, original, start, zone)?;
            if result.start_at >= from && result.start_at < to {
                output.push(result);
            }
        }
        if output.len() > 10000 {
            return Err("large");
        }
    }
    Ok(output)
}

#[cfg(test)]
#[path = "calendar_parser_tests.rs"]
mod tests;
