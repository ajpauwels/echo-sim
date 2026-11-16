use std::{error::Error, fmt::Display};

use config::{Config as ExternalConfig, ConfigError as ExternalConfigError};
use serde::{Deserialize, Serialize};

use crate::distribution::Distribution;

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DistributionConfig {
    #[default]
    Exp1,
    Exp {
        lambda: f64,
    },
    Affine {
        scale: Option<f64>,
        offset: Option<f64>,
        #[serde(default)]
        scale_offset: bool,
        distribution: Box<DistributionConfig>,
    },
    Truncated {
        min: Option<f64>,
        max: Option<f64>,
        distribution: Box<DistributionConfig>,
    },
    Mixture {
        components: Vec<WeightedDistributionConfig>,
    },
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct WeightedDistributionConfig {
    pub weight: f64,
    pub distribution: DistributionConfig,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct MessageGeneration {
    pub iat_distribution: Distribution,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub seed: u64,
    pub message_generation: MessageGeneration,
}

#[derive(Debug)]
pub struct ConfigError {
    parent: ExternalConfigError,
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.parent)
    }
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.parent)
    }
}

impl From<ExternalConfigError> for ConfigError {
    fn from(value: ExternalConfigError) -> Self {
        ConfigError { parent: value }
    }
}

pub fn load_config(path: &str, prefix: &str) -> Result<Config, ConfigError> {
    let mut config_builder = ExternalConfig::builder();
    let paths = std::fs::read_dir(path)
        .map_err(|_| ExternalConfigError::Message(format!("Could not read directory at {path}")))?
        .filter_map(|result| result.ok())
        .map(|de| de.path());
    for path in paths {
        config_builder = match path.as_path().to_str() {
            Some(s) => {
                if s.ends_with(".yaml") {
                    config_builder.add_source(config::File::with_name(s))
                } else {
                    config_builder
                }
            }
            None => config_builder,
        };
    }
    Ok(config_builder
        .add_source(config::Environment::with_prefix(prefix).separator("_"))
        .build()?
        .try_deserialize::<Config>()?)
}
