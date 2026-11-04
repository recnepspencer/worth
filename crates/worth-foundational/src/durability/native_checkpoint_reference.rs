//! Logical Native coordinates within the admission image's source and cut.
use super::{identity_seal, DurableIdentity};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Recorded meaning only. Source/Home affinity belongs to the enclosing cut.
/// No runtime instance or root allocation identity can occupy these fields.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCheckpointBranchObservation<'a> {
    branch: Cow<'a, str>,
    generation: u64,
    target: NativeCheckpointBranchTarget<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeCheckpointBranchReference<'a> {
    observation: NativeCheckpointBranchObservation<'a>,
    truth_version: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum NativeCheckpointBranchTarget<'a> {
    Empty,
    Canonical {
        commit: u64,
        version: u64,
        parents: Cow<'a, [u64]>,
        truth_root: [u8; 32],
        schema_root: [u8; 32],
    },
}

impl<'a> NativeCheckpointBranchObservation<'a> {
    pub fn from_recorded(
        branch: &'a str,
        generation: u64,
        target: NativeCheckpointBranchTarget<'a>,
    ) -> Self {
        Self {
            branch: Cow::Borrowed(branch),
            generation,
            target,
        }
    }
    pub fn branch(&self) -> &str {
        &self.branch
    }
    pub const fn generation(&self) -> u64 {
        self.generation
    }
    pub fn target(&self) -> &NativeCheckpointBranchTarget<'a> {
        &self.target
    }
}

impl<'a> NativeCheckpointBranchReference<'a> {
    pub fn from_recorded(
        observation: NativeCheckpointBranchObservation<'a>,
        truth_version: u64,
    ) -> Self {
        Self {
            observation,
            truth_version,
        }
    }
    pub fn observation(&self) -> &NativeCheckpointBranchObservation<'a> {
        &self.observation
    }
    pub const fn truth_version(&self) -> u64 {
        self.truth_version
    }
}

impl identity_seal::Sealed for NativeCheckpointBranchObservation<'_> {}
impl DurableIdentity for NativeCheckpointBranchObservation<'_> {}
impl identity_seal::Sealed for NativeCheckpointBranchReference<'_> {}
impl DurableIdentity for NativeCheckpointBranchReference<'_> {}
impl identity_seal::Sealed for NativeCheckpointBranchTarget<'_> {}
impl DurableIdentity for NativeCheckpointBranchTarget<'_> {}
