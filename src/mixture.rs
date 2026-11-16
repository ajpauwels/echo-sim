use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mixture<D> {
    weights: rand_distr::weighted::WeightedIndex<f64>,
    distributions: Vec<D>,
}

impl<D> Mixture<D> {
    pub fn new(distributions_and_weights: Vec<(D, f64)>) -> Self {
        let weights = rand_distr::weighted::WeightedIndex::new(
            distributions_and_weights.iter().map(|(_, weight)| *weight),
        )
        .unwrap();
        let distributions = distributions_and_weights
            .into_iter()
            .map(|(d, _)| d)
            .collect();
        Self {
            weights,
            distributions,
        }
    }
}

impl<T, D> rand_distr::Distribution<T> for Mixture<D>
where
    D: rand_distr::Distribution<T>,
{
    fn sample<R>(&self, rng: &mut R) -> T
    where
        R: Rng + ?Sized,
    {
        let distribution = &self.distributions[self.weights.sample(rng)];
        distribution.sample(rng)
    }
}
