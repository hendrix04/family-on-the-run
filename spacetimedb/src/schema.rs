use spacetimedb::Identity;

#[spacetimedb::table(accessor = household)]
pub struct Household {
    #[primary_key]
    pub id: u8,
    pub owner: Identity,
    pub access_code: String,
}

#[spacetimedb::table(accessor = device)]
pub struct Device {
    #[primary_key]
    pub identity: Identity,
}

#[spacetimedb::table(accessor = person)]
pub struct Person {
    #[primary_key]
    pub id: u32,
    pub name: String,
    pub color: String,
    #[index(btree)]
    pub position: u32,
}

#[spacetimedb::table(accessor = task)]
pub struct Task {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub person_id: u32,
    pub title: String,
    pub completed: bool,
    #[index(btree)]
    pub created_at: i64,
}

#[spacetimedb::table(accessor = calendar_event)]
pub struct CalendarEvent {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub title: String,
    #[index(btree)]
    pub start_at: i64,
    pub end_at: i64,
    pub all_day: bool,
    pub color: String,
    pub location: String,
}

#[spacetimedb::table(accessor = admin_access)]
pub struct AdminAccess {
    #[primary_key]
    pub id: u8,
    pub access_code: String,
}

#[spacetimedb::table(accessor = admin_device)]
pub struct AdminDevice {
    #[primary_key]
    pub identity: Identity,
}

#[spacetimedb::table(accessor = google_config)]
pub struct GoogleConfig {
    #[primary_key]
    pub id: u8,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[spacetimedb::table(accessor = google_calendar_source)]
pub struct GoogleCalendarSource {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[unique]
    pub calendar_id: String,
    #[index(btree)]
    pub name: String,
    pub color: String,
}

#[derive(spacetimedb::SpacetimeType)]
pub struct GoogleConfigSummary {
    pub id: u8,
    pub client_id: String,
    pub redirect_uri: String,
    pub has_client_secret: bool,
}
