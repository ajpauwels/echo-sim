use nexosim::{
    model::{Context, Model},
    ports::Output,
};
use serde::{Deserialize, Serialize};

use crate::{composition::Composition, message::Message};

#[derive(Serialize, Deserialize)]
pub struct Client {
    id: String,
    to: String,
    pub message_out: Output<Message>,
}

#[Model]
impl Client {
    pub fn new(id: String, to: String) -> Self {
        Client {
            id,
            to,
            message_out: Output::<Message>::default(),
        }
    }

    pub async fn receive(&mut self, message: Message) {
        println!(
            "{} received message from {}: {}",
            self.id, message.from, message.content
        );
    }

    #[nexosim(schedulable)]
    pub async fn send(&mut self, _: Composition, cx: &Context<Self>) {
        let message = Message {
            to: self.to.clone(),
            from: self.id.clone(),
            content: "hello, world!".to_owned(),
        };
        self.message_out.send(message).await;
        println!("Sending message at: {}", cx.time());
        // cx.schedule_event(Duration::from_secs(2), schedulable!(Self::send), ())
        //     .unwrap();
    }
}
