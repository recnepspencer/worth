//! Original checkpoint coordinates exchanged by Native, World and their composer.
use super::HomeIdentity;
use serde::{Deserialize, Serialize};

/// Descriptive affinity; none of these fields grants source or receiver authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CheckpointAdmissionScope {
    Home {
        home: HomeIdentity,
        native_origin: [u8; 32],
        complete_source_digest: [u8; 32],
        world_through: [u64; 3],
        world_image_digest: [u8; 32],
    },
    Interim {
        transport_origin: [u8; 32],
        registry_revision: u64,
        world_snapshot_body_digest: [u8; 32],
    },
}

/// The same base Native and World cut, before the outer package is sealed.
/// A serialized cut is DATA; admission also requires the actual protected source.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointAdmissionCut {
    scope: CheckpointAdmissionScope,
    canonical_through: u64,
    transitions_through: u64,
    checkpoint_digest: [u8; 32],
}

impl CheckpointAdmissionCut {
    pub const fn new(
        scope: CheckpointAdmissionScope,
        canonical_through: u64,
        transitions_through: u64,
        checkpoint_digest: [u8; 32],
    ) -> Self {
        Self {
            scope,
            canonical_through,
            transitions_through,
            checkpoint_digest,
        }
    }

    pub const fn scope(&self) -> &CheckpointAdmissionScope {
        &self.scope
    }
    pub const fn canonical_through(&self) -> u64 {
        self.canonical_through
    }
    pub const fn transitions_through(&self) -> u64 {
        self.transitions_through
    }
    pub const fn checkpoint_digest(&self) -> &[u8; 32] {
        &self.checkpoint_digest
    }
}

/// Hash of the captured installed product meaning and its exact role.
/// Copying this identity cannot bind a fresh receiver.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CheckpointProductBindingIdentity([u8; 32]);

impl CheckpointProductBindingIdentity {
    pub const fn from_observed_digest(digest: [u8; 32]) -> Self {
        Self(digest)
    }
    pub const fn digest(&self) -> &[u8; 32] {
        &self.0
    }
}
