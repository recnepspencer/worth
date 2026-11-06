mod construction;
mod evolution_classification;
mod mask_admission;

pub(crate) use mask_admission::MaskModeAdmission;

use super::{AbsenceLaw, AspectEquivalenceBasis, AspectShape};
use crate::aspects::identity::{AspectContractRevision, AspectIdentity};
use crate::aspects::keys::AspectKey;
use crate::aspects::masks::AspectMaskContract;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AspectContract {
    definition: std::sync::Arc<AspectContractDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AspectContractDefinition {
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
        let shape = match &self.definition.shape {
            AspectShape::Struct(shape) => shape.fields().iter().fold(1_usize, |total, field| {
                total
                    .saturating_add(field.key().as_str().len())
                    .saturating_add(4)
            }),
            _ => 2,
        };
        self.definition
            .key
            .as_str()
            .len()
            .saturating_add(20)
            .saturating_add(shape)
    }

    /// Allocator capacity retained exclusively by this contract, excluding
    /// its inline `AspectContract` storage.
    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        self.definition
            .key
            .owned_allocation_capacity_bytes()
            .saturating_add(match &self.definition.shape {
                AspectShape::Struct(shape) => shape.owned_allocation_capacity_bytes(),
                _ => 0,
            })
    }
}
