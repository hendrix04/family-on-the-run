use super::{require_device, same_code, schema::*, title, valid_color};
use spacetimedb::{ReducerContext, Table, ViewContext};
use url::Url;

pub(crate) fn require_admin(ctx: &ReducerContext) -> Result<(), String> {
    require_device(ctx)?;
    ctx.db
        .admin_device()
        .identity()
        .find(ctx.sender())
        .map(|_| ())
        .ok_or("Unlock Admin on this screen first".into())
}

#[spacetimedb::reducer]
pub fn configure_admin_access(ctx: &ReducerContext, access_code: String) -> Result<(), String> {
    let household = ctx.db.household().id().find(1).ok_or("Household missing")?;
    if ctx.sender() != household.owner {
        return Err("Only the database owner can configure admin access".into());
    }
    if !(24..=128).contains(&access_code.len()) {
        return Err("Use an admin code between 24 and 128 characters".into());
    }
    let access = AdminAccess { id: 1, access_code };
    if ctx.db.admin_access().id().find(1).is_some() {
        ctx.db.admin_access().id().update(access);
    } else {
        ctx.db.admin_access().insert(access);
    }
    for device in ctx.db.admin_device().iter() {
        ctx.db.admin_device().identity().delete(device.identity);
    }
    Ok(())
}

#[spacetimedb::reducer]
pub fn unlock_admin(ctx: &ReducerContext, access_code: String) -> Result<(), String> {
    require_device(ctx)?;
    let access = ctx
        .db
        .admin_access()
        .id()
        .find(1)
        .ok_or("Admin access has not been configured")?;
    if !same_code(&access.access_code, &access_code) {
        return Err("That admin access code isn't correct".into());
    }
    if ctx
        .db
        .admin_device()
        .identity()
        .find(ctx.sender())
        .is_none()
    {
        ctx.db.admin_device().insert(AdminDevice {
            identity: ctx.sender(),
        });
    }
    Ok(())
}

#[spacetimedb::reducer]
pub fn lock_admin(ctx: &ReducerContext) {
    ctx.db.admin_device().identity().delete(ctx.sender());
}

fn validate_oauth(client_id: &str, redirect_uri: &str) -> Result<(), String> {
    if client_id.len() > 300
        || !client_id.ends_with(".apps.googleusercontent.com")
        || client_id.bytes().any(|b| b.is_ascii_whitespace())
    {
        return Err("Enter a Google OAuth client ID ending in .apps.googleusercontent.com".into());
    }
    let uri = Url::parse(redirect_uri).map_err(|_| "Enter a valid redirect URL")?;
    let local_http = uri.scheme() == "http"
        && matches!(uri.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if redirect_uri.len() > 2000
        || (!local_http && uri.scheme() != "https")
        || uri.host_str().is_none()
        || uri.fragment().is_some()
        || !uri.username().is_empty()
        || uri.password().is_some()
    {
        return Err(
            "Use an HTTPS redirect URL, or HTTP on localhost, without credentials or a fragment"
                .into(),
        );
    }
    Ok(())
}

#[spacetimedb::reducer]
pub fn save_google_config(
    ctx: &ReducerContext,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    clear_secret: bool,
) -> Result<(), String> {
    require_admin(ctx)?;
    let client_id = client_id.trim().to_owned();
    let redirect_uri = redirect_uri.trim().to_owned();
    validate_oauth(&client_id, &redirect_uri)?;
    if client_secret.len() > 2000 {
        return Err("Client secret is too long".into());
    }
    let existing = ctx.db.google_config().id().find(1);
    let secret = if clear_secret {
        String::new()
    } else if !client_secret.trim().is_empty() {
        client_secret.trim().to_owned()
    } else {
        existing
            .as_ref()
            .filter(|config| config.client_id == client_id)
            .map(|config| config.client_secret.clone())
            .unwrap_or_default()
    };
    let config = GoogleConfig {
        id: 1,
        client_id,
        client_secret: secret,
        redirect_uri,
    };
    if existing.is_some() {
        ctx.db.google_config().id().update(config);
    } else {
        ctx.db.google_config().insert(config);
    }
    Ok(())
}

#[spacetimedb::reducer]
pub fn clear_google_config(ctx: &ReducerContext) -> Result<(), String> {
    require_admin(ctx)?;
    ctx.db.google_config().id().delete(1);
    Ok(())
}

#[spacetimedb::reducer]
pub fn add_google_calendar(
    ctx: &ReducerContext,
    calendar_id: String,
    name: String,
    color: String,
) -> Result<(), String> {
    require_admin(ctx)?;
    let calendar_id = calendar_id.trim().to_owned();
    if calendar_id.is_empty()
        || calendar_id.len() > 300
        || calendar_id.bytes().any(|b| b.is_ascii_whitespace())
    {
        return Err(
            "Enter a calendar ID without spaces, such as an email address or primary".into(),
        );
    }
    if !valid_color(&color) {
        return Err("Choose a valid calendar color".into());
    }
    if ctx
        .db
        .google_calendar_source()
        .calendar_id()
        .find(&calendar_id)
        .is_some()
    {
        return Err("That calendar is already configured".into());
    }
    ctx.db
        .google_calendar_source()
        .insert(GoogleCalendarSource {
            id: 0,
            calendar_id,
            name: title(name)?,
            color,
        });
    Ok(())
}

#[spacetimedb::reducer]
pub fn remove_google_calendar(ctx: &ReducerContext, source_id: u64) -> Result<(), String> {
    require_admin(ctx)?;
    if !ctx.db.google_calendar_source().id().delete(source_id) {
        return Err("Calendar source not found".into());
    }
    Ok(())
}

#[spacetimedb::reducer]
pub fn update_person(
    ctx: &ReducerContext,
    person_id: u32,
    name: String,
    color: String,
) -> Result<(), String> {
    require_admin(ctx)?;
    let mut person = ctx
        .db
        .person()
        .id()
        .find(person_id)
        .ok_or("Person not found")?;
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 50 {
        return Err("Use a name between 1 and 50 characters".into());
    }
    if !valid_color(&color) {
        return Err("Choose a valid member color".into());
    }
    person.name = name.to_owned();
    person.color = color;
    ctx.db.person().id().update(person);
    Ok(())
}

#[spacetimedb::view(accessor = my_admin, public)]
pub fn my_admin(ctx: &ViewContext) -> Option<AdminDevice> {
    ctx.db.device().identity().find(ctx.sender())?;
    ctx.db.admin_device().identity().find(ctx.sender())
}

#[spacetimedb::view(accessor = admin_google_config, public)]
pub fn admin_google_config(ctx: &ViewContext) -> Option<GoogleConfigSummary> {
    my_admin(ctx)?;
    let config = ctx.db.google_config().id().find(1)?;
    Some(GoogleConfigSummary {
        id: config.id,
        client_id: config.client_id,
        redirect_uri: config.redirect_uri,
        has_client_secret: !config.client_secret.is_empty(),
    })
}

#[spacetimedb::view(accessor = admin_google_calendars, public)]
pub fn admin_google_calendars(ctx: &ViewContext) -> Vec<GoogleCalendarSource> {
    if my_admin(ctx).is_none() {
        return vec![];
    }
    ctx.db
        .google_calendar_source()
        .name()
        .filter(""..)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_google_oauth_redirects() {
        let client = "123.apps.googleusercontent.com";
        assert!(validate_oauth(client, "https://family.example/oauth/google/callback").is_ok());
        assert!(validate_oauth(client, "http://localhost:5173/oauth/google/callback").is_ok());
        for uri in [
            "http://family.example/callback",
            "javascript:alert(1)",
            "https://family.example/callback#secret",
            "https://user:password@family.example/callback",
        ] {
            assert!(validate_oauth(client, uri).is_err());
        }
        assert!(validate_oauth("invalid-client", "https://family.example/callback").is_err());
    }
}
