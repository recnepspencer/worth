use core::num::NonZeroU64;

use serde::{Deserialize, Serialize};

/// Generation encoded by the artifact, or an explicit absence of that field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PhysicalArtifactGeneration {
    NotEncoded,
    Encoded(NonZeroU64),
}

impl PhysicalArtifactGeneration {
    pub const fn encoded(value: u64) -> Option<Self> {
        match NonZeroU64::new(value) {
            Some(value) => Some(Self::Encoded(value)),
            None => None,
        }
    }
}
