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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ValidatedAspect {
    key: AspectKey,
    #[serde(skip)]
    contract: AspectContract,
    contract_identity: AspectIdentity,
    contract_revision: AspectContractRevision,
    kind: ContractValidatedAspectValueKind,
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
    pub(crate) fn scalar(contract: AspectContract, value: AspectValue) -> Self {
        Self {
            value: std::sync::Arc::new(ValidatedAspect {
                key: contract.key().clone(),
                contract_identity: contract.identity(),
                contract_revision: contract.revision(),
                contract,
                kind: ContractValidatedAspectValueKind::Scalar(value),
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
            .saturating_add(std::mem::size_of::<ValidatedAspect>())
            .saturating_add(2 * std::mem::size_of::<usize>())
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
