use std::{collections::BTreeMap, time::Duration};

use nexosim::{
    model::{Context, Model, schedulable},
    ports::Output,
    simulation::Address,
};
use rand::Rng;
use rand_distr::Distribution;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{client::Client, composition::Composition};

#[derive(Serialize, Deserialize)]
pub struct Chat<D, R> {
    pub composition_outs: BTreeMap<String, Output<Composition>>,
    pub iat_distribution: D,
    rng: R,
}

/// UNCOMMENT IF YOU WANT TO INITIALISE WITHOUT CUSTOM RNG
// impl<D> Chat<D, Xoshiro256PlusPlus>
// where
//     D: Distribution<f64> + Send + Serialize + DeserializeOwned + 'static,
// {
//     pub fn new(iat_distribution: D) -> Self {
//         Self::new_with_rng(iat_distribution, rand::make_rng())
//     }
// }

#[Model]
impl<D, R> Chat<D, R>
where
    D: Distribution<f64> + Send + Serialize + DeserializeOwned + 'static,
    R: Rng + Send + Serialize + DeserializeOwned + 'static,
{
    pub fn new_with_rng(iat_distribution: D, rng: R) -> Self {
        Chat {
            composition_outs: BTreeMap::default(),
            iat_distribution,
            rng,
        }
    }

    pub fn add_client(&mut self, id: &str, address: impl Into<Address<Client>>) {
        let mut output = Output::new();
        output.connect(Client::send, address);
        self.composition_outs.insert(id.to_owned(), output);
    }

    #[nexosim(init)]
    async fn init(&mut self, cx: &Context<Self>) {
        let iat = self.iat_distribution.sample(&mut self.rng);
        cx.schedule_event(
            // Clamp to 1 nanosecond if we get 0-samples and it
            // creates problems
            // Duration::from_secs_f64(iat).max(Duration::from_nanos(1)),
            Duration::from_secs_f64(iat),
            schedulable!(Chat::<D, R>::compose),
            (),
        )
        .unwrap()
    }

    #[nexosim(schedulable)]
    async fn compose(&mut self, _: ()) {
        // if let Some(output) = self.composition_outs.get_mut(&author) {
        //     output.send(Composition::new()).await;
        // }
    }
}
