use serde::Serialize;
use worth_proof::{Artifact, PhaseMarker};

use super::{AuthoritativeStateAdmissionDenial, CanonicalAspectStateMap};
use crate::aspects::keys::AspectKey;
use crate::aspects::validation::ContractValidatedAspectValue;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthoritativeRecordAspectState {
    aspects: CanonicalAspectStateMap,
}

impl AuthoritativeRecordAspectState {
    pub(crate) fn from_validated_entries(
        entries: impl IntoIterator<Item = ContractValidatedAspectValue>,
    ) -> Result<Self, AuthoritativeStateAdmissionDenial> {
        Ok(Self {
            aspects: CanonicalAspectStateMap::from_validated_entries(entries)?,
        })
    }

    pub(crate) fn from_canonical_map(aspects: CanonicalAspectStateMap) -> Self {
        Self { aspects }
    }

    pub fn aspects(&self) -> &CanonicalAspectStateMap {
        &self.aspects
    }

    pub fn get(&self, key: &AspectKey) -> Option<&ContractValidatedAspectValue> {
        self.aspects.get(key)
    }

    /// A record copy shares the persistent map root and immutable admitted values.
    /// Inline reference storage is already part of the destination record's size;
    /// cloning allocates no additional heap backing. Its fixed shared-reference
    /// work remains part of the Native record-read checkpoint.
    pub fn clone_allocation_capacity_bytes(&self) -> usize {
        0
    }

    /// Owner-accounted allocation reachable from this authoritative state,
    /// excluding the state's inline wrapper.
    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        self.aspects.owned_allocation_capacity_bytes()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthoritativeRecordAspectStateAdmitted;

impl PhaseMarker for AuthoritativeRecordAspectStateAdmitted {}

pub type AuthoritativeRecordAspectStateArtifact =
    Artifact<AuthoritativeRecordAspectStateAdmitted, AuthoritativeRecordAspectState>;
