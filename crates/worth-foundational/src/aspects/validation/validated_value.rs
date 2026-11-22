use worth_proof::{Artifact, PhaseMarker};

use crate::aspects::contracts::AspectContract;
use crate::aspects::identity::{AspectContractRevision, AspectIdentity};
use crate::aspects::keys::AspectKey;
use crate::aspects::structs::StructAspectValue;
use crate::values::AspectValue;
use serde::Serialize;
#[cfg(test)]
mod clone_observation;
mod court_doors_for_test;
#[cfg(test)]
pub(in crate::aspects) use clone_observation::CloneObservation;

#[derive(PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct ContractValidatedAspectValue {
    value: std::sync::Arc<ValidatedAspect>,
}

#[derive(Debug, Clone, Eq, Serialize)]
struct ValidatedAspect {
    key: AspectKey,
    #[serde(skip)]
    contract: AspectContract,
    contract_identity: AspectIdentity,
    contract_revision: AspectContractRevision,
    kind: ContractValidatedAspectValueKind,
    #[serde(skip)]
    birth_seq: Option<u64>,
}

impl PartialEq for ValidatedAspect {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.contract == other.contract
            && self.contract_identity == other.contract_identity
            && self.contract_revision == other.contract_revision
            && self.kind == other.kind
    }
}

impl Clone for ContractValidatedAspectValue {
    fn clone(&self) -> Self {
        #[cfg(test)]
        clone_observation::record_clone();
        Self {
            value: std::sync::Arc::clone(&self.value),
        }
    }
}

impl ContractValidatedAspectValue {
    /// Newly admitted cell, cloned key and reachable immutable contract backing.
    /// Payload storage is described separately by its concrete value owner.
    pub fn admission_storage_bytes(contract: &AspectContract) -> Option<usize> {
        Self::admission_cell_storage_bytes()?.checked_add(contract.key().as_str().len())
    }

    /// Actual admitted-value Arc cell, excluding its separately owned key and payload.
    pub fn admission_cell_storage_bytes() -> Option<usize> {
        Some(
            std::alloc::Layout::new::<[usize; 2]>()
                .extend(std::alloc::Layout::new::<ValidatedAspect>())
                .ok()?
                .0
                .pad_to_align()
                .size(),
        )
    }

    pub(in crate::aspects) fn struct_retained_storage_bytes(
        contract: &AspectContract,
        fields: usize,
    ) -> Option<usize> {
        Self::admission_storage_bytes(contract)?
            .checked_add(contract.owned_allocation_capacity_bytes())?
            .checked_add(fields)
    }

    pub(crate) fn scalar(contract: AspectContract, value: AspectValue) -> Self {
        Self {
            value: std::sync::Arc::new(ValidatedAspect {
                key: contract.key().clone(),
                contract_identity: contract.identity(),
                contract_revision: contract.revision(),
                contract,
                kind: ContractValidatedAspectValueKind::Scalar(value),
                birth_seq: crate::aspects::allocation_visitor::next_allocation_birth_seq(),
            }),
        }
    }

    pub(crate) fn struct_value(contract: AspectContract, value: StructAspectValue) -> Self {
        Self {
            value: std::sync::Arc::new(ValidatedAspect {
                key: contract.key().clone(),
                contract_identity: contract.identity(),
                contract_revision: contract.revision(),
                contract,
                kind: ContractValidatedAspectValueKind::Struct(value),
                birth_seq: crate::aspects::allocation_visitor::next_allocation_birth_seq(),
            }),
        }
    }

    pub fn key(&self) -> &AspectKey {
        &self.value.key
    }

    pub fn contract_revision(&self) -> AspectContractRevision {
        self.value.contract_revision
    }

    pub fn contract_identity(&self) -> AspectIdentity {
        self.value.contract_identity
    }

    pub fn contract(&self) -> &AspectContract {
        &self.value.contract
    }

    /// Stable logical width of the native semantic material retained here.
    ///
    /// This is deliberately independent of allocator layout and formatting.
    pub fn semantic_byte_width(&self) -> usize {
        let value = match &self.value.kind {
            ContractValidatedAspectValueKind::Scalar(value) => value.semantic_byte_width(),
            ContractValidatedAspectValueKind::Struct(value) => value.semantic_byte_width(),
        };
        self.value
            .contract
            .semantic_byte_width()
            .saturating_add(value)
    }

    pub fn view(&self) -> ContractValidatedAspectValueView<'_> {
        match &self.value.kind {
            ContractValidatedAspectValueKind::Scalar(value) => {
                ContractValidatedAspectValueView::Scalar(value)
            }
            ContractValidatedAspectValueKind::Struct(value) => {
                ContractValidatedAspectValueView::Struct(value)
            }
        }
    }

    /// Reachable payload allocation, including shared backing; not a clone quote.
    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        let value_bytes = match &self.value.kind {
            ContractValidatedAspectValueKind::Scalar(value) => {
                value.owned_allocation_capacity_bytes()
            }
            ContractValidatedAspectValueKind::Struct(value) => {
                value.owned_allocation_capacity_bytes()
            }
        };
        self.value
            .key
            .owned_allocation_capacity_bytes()
            .saturating_add(self.value.contract.owned_allocation_capacity_bytes())
            .saturating_add(value_bytes)
            .saturating_add(
                Self::admission_cell_storage_bytes()
                    .expect("the fixed validated aspect Arc layout fits the address space"),
            )
    }

    pub(crate) fn shares_allocation(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.value, &other.value)
    }

    fn admission_cell_bytes() -> u64 {
        Self::admission_cell_storage_bytes()
            .expect("the fixed validated aspect Arc layout fits the address space") as u64
    }

    pub(crate) fn visit_allocations(
        &self,
        ancestor_unique: bool,
        visitor: &mut dyn crate::aspects::FoundationalAllocationVisitor,
    ) {
        let strong_count = std::sync::Arc::strong_count(&self.value);
        let unique = ancestor_unique && strong_count == 1;
        let observation = crate::aspects::FoundationalAllocationObservation {
            id: crate::aspects::allocation_visitor::arc_id(&self.value),
            bytes: Self::admission_cell_bytes(),
            unique,
            strong_count,
            birth_seq: self.value.birth_seq,
            kind: crate::aspects::FoundationalAllocationKind::ValueBacking,
        };
        if !visitor.visit(observation) {
            return;
        }
        self.value
            .key
            .visit_allocations(unique, self.value.birth_seq, visitor);
        self.value.contract.visit_allocations(unique, visitor);
        match &self.value.kind {
            ContractValidatedAspectValueKind::Scalar(value) => {
                value.visit_allocations(unique, self.value.birth_seq, visitor);
            }
            ContractValidatedAspectValueKind::Struct(value) => {
                value.visit_allocations(unique, visitor);
            }
        }
    }

    pub(crate) fn visit_new_allocations(
        &self,
        previous: Option<&Self>,
        ancestor_unique: bool,
        visitor: &mut dyn crate::aspects::FoundationalAllocationVisitor,
    ) {
        let Some(previous) = previous else {
            self.visit_allocations(ancestor_unique, visitor);
            return;
        };
        if self.shares_allocation(previous) {
            return;
        }
        let strong_count = std::sync::Arc::strong_count(&self.value);
        let unique = ancestor_unique && strong_count == 1;
        let observation = crate::aspects::FoundationalAllocationObservation {
            id: crate::aspects::allocation_visitor::arc_id(&self.value),
            bytes: Self::admission_cell_bytes(),
            unique,
            strong_count,
            birth_seq: self.value.birth_seq,
            kind: crate::aspects::FoundationalAllocationKind::ValueBacking,
        };
        if !visitor.visit(observation) {
            return;
        }
        self.value
            .key
            .visit_allocations(unique, self.value.birth_seq, visitor);
        if !self.value.contract.shares_definition(&previous.value.contract) {
            self.value.contract.visit_allocations(unique, visitor);
        }
        match (&self.value.kind, &previous.value.kind) {
            (
                ContractValidatedAspectValueKind::Struct(current),
                ContractValidatedAspectValueKind::Struct(prior),
            ) => current.visit_new_allocations(prior, unique, visitor),
            (ContractValidatedAspectValueKind::Struct(current), _) => {
                current.visit_allocations(unique, visitor);
            }
            (
                ContractValidatedAspectValueKind::Scalar(current),
                ContractValidatedAspectValueKind::Scalar(prior),
            ) => current.visit_new_allocations(Some(prior), unique, self.value.birth_seq, visitor),
            (ContractValidatedAspectValueKind::Scalar(current), _) => {
                current.visit_allocations(unique, self.value.birth_seq, visitor);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
enum ContractValidatedAspectValueKind {
    Scalar(AspectValue),
    Struct(StructAspectValue),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractValidatedAspectValueView<'a> {
    Scalar(&'a AspectValue),
    Struct(&'a StructAspectValue),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContractValidated;

impl PhaseMarker for ContractValidated {}

pub type ContractValidatedAspectArtifact =
    Artifact<ContractValidated, ContractValidatedAspectValue>;

impl std::fmt::Debug for ContractValidatedAspectValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContractValidatedAspectValue")
            .field("key", &self.value.key)
            .field("contract", &self.value.contract)
            .field("contract_identity", &self.value.contract_identity)
            .field("contract_revision", &self.value.contract_revision)
            .field("kind", &self.value.kind)
            .finish()
    }
}
