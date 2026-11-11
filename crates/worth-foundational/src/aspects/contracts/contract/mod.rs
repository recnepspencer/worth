mod construction;
mod evolution_classification;
mod mask_admission;

pub(crate) use mask_admission::MaskModeAdmission;

use super::{AbsenceLaw, AspectEquivalenceBasis, AspectShape};
use crate::aspects::identity::{AspectContractRevision, AspectIdentity};
use crate::aspects::keys::AspectKey;
use crate::aspects::masks::AspectMaskContract;

#[derive(Clone, PartialEq, Eq)]
pub struct AspectContract {
    definition: std::sync::Arc<AspectContractDefinition>,
}

#[derive(Debug, Clone)]
struct AspectContractDefinition {
    semantic_bytes: usize,
    allocation_bytes: usize,
    key: AspectKey,
    identity: AspectIdentity,
    revision: AspectContractRevision,
    shape: AspectShape,
    masks: AspectMaskContract,
    absence: AbsenceLaw,
    equivalence: AspectEquivalenceBasis,
    evolution: crate::aspects::evolution::AspectEvolutionPolicy,
}

impl AspectContract {
    /// Clone shares the immutable definition and allocates no heap backing.
    pub fn clone_allocation_bytes(&self) -> Option<usize> {
        Some(0)
    }

    pub fn key(&self) -> &AspectKey {
        &self.definition.key
    }

    pub fn identity(&self) -> AspectIdentity {
        self.definition.identity
    }

    pub fn revision(&self) -> AspectContractRevision {
        self.definition.revision
    }

    pub fn shape(&self) -> &AspectShape {
        &self.definition.shape
    }

    pub fn masks(&self) -> &AspectMaskContract {
        &self.definition.masks
    }

    pub fn absence(&self) -> AbsenceLaw {
        self.definition.absence
    }

    pub fn equivalence(&self) -> AspectEquivalenceBasis {
        self.definition.equivalence
    }

    pub fn evolution(&self) -> crate::aspects::evolution::AspectEvolutionPolicy {
        self.definition.evolution
    }

    /// Stable logical width of this contract's native semantic material.
    pub fn semantic_byte_width(&self) -> usize {
        self.definition.semantic_bytes
    }

    /// Reachable allocation, including shared backing; not a clone quote.
    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        self.definition.allocation_bytes
    }
}

impl AspectContract {
    pub(crate) fn shares_definition(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.definition, &other.definition)
    }
}
impl std::fmt::Debug for AspectContract {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AspectContract")
            .field("key", self.key())
            .field("identity", &self.identity())
            .field("revision", &self.revision())
            .field("shape", self.shape())
            .field("masks", self.masks())
            .field("absence", &self.absence())
            .field("equivalence", &self.equivalence())
            .field("evolution", &self.evolution())
            .finish()
    }
}

impl PartialEq for AspectContractDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.identity == other.identity
            && self.revision == other.revision
            && self.shape == other.shape
            && self.masks == other.masks
            && self.absence == other.absence
            && self.equivalence == other.equivalence
            && self.evolution == other.evolution
    }
}
impl Eq for AspectContractDefinition {}
