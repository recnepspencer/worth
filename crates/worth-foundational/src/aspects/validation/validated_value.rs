use worth_proof::{Artifact, PhaseMarker};

use crate::aspects::contracts::AspectContract;
use crate::aspects::identity::{AspectContractRevision, AspectIdentity};
use crate::aspects::keys::AspectKey;
use crate::aspects::structs::StructAspectValue;
use crate::values::AspectValue;
use serde::Serialize;
mod court_doors_for_test;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContractValidatedAspectValue {
    key: AspectKey,
    #[serde(skip)]
    contract: AspectContract,
    contract_identity: AspectIdentity,
    contract_revision: AspectContractRevision,
    kind: ContractValidatedAspectValueKind,
}

impl ContractValidatedAspectValue {
    pub(crate) fn scalar(contract: AspectContract, value: AspectValue) -> Self {
        Self {
            key: contract.key().clone(),
            contract_identity: contract.identity(),
            contract_revision: contract.revision(),
            contract,
            kind: ContractValidatedAspectValueKind::Scalar(value),
        }
    }

    pub(crate) fn struct_value(contract: AspectContract, value: StructAspectValue) -> Self {
        Self {
            key: contract.key().clone(),
            contract_identity: contract.identity(),
            contract_revision: contract.revision(),
            contract,
            kind: ContractValidatedAspectValueKind::Struct(value),
        }
    }

    pub fn key(&self) -> &AspectKey {
        &self.key
    }

    pub fn contract_revision(&self) -> AspectContractRevision {
        self.contract_revision
    }

    pub fn contract_identity(&self) -> AspectIdentity {
        self.contract_identity
    }

    pub fn contract(&self) -> &AspectContract {
        &self.contract
    }

    /// Stable logical width of the native semantic material retained here.
    ///
    /// This is deliberately independent of allocator layout and formatting.
    pub fn semantic_byte_width(&self) -> usize {
        let value = match &self.kind {
            ContractValidatedAspectValueKind::Scalar(value) => value.semantic_byte_width(),
            ContractValidatedAspectValueKind::Struct(value) => {
                value.fields().fold(0_usize, |total, (field, value)| {
                    total
                        .saturating_add(field.as_str().len())
                        .saturating_add(value.semantic_byte_width())
                })
            }
        };
        self.contract.semantic_byte_width().saturating_add(value)
    }

    pub fn view(&self) -> ContractValidatedAspectValueView<'_> {
        match &self.kind {
            ContractValidatedAspectValueKind::Scalar(value) => {
                ContractValidatedAspectValueView::Scalar(value)
            }
            ContractValidatedAspectValueKind::Struct(value) => {
                ContractValidatedAspectValueView::Struct(value)
            }
        }
    }

    /// Allocator capacity reachable only through this validated value,
    /// excluding its inline representation in the owning collection.
    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        let value_bytes = match &self.kind {
            ContractValidatedAspectValueKind::Scalar(value) => {
                value.owned_allocation_capacity_bytes()
            }
            ContractValidatedAspectValueKind::Struct(value) => {
                value.owned_allocation_capacity_bytes()
            }
        };
        self.key
            .owned_allocation_capacity_bytes()
            .saturating_add(self.contract.owned_allocation_capacity_bytes())
            .saturating_add(value_bytes)
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
