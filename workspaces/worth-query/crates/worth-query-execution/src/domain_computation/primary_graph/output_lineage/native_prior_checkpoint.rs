//! Portable performed locator or original recovered metadata; neither grants Ready.
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity;

pub(super) enum NativePriorCheckpointLocator {
    Performed { producer: String, source: [u8; 32] },
    Recovered(WorthQueryAcceptedOutputCheckpointIdentity),
}

impl NativePriorCheckpointLocator {
    /// Retain descriptive predecessor coordinates; native readmission grants custody.
    pub(super) fn try_retain(&self) -> Option<Self> {
        Some(match self {
            Self::Performed { producer, source } => {
                let mut retained = String::new();
                retained.try_reserve_exact(producer.len()).ok()?;
                retained.push_str(producer);
                Self::Performed {
                    producer: retained,
                    source: *source,
                }
            }
            // The accepted descriptor shares its immutable heaps and original payer.
            Self::Recovered(checkpoint) => Self::Recovered(checkpoint.clone()),
        })
    }

    pub(super) fn recovered(&self) -> Option<&WorthQueryAcceptedOutputCheckpointIdentity> {
        match self {
            Self::Recovered(checkpoint) => Some(checkpoint),
            Self::Performed { .. } => None,
        }
    }
    pub(super) fn producer(&self) -> &str {
        match self {
            Self::Performed { producer, .. } => producer,
            Self::Recovered(checkpoint) => &checkpoint.producer,
        }
    }
    pub(super) fn source(&self) -> [u8; 32] {
        match self {
            Self::Performed { source, .. } => *source,
            Self::Recovered(checkpoint) => checkpoint.source,
        }
    }
}
