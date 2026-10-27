use super::WorthQueryOperationRelationTouchScope;
use crate::application_schema::WorthQueryInstalledApplicationAspectContract;
use std::sync::Arc;
use worth_foundational::facade::{AspectContract, CanonicalBasisReadyArtifact, CanonicalFieldPath};

/// Installed create-only field scope, retaining the actual native relation contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOperationRelationFieldInitializationScope {
    relation: WorthQueryOperationRelationTouchScope,
    contract: AspectContract,
    canonical_contract_basis: Arc<CanonicalBasisReadyArtifact>,
    canonical_contract_material: Arc<str>,
    field_path: CanonicalFieldPath,
}

impl WorthQueryOperationRelationFieldInitializationScope {
    pub(crate) fn new(
        relation: WorthQueryOperationRelationTouchScope,
        installed: &WorthQueryInstalledApplicationAspectContract,
        field_path: CanonicalFieldPath,
    ) -> Self {
        Self {
            relation,
            contract: installed.contract().clone(),
            canonical_contract_basis: installed.retain_canonical_contract_basis(),
            canonical_contract_material: installed.retain_canonical_contract_material(),
            field_path,
        }
    }
    pub const fn relation(&self) -> &WorthQueryOperationRelationTouchScope {
        &self.relation
    }
    pub const fn contract(&self) -> &AspectContract {
        &self.contract
    }
    pub const fn field_path(&self) -> &CanonicalFieldPath {
        &self.field_path
    }
    pub(crate) fn canonical_contract_basis(&self) -> &CanonicalBasisReadyArtifact {
        &self.canonical_contract_basis
    }
    pub(crate) fn canonical_contract_material(&self) -> &str {
        &self.canonical_contract_material
    }
}
