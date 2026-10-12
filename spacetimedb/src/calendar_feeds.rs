use super::calendar_sync::calendar_sync_job;
use super::{
    admin::{my_admin, require_admin},
    schema::*,
    title, valid_color,
};
use spacetimedb::{ReducerContext, Table, ViewContext};
use url::Url;

#[spacetimedb::table(accessor = calendar_feed)]
pub struct CalendarFeed {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub name: String,
    pub color: String,
    pub url: String,
    pub revision: u64,
    pub last_attempt: i64,
    pub last_success: i64,
    pub error: String,
    pub event_count: u32,
}

#[derive(spacetimedb::SpacetimeType)]
pub struct FeedSummary {
    pub id: u64,
    pub name: String,
    pub color: String,
    pub last_attempt: i64,
    pub last_success: i64,
    pub error: String,
    pub event_count: u32,
}

#[derive(spacetimedb::SpacetimeType)]
pub struct FeedEventInput {
    pub key: String,
    pub title: String,
    pub start_at: i64,
    pub end_at: i64,
    pub all_day: bool,
    pub starts_on: String,
    pub ends_on: String,
    pub location: String,
}

#[spacetimedb::table(accessor = imported_event)]
pub struct ImportedEvent {
    #[primary_key]
    pub id: String,
    #[index(btree)]
    pub feed_id: u64,
    pub title: String,
    pub start_at: i64,
    pub end_at: i64,
    pub all_day: bool,
    pub starts_on: String,
    pub ends_on: String,
    pub location: String,
    pub color: String,
    pub source_name: String,
}

pub(crate) fn valid_feed_url(value: &str) -> bool {
    Url::parse(value).is_ok_and(|url| {
        value.len() <= 2000
            && url.scheme() == "https"
            && url.host_str() == Some("calendar.google.com")
            && url.port().is_none()
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none()
            && url.path().starts_with("/calendar/ical/")
            && url.path().ends_with("/basic.ics")
    })
}

#[spacetimedb::reducer]
pub fn add_calendar_feed(
    ctx: &ReducerContext,
    name: String,
    color: String,
    url: String,
) -> Result<(), String> {
    require_admin(ctx)?;
    let url = url.trim().to_owned();
    if !valid_feed_url(&url) {
        return Err(
            "Paste Google's Secret address in iCal format (a calendar.google.com basic.ics URL)"
                .into(),
        );
    }
    if !valid_color(&color) {
        return Err("Choose a valid calendar color".into());
    }
    if ctx.db.calendar_feed().iter().any(|f| f.url == url) {
        return Err("That feed is already configured".into());
    }
    let feed = ctx.db.calendar_feed().insert(CalendarFeed {
        id: 0,
        name: title(name)?,
        color,
        url,
        revision: 1,
        last_attempt: 0,
        last_success: 0,
        error: String::new(),
        event_count: 0,
    });
    super::calendar_sync::schedule_feed(ctx, feed.id, true);
    Ok(())
}

#[spacetimedb::reducer]
pub fn remove_calendar_feed(ctx: &ReducerContext, feed_id: u64) -> Result<(), String> {
    require_admin(ctx)?;
    if !ctx.db.calendar_feed().id().delete(feed_id) {
        return Err("Feed not found".into());
    }
    ctx.db.imported_event().feed_id().delete(feed_id);
    ctx.db.calendar_sync_job().feed_id().delete(feed_id);
    Ok(())
}

#[spacetimedb::reducer]
pub fn refresh_calendar_feed(ctx: &ReducerContext, feed_id: u64) -> Result<(), String> {
    require_admin(ctx)?;
    let mut feed = ctx
        .db
        .calendar_feed()
        .id()
        .find(feed_id)
        .ok_or("Feed not found")?;
    feed.revision += 1;
    ctx.db.calendar_feed().id().update(feed);
    super::calendar_sync::schedule_feed(ctx, feed_id, false);
    Ok(())
}

#[spacetimedb::view(accessor = admin_calendar_feeds, public)]
pub fn admin_calendar_feeds(ctx: &ViewContext) -> Vec<FeedSummary> {
    if my_admin(ctx).is_none() {
        return vec![];
    }
    ctx.db
        .calendar_feed()
        .name()
        .filter(""..)
        .map(|f| FeedSummary {
            id: f.id,
            name: f.name,
            color: f.color,
            last_attempt: f.last_attempt,
            last_success: f.last_success,
            error: f.error,
            event_count: f.event_count,
        })
        .collect()
}

#[spacetimedb::view(accessor = household_imported_events, public)]
pub fn household_imported_events(ctx: &ViewContext) -> Vec<ImportedEvent> {
    if ctx.db.device().identity().find(ctx.sender()).is_none() {
        return vec![];
    }
    ctx.db.imported_event().feed_id().filter(0u64..).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restricts_feed_destinations() {
        assert!(valid_feed_url(
            "https://calendar.google.com/calendar/ical/example%40gmail.com/private-0123456789abcdef0123456789abcdef/basic.ics"
        ));
        for url in [
            "http://calendar.google.com/calendar/ical/x/basic.ics",
            "https://localhost/calendar/ical/x/basic.ics",
            "https://calendar.google.com.evil.example/calendar/ical/x/basic.ics",
            "https://calendar.google.com/calendar/ical/x/basic.ics#secret",
        ] {
            assert!(!valid_feed_url(url));
        }
    }
}
