use crate::common::AppState;
use std::sync::Arc;
use teloxide::types::User;
use anyhow::Result;

pub const SUPER_ADMIN: &str = "SUPERADMIN";
pub const PROBIOT_OWNER: &str = "PROBOT_OWNER";
pub const PROBIOT_MEMBER: &str = "PROBIOT_MEMBER";

async fn get_roles(app_state: Arc<AppState>, maybe_user: Option<&User>) -> Result<Vec<String>> {
    let Some(User { id, .. }) = maybe_user else {
        return Ok(Vec::new());
    };

    app_state
        .local_db
        .get_user_permissions_tg_id(id.0)
        .await
}

pub async fn has_role(app_state: Arc<AppState>, user: Option<&User>, role: &str) -> Result<bool> {
    let user_roles = get_roles(app_state, user).await?;
    
    let role_str = String::from(role);
    let super_admin_str = String::from(SUPER_ADMIN);
    
    let has_role = user_roles.contains(&role_str) || user_roles.contains(&super_admin_str);
    
    Ok(has_role)
}
