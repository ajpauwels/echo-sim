use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub to: String,
    pub from: String,
    pub content: String,
}
