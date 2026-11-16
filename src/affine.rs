use std::ops::{Add, Mul};

use serde::{Deserialize, Serialize};

use crate::distribution::Distribution;

pub struct AffineConfig {
    scale: Option<f64>,
    offset: Option<f64>,
    scale_offset: bool,
    distribution: DistributionConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Affine<D, S, O> {
    distribution: D,
    scale: S,
    offset: O,
    scale_offset: bool,
}

impl<D, S, O> Affine<D, S, O>
where
    O: Mul<S, Output = O>,
    S: Copy,
{
    pub fn new(distribution: D, scale: S, offset: O, scale_offset: bool) -> Self {
        let offset = if scale_offset { offset * scale } else { offset };
        Self {
            distribution,
            scale,
            offset,
            scale_offset,
        }
    }

    pub fn inner_distribution(&self) -> &D {
        &self.distribution
    }

    pub fn scale(&self) -> &S {
        &self.scale
    }

    pub fn offset(&self) -> &O {
        &self.offset
    }

    pub fn scale_offset(&self) -> bool {
        self.scale_offset
    }
}

impl<D, S, O> rand_distr::Distribution<O> for Affine<D, S, O>
where
    D: rand_distr::Distribution<O>,
    S: Copy,
    O: Mul<S, Output = O> + Add<Output = O> + Copy,
{
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> O {
        self.distribution.sample(rng) * self.scale + self.offset
    }
}

impl From<DistributionConfig> for Distribution {}

// impl<D, S, O> From<Affine<D, S, O>> for Distribution {
//     fn from(value: Affine<D, S, O>) -> Self {
//         Distribution::Affine {
//             distribution: value,
//         }
//     }
// }
