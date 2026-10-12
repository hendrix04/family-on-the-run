mod admin;
mod calendar_feeds;
mod calendar_parser;
mod calendar_sync;
mod schema;
mod static_site;

use schema::*;
use spacetimedb::{ReducerContext, Table, ViewContext};

#[spacetimedb::reducer(init)]
pub fn init(ctx: &ReducerContext) {
    ctx.db.household().insert(Household {
        id: 1,
        owner: ctx.sender(),
        access_code: String::new(),
    });
    for (position, (name, color)) in [
        ("Lindsay", "#bf755e"),
        ("Matt", "#6d8ba1"),
        ("Carter", "#b69a55"),
        ("Sydney", "#8a7ca4"),
    ]
    .into_iter()
    .enumerate()
    {
        ctx.db.person().insert(Person {
            id: position as u32 + 1,
            name: name.into(),
            color: color.into(),
            position: position as u32,
        });
    }
}

#[spacetimedb::reducer]
pub fn configure_access(ctx: &ReducerContext, access_code: String) -> Result<(), String> {
    let mut household = ctx.db.household().id().find(1).ok_or("Household missing")?;
    if ctx.sender() != household.owner {
        return Err("Only the database owner can configure household access".into());
    }
    if !(24..=128).contains(&access_code.len()) {
        return Err("Use an access code between 24 and 128 characters".into());
    }
    household.access_code = access_code;
    ctx.db.household().id().update(household);
    // Rotating the code also requires existing screens to pair again.
    for device in ctx.db.device().iter() {
        ctx.db.device().identity().delete(device.identity);
    }
    for device in ctx.db.admin_device().iter() {
        ctx.db.admin_device().identity().delete(device.identity);
    }
    Ok(())
}

#[spacetimedb::reducer]
pub fn join_household(ctx: &ReducerContext, access_code: String) -> Result<(), String> {
    let household = ctx.db.household().id().find(1).ok_or("Household missing")?;
    if household.access_code.is_empty() || !same_code(&household.access_code, &access_code) {
        return Err("That household access code isn't correct".into());
    }
    if ctx.db.device().identity().find(ctx.sender()).is_none() {
        ctx.db.device().insert(Device {
            identity: ctx.sender(),
        });
    }
    Ok(())
}

#[spacetimedb::reducer]
pub fn leave_household(ctx: &ReducerContext) {
    ctx.db.device().identity().delete(ctx.sender());
    ctx.db.admin_device().identity().delete(ctx.sender());
}

fn same_code(expected: &str, supplied: &str) -> bool {
    expected.len() == supplied.len()
        && expected
            .bytes()
            .zip(supplied.bytes())
            .fold(0u8, |diff, (a, b)| diff | (a ^ b))
            == 0
}

fn require_device(ctx: &ReducerContext) -> Result<(), String> {
    ctx.db
        .device()
        .identity()
        .find(ctx.sender())
        .map(|_| ())
        .ok_or("Connect this screen to your household first".into())
}

fn title(value: String) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 200 {
        return Err("Enter a title between 1 and 200 characters".into());
    }
    Ok(value.to_owned())
}

fn valid_color(color: &str) -> bool {
    color.len() == 7 && color.starts_with('#') && color[1..].bytes().all(|b| b.is_ascii_hexdigit())
}

#[spacetimedb::reducer]
pub fn add_task(ctx: &ReducerContext, person_id: u32, task_title: String) -> Result<(), String> {
    require_device(ctx)?;
    ctx.db
        .person()
        .id()
        .find(person_id)
        .ok_or("Person not found")?;
    ctx.db.task().insert(Task {
        id: 0,
        person_id,
        title: title(task_title)?,
        completed: false,
        created_at: ctx.timestamp.to_micros_since_unix_epoch(),
    });
    Ok(())
}

#[spacetimedb::reducer]
pub fn update_task(
    ctx: &ReducerContext,
    task_id: u64,
    task_title: String,
    completed: bool,
) -> Result<(), String> {
    require_device(ctx)?;
    let mut task = ctx.db.task().id().find(task_id).ok_or("Task not found")?;
    task.title = title(task_title)?;
    task.completed = completed;
    ctx.db.task().id().update(task);
    Ok(())
}

#[spacetimedb::reducer]
pub fn delete_task(ctx: &ReducerContext, task_id: u64) -> Result<(), String> {
    require_device(ctx)?;
    if !ctx.db.task().id().delete(task_id) {
        return Err("Task not found".into());
    }
    Ok(())
}

#[spacetimedb::reducer]
pub fn add_event(
    ctx: &ReducerContext,
    event_title: String,
    start_at: i64,
    end_at: i64,
    all_day: bool,
    color: String,
    location: String,
) -> Result<(), String> {
    require_device(ctx)?;
    if end_at <= start_at || start_at < 0 || end_at > 32_503_680_000_000 {
        return Err("Choose an end time after the start time".into());
    }
    if !valid_color(&color) {
        return Err("Choose a valid calendar color".into());
    }
    if location.chars().count() > 200 {
        return Err("Keep the location under 200 characters".into());
    }
    ctx.db.calendar_event().insert(CalendarEvent {
        id: 0,
        title: title(event_title)?,
        start_at,
        end_at,
        all_day,
        color,
        location: location.trim().into(),
    });
    Ok(())
}

#[spacetimedb::reducer]
pub fn delete_event(ctx: &ReducerContext, event_id: u64) -> Result<(), String> {
    require_device(ctx)?;
    if !ctx.db.calendar_event().id().delete(event_id) {
        return Err("Event not found".into());
    }
    Ok(())
}

#[spacetimedb::view(accessor = my_device, public)]
pub fn my_device(ctx: &ViewContext) -> Option<Device> {
    ctx.db.device().identity().find(ctx.sender())
}

#[spacetimedb::view(accessor = household_people, public)]
pub fn household_people(ctx: &ViewContext) -> Vec<Person> {
    if ctx.db.device().identity().find(ctx.sender()).is_none() {
        return vec![];
    }
    ctx.db.person().position().filter(0u32..).collect()
}

#[spacetimedb::view(accessor = household_tasks, public)]
pub fn household_tasks(ctx: &ViewContext) -> Vec<Task> {
    if ctx.db.device().identity().find(ctx.sender()).is_none() {
        return vec![];
    }
    ctx.db.task().created_at().filter(0i64..).collect()
}

#[spacetimedb::view(accessor = household_events, public)]
pub fn household_events(ctx: &ViewContext) -> Vec<CalendarEvent> {
    if ctx.db.device().identity().find(ctx.sender()).is_none() {
        return vec![];
    }
    ctx.db.calendar_event().start_at().filter(0i64..).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_blank_and_oversized_titles_and_incorrect_access_codes() {
        assert!(title("   ".into()).is_err());
        assert!(title("a".repeat(201)).is_err());
        assert_eq!(title("  A real task  ".into()).unwrap(), "A real task");
        assert!(!same_code("household-code", "household-coda"));
        assert!(!same_code("household-code", "household"));
        assert!(same_code("household-code", "household-code"));
    }
}
