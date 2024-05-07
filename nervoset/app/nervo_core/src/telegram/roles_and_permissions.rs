use crate::common::AppState;
use std::sync::Arc;
use teloxide::types::User;

pub const SUPERADMIN: &str = "SUPERADMIN";
pub const PROBIOT_OWNER: &str = "PROBOT_OWNER";
pub const PROBIOT_MEMBER: &str = "PROBIOT_MEMBER";

pub async fn get_roles(app_state: Arc<AppState>, user: Option<&User>) -> Vec<String> {
    match user {
        Some(u) => app_state
            .local_db
            .get_user_permissions_tg_id(u.clone().id.0)
            .await
            .unwrap(),
        None => Vec::new(),
    }
}

pub async fn has_role(app_state: Arc<AppState>, user: Option<&User>, role: &String) -> bool {
    let user_roles = get_roles(app_state, user).await;
    user_roles.contains(&role) || user_roles.contains(&SUPERADMIN.to_string())
}
