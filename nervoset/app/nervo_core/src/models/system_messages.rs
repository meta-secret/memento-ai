use crate::utils::ai_utils::RESOURCES_DIR;
use nervo_sdk::agent_type::{AgentType, NervoAgentType};
use serde_derive::{Deserialize, Serialize};
use tokio::fs;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SystemMessages {
    pub start: String,
    pub manual: String,
    pub wait_second: String,
    pub empty_message: String,
    pub cant_get_message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum SystemMessage {
    Start(AgentType),
    Manual(AgentType),
    WaitSecond(AgentType),
    EmptyMessage(AgentType),
    CantGetYourMessage(AgentType),
}

impl SystemMessage {
    fn agent_type(&self) -> AgentType {
        match self {
            SystemMessage::Start(agent_type) => agent_type.clone(),
            SystemMessage::Manual(agent_type) => agent_type.clone(),
            SystemMessage::WaitSecond(agent_type) => agent_type.clone(),
            SystemMessage::EmptyMessage(agent_type) => agent_type.clone(),
            &SystemMessage::CantGetYourMessage(agent_type) => agent_type.clone(),
        }
    }

    pub async fn as_str(&self) -> anyhow::Result<String> {
        let agent = NervoAgentType::get_name(self.agent_type());
        let system_msg_file = format!("{}{}/system_messages.json", RESOURCES_DIR, agent);

        let json_string = fs::read_to_string(system_msg_file).await?;
        let system_messages_models: SystemMessages = serde_json::from_str(&json_string)?;

        match self {
            SystemMessage::Start(_) => Ok(system_messages_models.start.clone()),
            SystemMessage::Manual(_) => Ok(system_messages_models.manual.clone()),
            SystemMessage::WaitSecond(_) => Ok(system_messages_models.wait_second.clone()),
            SystemMessage::EmptyMessage(_) => Ok(system_messages_models.empty_message.clone()),
            &SystemMessage::CantGetYourMessage(_) => {
                Ok(system_messages_models.cant_get_message.clone())
            }
        }
    }
}
