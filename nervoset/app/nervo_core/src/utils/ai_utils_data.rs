pub mod system_role {
    use crate::utils::ai_utils::RESOURCES_DIR;
    use nervo_sdk::agent_type::{AgentType, NervoAgentType};

    pub enum RoleType {
        Clearing,
        UniquePointsFinal,
        AssistantMemory,
        ConclusionsPreprocessing,
        SearchKeywords,
    }

    impl RoleType {
        pub fn file_name(&self) -> FileName {
            let file_name = match self {
                RoleType::Clearing => FileName::CLEARING,
                RoleType::UniquePointsFinal => FileName::UNIQUE_POINTS_FINAL,
                RoleType::AssistantMemory => FileName::ASSISTANT_MEMORY,
                RoleType::ConclusionsPreprocessing => FileName::CONCLUSIONS_PREPROCESSING,
                RoleType::SearchKeywords => FileName::SEARCH_KEYWORDS,
            };

            let path = format!("system_roles/{}", file_name);
            FileName(path)
        }
    }

    pub struct FileName(String);
    impl FileName {
        const CRAP: &'static str = "crap.txt";
        const CLEARING: &'static str = "clearing.txt";
        const UNIQUE_POINTS_FINAL: &'static str = "unique_points_final.txt";
        const ASSISTANT_MEMORY: &'static str = "assistant_memory.txt";
        const CONCLUSIONS_PREPROCESSING: &'static str = "conclusions_pre_processing.txt";
        const SEARCH_KEYWORDS: &'static str = "search_keywords.txt";
    }

    pub struct RolePathBuilder {
        pub agent_type: AgentType,
        pub role_type: RoleType,
    }

    impl RolePathBuilder {
        fn resource_path(&self) -> String {
            let agent_type_name = NervoAgentType::get_name(self.agent_type);
            let base_path = format!("{}{}/", RESOURCES_DIR, agent_type_name);

            format!("{}{}", base_path, self.role_type.file_name().0)
        }

        pub fn resource_path_content(&self) -> anyhow::Result<String> {
            Ok(std::fs::read_to_string(self.resource_path())?)
        }
    }

    #[cfg(test)]
    mod test {
        use crate::utils::ai_utils_data::system_role::{RolePathBuilder, RoleType};
        use nervo_sdk::agent_type::{AgentType, NervoAgentType};

        #[test]
        fn resource_path_test() {
            let role_path = RolePathBuilder {
                agent_type: AgentType::Kevin,
                role_type: RoleType::Clearing,
            };

            let expected = {
                let agent_name = NervoAgentType::get_name(AgentType::Kevin);
                format!("../resources/agent/{agent_name}/system_roles/clearing.txt")
            };
            assert_eq!(expected, role_path.resource_path());
        }
    }
}

pub enum SortingType {
    Ascending,
    Descending,
    None,
}

pub enum TruncatingType {
    Truncated(u8),
    None,
}
