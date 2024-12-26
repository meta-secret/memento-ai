use std::collections::{HashMap};
use std::sync::{Arc, RwLock};
use teloxide::prelude::{ChatId};
use crate::context::dialogue_state::{Dialogue, MaxSize};

#[derive(Default)]
pub struct UserContext {
    context: Arc<RwLock<HashMap<ChatId, Dialogue>>>,
}

impl UserContext {
    pub fn new() -> UserContext {
        UserContext {
            context: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn add_user_interaction_to_dialogue(
        &self,
        user_request: &str,
        chat_id: &ChatId,
        llm_response: &str,
        timestamp: String,
    ) {
        let mut dialogue = if let Some(dialogue) = self.get_dialogue(chat_id) {
            dialogue
        } else {
            Dialogue::new(MaxSize(20))
        };
        
        dialogue.add_user_interaction(
            user_request.to_string(),
            llm_response.to_string(),
            timestamp
        );

        self.set_dialogue_state(&chat_id, dialogue)
    }

    fn set_dialogue_state(&self, chat_id: &ChatId, dialogue_state: Dialogue) {
        let mut context = self.context
            .write()
            .expect("Couldn't capture thread for writing");
        context.insert(*chat_id, dialogue_state);
    }
    
    pub fn get_dialogue_string(&self, chat_id: &ChatId) -> String {
        let dialogue = self.get_dialogue(chat_id);
        let dialogue_string = match dialogue {
            None => {"".to_string()}
            Some(dialogue) => {
                dialogue.to_string()
            }
        };
        dialogue_string
    }
    
    fn get_dialogue(&self, chat_id: &ChatId) -> Option<Dialogue> {
        let read_guard = self.context.write().expect("Couldn't capture thread for reading");
        let dialogue = read_guard.get(chat_id).cloned();
        dialogue
    }
    
    pub fn last_llm_response(&self, chat_id: &ChatId) -> Option<String> {
        let dialogue = self.get_dialogue(chat_id);
        let response = dialogue?.last_llm_response();
        Some(response?)
    }
}
