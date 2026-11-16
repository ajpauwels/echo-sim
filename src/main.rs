use std::time::Duration;

use chat::Chat;
use client::Client;
use nexosim::{
    simulation::{Mailbox, SimInit, SimulationError},
    time::MonotonicTime,
};
use provider::Provider;
use rand::{SeedableRng, rngs::Xoshiro256PlusPlus};

mod affine;
mod chat;
mod client;
mod composition;
mod config;
mod distribution;
mod message;
mod mixture;
mod provider;
mod truncated;

fn main() -> Result<(), SimulationError> {
    // Get app config
    let config_path = "./config";
    let config_env_prefix = "APPCFG";
    let config = config::load_config(config_path, config_env_prefix).unwrap();

    // Seed the master and sub-master rngs
    let mut master_rng = Xoshiro256PlusPlus::seed_from_u64(config.seed);
    let mut message_generation_rng = Xoshiro256PlusPlus::from_rng(&mut master_rng);

    // Here we go
    println!("{}", serde_saphyr::to_string(&config).unwrap());

    for _ in 0..1 {
        let _ = Chat::new_with_rng(
            config.message_generation.iat_distribution.clone(),
            Xoshiro256PlusPlus::from_rng(&mut message_generation_rng),
        );
    }

    let mut provider = Provider::default();
    let provider_mbox = Mailbox::new();
    let mut client1 = Client::new("client1".to_owned(), "client2".to_owned());
    let client1_mbox = Mailbox::new();
    client1
        .message_out
        .connect(Provider::receive, &provider_mbox);
    provider.add_client("client1", &client1_mbox);
    let mut client2 = Client::new("client2".to_owned(), "client1".to_owned());
    let client2_mbox = Mailbox::new();
    client2
        .message_out
        .connect(Provider::receive, &provider_mbox);
    provider.add_client("client2", &client2_mbox);

    // let input = EventSource::new()
    //     .connect(Client::receive, &client1_mbox)
    //     .register(&mut bench);
    // let (sink, _) = event_slot(SinkState::Enabled);
    // client1.message_out.connect_sink(sink);
    let bench = SimInit::new();
    let t0 = MonotonicTime::EPOCH;
    let mut simulation = bench
        .add_model(provider, provider_mbox, "provider")
        .add_model(client1, client1_mbox, "client1")
        .add_model(client2, client2_mbox, "client2")
        .init(t0)?;
    simulation.step_until(Duration::from_secs(10))?;
    Ok(())
}
