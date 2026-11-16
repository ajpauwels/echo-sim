use std::{fmt, ops::Bound};

use rand::{Rng, prelude::*};
use rand_distr::Exp1;
use serde::{Deserialize, Serialize};

use crate::{
    affine::Affine,
    config::DistributionConfig,
    mixture::Mixture,
    truncated::{self, Truncated},
};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
// #[serde(try_from = "DistributionConfig", into = "DistributionConfig")]
#[serde(try_from = "DistributionConfig")]
pub enum Distribution {
    #[default]
    Exp1,
    Exp {
        distribution: rand_distr::Exp<f64>,
        lambda: f64,
    },
    Affine {
        distribution: Affine<Box<Self>, f64, f64>,
    },
    Truncated(Truncated<Box<Self>, f64>),
    Mixture(Mixture<Self>),
}

impl rand_distr::Distribution<f64> for Distribution {
    fn sample<R>(&self, rng: &mut R) -> f64
    where
        R: Rng + ?Sized,
    {
        match self {
            Distribution::Exp1 => rand::rng().sample(Exp1),
            Distribution::Exp {
                distribution,
                lambda: _,
            } => distribution.sample(rng),
            Distribution::Truncated(d) => d.sample(rng),
            Distribution::Mixture(d) => d.sample(rng),
            Distribution::Affine { distribution } => distribution.sample(rng),
        }
    }
}

impl rand_distr::Distribution<f64> for Box<Distribution> {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        (**self).sample(rng)
    }
}

impl From<Distribution> for DistributionConfig {
    fn from(value: Distribution) -> Self {
        (&value).into()
    }
}

impl From<&Distribution> for DistributionConfig {
    fn from(value: &Distribution) -> Self {
        match value {
            Distribution::Exp1 => DistributionConfig::Exp1,
            Distribution::Exp { lambda, .. } => DistributionConfig::Exp { lambda: *lambda },
            Distribution::Affine { distribution: a } => DistributionConfig::Affine {
                scale: Some(*a.scale()),
                offset: Some(*a.offset()),
                scale_offset: a.scale_offset(),
                distribution: Box::new(DistributionConfig::from(&**a.inner_distribution())),
            },
            Distribution::Truncated(_) => todo!(),
            Distribution::Mixture(_) => todo!(),
        }
    }
}

impl TryFrom<DistributionConfig> for Distribution {
    type Error = ConfigToRuntimeError;

    fn try_from(value: DistributionConfig) -> Result<Self, ConfigToRuntimeError> {
        (&value).try_into()
    }
}

impl TryFrom<&DistributionConfig> for Distribution {
    type Error = ConfigToRuntimeError;

    fn try_from(c: &DistributionConfig) -> Result<Self, ConfigToRuntimeError> {
        Ok(match c {
            DistributionConfig::Exp1 => Distribution::Exp1,
            DistributionConfig::Exp { lambda } => Distribution::Exp {
                distribution: rand_distr::Exp::new(*lambda)?,
                lambda: *lambda,
            },
            DistributionConfig::Affine {
                scale,
                offset,
                scale_offset,
                distribution,
            } => Distribution::Affine {
                distribution: Affine::new(
                    Box::new(Distribution::try_from(distribution.as_ref())?),
                    scale.unwrap_or(1.0),
                    offset.unwrap_or(0.0),
                    *scale_offset,
                ),
            },
            DistributionConfig::Truncated {
                min,
                max,
                distribution,
            } => {
                let range = (
                    min.map_or(Bound::Unbounded, Bound::Excluded),
                    max.map_or(Bound::Unbounded, Bound::Included),
                );
                Distribution::Truncated(Truncated::new(
                    Box::new(Distribution::try_from(distribution.as_ref())?),
                    range,
                )?)
            }
            DistributionConfig::Mixture { components } => {
                let parts = components
                    .iter()
                    .map(|w| Ok((Distribution::try_from(&w.distribution)?, w.weight)))
                    .collect::<Result<Vec<_>, ConfigToRuntimeError>>()?;
                Distribution::Mixture(Mixture::new(parts))
            }
        })
    }
}

#[derive(Debug)]
pub enum ConfigToRuntimeError {
    ExpError(rand_distr::ExpError),
    TruncatedError(truncated::TruncatedError),
}

impl fmt::Display for ConfigToRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigToRuntimeError::ExpError(e) => write!(f, "{}", &e.to_string()),
            ConfigToRuntimeError::TruncatedError(e) => write!(f, "{}", &e.to_string()),
        }
    }
}

impl std::error::Error for ConfigToRuntimeError {}

impl From<rand_distr::ExpError> for ConfigToRuntimeError {
    fn from(value: rand_distr::ExpError) -> Self {
        Self::ExpError(value)
    }
}

impl From<truncated::TruncatedError> for ConfigToRuntimeError {
    fn from(value: truncated::TruncatedError) -> Self {
        Self::TruncatedError(value)
    }
}
