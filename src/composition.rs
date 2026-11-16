use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct Composition {
    chat_id: u64,
    recipients: Vec<u64>,
}

impl Composition {
    pub fn new(chat_id: u64, recipients: Vec<u64>) -> Self {
        Composition {
            chat_id,
            recipients,
        }
    }
}
