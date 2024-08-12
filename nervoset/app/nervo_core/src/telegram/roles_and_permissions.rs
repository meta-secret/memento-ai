use anyhow::Result;
use teloxide::types::User;
use crate::db::local_db::LocalDb;

pub const SUPER_ADMIN: &str = "SUPERADMIN";
pub const PROBIOT_OWNER: &str = "PROBOT_OWNER";
pub const PROBIOT_MEMBER: &str = "PROBIOT_MEMBER";

async fn get_roles(local_db: &LocalDb, maybe_user: Option<&User>) -> Result<Vec<String>> {
    let Some(User { id, .. }) = maybe_user else {
        return Ok(Vec::new());
    };

    local_db.get_user_permissions_tg_id(id.0).await
}

pub async fn has_role(local_db: &LocalDb, user: Option<&User>, role: &str) -> Result<bool> {
    let user_roles = get_roles(local_db, user).await?;

    let role_str = String::from(role);
    let super_admin_str = String::from(SUPER_ADMIN);

    let has_role = user_roles.contains(&role_str) || user_roles.contains(&super_admin_str);

    Ok(has_role)
}
