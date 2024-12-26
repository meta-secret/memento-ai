use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct QdrantKeywords {
    keywords: Vec<String>,
}

impl QdrantKeywords {
    pub fn new(keywords: Vec<String>) -> Self {
        Self { keywords }
    }

    fn update_keywords(&mut self, keywords: Vec<String>) {
        self.keywords = keywords;
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ConclusionsKeywords {
    conclusions: Vec<String>,
}

impl ConclusionsKeywords {
    fn new(conclusions: Vec<String>) -> Self {
        Self { conclusions }
    }

    fn update_conclusions(&mut self, conclusions: Vec<String>) {
        self.conclusions = conclusions;
    }
}
