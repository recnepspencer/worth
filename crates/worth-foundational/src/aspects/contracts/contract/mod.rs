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
        &self.key
    }

    pub fn identity(&self) -> AspectIdentity {
        self.identity
    }

    pub fn revision(&self) -> AspectContractRevision {
        self.revision
    }

    pub fn shape(&self) -> &AspectShape {
        &self.shape
    }

    pub fn masks(&self) -> &AspectMaskContract {
        &self.masks
    }

    pub fn absence(&self) -> AbsenceLaw {
        self.absence
    }

    pub fn equivalence(&self) -> AspectEquivalenceBasis {
        self.equivalence
    }

    pub fn evolution(&self) -> crate::aspects::evolution::AspectEvolutionPolicy {
        self.evolution
    }

    /// Stable logical width of this contract's native semantic material.
    pub fn semantic_byte_width(&self) -> usize {
        let shape = match &self.shape {
            AspectShape::Struct(shape) => shape.fields().iter().fold(1_usize, |total, field| {
                total
                    .saturating_add(field.key().as_str().len())
                    .saturating_add(4)
            }),
            _ => 2,
        };
        self.key
            .as_str()
            .len()
            .saturating_add(20)
            .saturating_add(shape)
    }

    /// Allocator capacity retained exclusively by this contract, excluding
    /// its inline `AspectContract` storage.
    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        self.key
            .owned_allocation_capacity_bytes()
            .saturating_add(match &self.shape {
                AspectShape::Struct(shape) => shape.owned_allocation_capacity_bytes(),
                _ => 0,
            })
    }
}
