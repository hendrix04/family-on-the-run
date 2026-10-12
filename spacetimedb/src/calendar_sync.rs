use super::{calendar_feeds::*, calendar_parser::parse_calendar, schema::*};
use spacetimedb::http::{Request, Timeout};
use spacetimedb::{ProcedureContext, ReducerContext, ScheduleAt, Table, TxContext};
use std::time::Duration;

#[spacetimedb::table(accessor = calendar_sync_job, scheduled(import_calendar_feed))]
pub struct CalendarSyncJob {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub scheduled_at: ScheduleAt,
    #[index(btree)]
    pub feed_id: u64,
}

pub fn schedule_feed(ctx: &ReducerContext, feed_id: u64, periodic: bool) {
    if periodic
        && !ctx
            .db
            .calendar_sync_job()
            .feed_id()
            .filter(feed_id)
            .any(|job| matches!(job.scheduled_at, ScheduleAt::Interval(_)))
    {
        ctx.db.calendar_sync_job().insert(CalendarSyncJob {
            id: 0,
            feed_id,
            scheduled_at: ScheduleAt::Interval(Duration::from_secs(300).into()),
        });
    }
    // Coalesce pending immediate refreshes; periodic schedules remain intact.
    for job in ctx.db.calendar_sync_job().feed_id().filter(feed_id) {
        if matches!(job.scheduled_at, ScheduleAt::Time(_)) {
            ctx.db.calendar_sync_job().id().delete(job.id);
        }
    }
    ctx.db.calendar_sync_job().insert(CalendarSyncJob {
        id: 0,
        feed_id,
        scheduled_at: ctx.timestamp.into(),
    });
}

#[spacetimedb::reducer]
pub fn initialize_calendar_schedules(ctx: &ReducerContext) -> Result<(), String> {
    if ctx
        .db
        .household()
        .id()
        .find(1)
        .ok_or("Household missing")?
        .owner
        != ctx.sender()
    {
        return Err("Only the database owner can initialize calendar schedules".into());
    }
    for feed in ctx.db.calendar_feed().iter() {
        schedule_feed(ctx, feed.id, true);
    }
    Ok(())
}

fn commit(
    ctx: &TxContext,
    feed_id: u64,
    revision: u64,
    now: i64,
    result: &Result<Vec<FeedEventInput>, &'static str>,
) {
    let Some(mut feed) = ctx.db.calendar_feed().id().find(feed_id) else {
        return;
    };
    if feed.revision != revision {
        return;
    }
    // Expire past imports even if the download fails; retain upcoming cached events.
    for event in ctx.db.imported_event().feed_id().filter(feed_id) {
        if event.start_at < now {
            ctx.db.imported_event().id().delete(event.id);
        }
    }
    feed.event_count = ctx.db.imported_event().feed_id().filter(feed_id).count() as u32;
    feed.last_attempt = now;
    match result {
        Err(code) => feed.error = match *code {
            "download" => "Could not download the feed. Check the link or try again later.",
            "large" => "This feed exceeds the import size limit.",
            "timezone" => "This feed contains an unsupported timezone or invalid local time.",
            "unsupported" => "This feed uses an unsupported recurrence feature.",
            _ => "Could not read this calendar feed. Check that the link returns a valid ICS calendar.",
        }.into(),
        Ok(events) => {
            ctx.db.imported_event().feed_id().delete(feed_id);
            for event in events {
                ctx.db.imported_event().insert(ImportedEvent { id: format!("feed:{feed_id}:{}", event.key), feed_id,
                    title: event.title.clone(), start_at: event.start_at, end_at: event.end_at,
                    all_day: event.all_day, starts_on: event.starts_on.clone(), ends_on: event.ends_on.clone(),
                    location: event.location.clone(), color: feed.color.clone(), source_name: feed.name.clone() });
            }
            feed.event_count = events.len() as u32;
            feed.last_success = now;
            feed.error.clear();
        }
    }
    ctx.db.calendar_feed().id().update(feed);
}

#[spacetimedb::procedure]
pub fn import_calendar_feed(ctx: &mut ProcedureContext, job: CalendarSyncJob) {
    if ctx.sender() != ctx.database_identity() {
        return;
    }
    let feed = ctx.with_tx(|tx| {
        let mut feed = tx.db.calendar_feed().id().find(job.feed_id)?;
        // A newer fetch supersedes any older in-flight fetch.
        feed.revision += 1;
        Some(tx.db.calendar_feed().id().update(feed))
    });
    let Some(feed) = feed else {
        return;
    };
    let now = ctx.timestamp.to_micros_since_unix_epoch() / 1000;
    let result = fetch(ctx, &feed.url).and_then(|ics| parse_calendar(&ics, now));
    ctx.with_tx(|tx| commit(tx, feed.id, feed.revision, now, &result));
}

fn fetch(ctx: &ProcedureContext, url: &str) -> Result<String, &'static str> {
    if !valid_feed_url(url) {
        return Err("download");
    }
    let request = Request::builder()
        .uri(url)
        .header("Accept", "text/calendar")
        .extension(Timeout(Duration::from_secs(20).into()))
        .body(())
        .map_err(|_| "download")?;
    let response = ctx.http.send(request).map_err(|_| "download")?;
    if !response.status().is_success() {
        return Err("download");
    }
    let bytes = response.into_body().into_bytes();
    if bytes.len() > 5 * 1024 * 1024 {
        return Err("large");
    }
    String::from_utf8(bytes.to_vec()).map_err(|_| "parse")
}
