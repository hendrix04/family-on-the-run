use super::*;
#[test]
fn handles_recurrence_exceptions_dst_and_all_day_dates() {
    let result = parse_calendar(
        include_str!("../../scripts/fixtures/calendar.ics"),
        1791720000000,
    )
    .unwrap();
    assert_eq!(result.len(), 4);
    let moved = result.iter().find(|e| e.title == "Moved class").unwrap();
    assert_eq!(moved.start_at, 1794153600000);
    let holiday = result.iter().find(|e| e.all_day).unwrap();
    assert_eq!(holiday.starts_on, "2026-10-31");
    assert_eq!(holiday.ends_on, "2026-11-03");
    assert!(parse_calendar("<html>bad</html>", 1791720000000).is_err());
}
#[test]
fn expands_all_day_recurrences_across_dst_and_cancelled_overrides() {
    let ics = "BEGIN:VCALENDAR\nVERSION:2.0\nX-WR-TIMEZONE:America/New_York\nBEGIN:VEVENT\nUID:days\nDTSTART;VALUE=DATE:20261025\nDTEND;VALUE=DATE:20261026\nRRULE:FREQ=WEEKLY;COUNT=4\nRDATE;VALUE=DATE:20261120\nEXDATE;VALUE=DATE:20261108\nSUMMARY:Day\nEND:VEVENT\nBEGIN:VEVENT\nUID:days\nRECURRENCE-ID;VALUE=DATE:20261115\nSTATUS:CANCELLED\nEND:VEVENT\nEND:VCALENDAR";
    let events = parse_calendar(ics, 1791720000000).unwrap();
    assert_eq!(
        events
            .iter()
            .map(|e| e.starts_on.as_str())
            .collect::<Vec<_>>(),
        vec!["2026-10-25", "2026-11-01", "2026-11-20"]
    );
    assert!(events.iter().all(|e| e.all_day));
    assert_eq!(events[1].end_at - events[1].start_at, 25 * 3600000);
}

#[test]
fn google_date_only_until_is_inclusive_in_calendar_timezone() {
    let ics = "BEGIN:VCALENDAR\nVERSION:2.0\nX-WR-TIMEZONE:America/New_York\nBEGIN:VEVENT\nUID:all-day\nDTSTART;VALUE=DATE:20261025\nDTEND;VALUE=DATE:20261026\nRRULE:FREQ=WEEKLY;WKST=SU;UNTIL=20261108;BYDAY=SU\nSUMMARY:Day\nEND:VEVENT\nEND:VCALENDAR";
    let events = parse_calendar(ics, 1791720000000).unwrap();
    assert_eq!(
        events
            .iter()
            .map(|e| e.starts_on.as_str())
            .collect::<Vec<_>>(),
        vec!["2026-10-25", "2026-11-01", "2026-11-08"]
    );
}

#[test]
fn only_future_starts_are_imported_including_old_recurring_series() {
    let now = Tz::UTC
        .with_ymd_and_hms(2026, 10, 11, 12, 0, 0)
        .unwrap()
        .timestamp_millis();
    let mut ics = "BEGIN:VCALENDAR\nVERSION:2.0\n".to_owned();
    for (uid, start, end, extra) in [
        ("past", "20261010T120000Z", "20261010T130000Z", ""),
        ("ongoing", "20261011T110000Z", "20261011T130000Z", ""),
        ("today", "20261011", "20261012", ""),
        ("now", "20261011T120000Z", "20261011T130000Z", ""),
        ("future", "20261012T120000Z", "20261012T130000Z", ""),
        ("outside", "20271012T120000Z", "20271012T130000Z", ""),
        (
            "old-series",
            "20251011T120000Z",
            "20251011T130000Z",
            "RRULE:FREQ=YEARLY;COUNT=3\n",
        ),
    ] {
        ics.push_str(&format!("BEGIN:VEVENT\nUID:{uid}\nDTSTART:{start}\nDTEND:{end}\nSUMMARY:{uid}\n{extra}END:VEVENT\n"));
    }
    ics.push_str("END:VCALENDAR\n");
    let events = parse_calendar(&ics, now).unwrap();
    assert!(events.iter().all(|e| e.start_at >= now));
    let mut titles = events.iter().map(|e| e.title.as_str()).collect::<Vec<_>>();
    titles.sort_unstable();
    assert_eq!(titles, vec!["future", "now", "old-series"]);
}
