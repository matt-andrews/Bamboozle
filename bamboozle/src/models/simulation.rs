use rand::Rng;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SimulationConfig {
    pub delay: Option<DelayConfig>,
    pub fault: Option<FaultConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum DelayConfig {
    Fixed {
        ms: u64,
    },
    Random {
        #[serde(rename = "minMs")]
        min_ms: u64,
        #[serde(rename = "maxMs")]
        max_ms: u64,
    },
    Gaussian {
        #[serde(rename = "meanMs")]
        mean_ms: f64,
        #[serde(rename = "stdDevMs")]
        std_dev_ms: f64,
    },
}

impl DelayConfig {
    pub fn sample_ms(&self) -> u64 {
        match self {
            Self::Fixed { ms } => *ms,
            Self::Random { min_ms, max_ms } => {
                let (start, end) = if min_ms <= max_ms {
                    (*min_ms, *max_ms)
                } else {
                    (*max_ms, *min_ms)
                };
                rand::thread_rng().gen_range(start..=end)
            }
            Self::Gaussian {
                mean_ms,
                std_dev_ms,
            } => Normal::new(*mean_ms, *std_dev_ms)
                .map(|dist| dist.sample(&mut rand::thread_rng()).max(0.0) as u64)
                .unwrap_or(*mean_ms as u64),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FaultConfig {
    #[serde(rename = "type")]
    pub kind: FaultKind,
    #[serde(default = "FaultConfig::default_probability")]
    pub probability: f64,
}

impl FaultConfig {
    fn default_probability() -> f64 {
        1.0
    }

    pub fn should_trigger(&self) -> bool {
        if self.probability >= 1.0 {
            true
        } else if self.probability <= 0.0 {
            false
        } else {
            rand::thread_rng().gen_bool(self.probability)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum FaultKind {
    ConnectionReset,
    EmptyResponse,
}
