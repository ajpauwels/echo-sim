use std::collections::BTreeMap;

use nexosim::{model::Model, ports::Output, simulation::Address};
use serde::{Deserialize, Serialize};

use crate::{client::Client, message::Message};

#[derive(Default, Serialize, Deserialize)]
pub struct Provider {
    push_outs: BTreeMap<String, Output<Message>>,
}

#[Model]
impl Provider {
    pub fn add_client(&mut self, id: &str, address: impl Into<Address<Client>>) {
        let mut output = Output::new();
        output.connect(Client::receive, address);
        self.push_outs.insert(id.to_owned(), output);
    }

    pub async fn receive(&mut self, message: Message) {
        if let Some(output) = self.push_outs.get_mut(&message.to) {
            output.send(message).await;
        }
    }
}
