use std::collections::VecDeque;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone)]
pub struct MaxSize(pub usize);

#[derive(Clone)]
pub struct Dialogue {
    messages: VecDeque<UserInteraction>,
    max_size: MaxSize,
}

impl Dialogue {
    pub fn new(max_size: MaxSize) -> Dialogue {
        Dialogue {
            messages: VecDeque::new(),
            max_size,
        }
    }

    pub fn to_string(&self) -> String {
        self.messages
            .iter()
            .map(|interaction| {
                let timestamp = &interaction.timestamp;
                let user_request = &interaction.user_request;
                let llm_response = &interaction.llm_response;

                format!(
                    "[{}] User: {}\nYou (Leo): {}",
                    timestamp, user_request, llm_response
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }
    
    pub fn add_user_interaction(
        &mut self,
        user_request: String,
        llm_response: String,
        timestamp: String,
    ) { 
        let entry = UserInteraction {
            timestamp,
            user_request,
            llm_response,
        };
        
        self.messages.push_back(entry);

        if self.messages.len() > self.max_size.0 {
            self.messages.pop_front();
        }
    }
    
    pub fn last_llm_response(&self) -> Option<String> {
        Some(self.messages.back()?.llm_response.clone())
    }
}


#[derive(Debug, Deserialize, Serialize, Clone)]
struct UserInteraction {
    timestamp: String,
    user_request: String,
    llm_response: String,
}