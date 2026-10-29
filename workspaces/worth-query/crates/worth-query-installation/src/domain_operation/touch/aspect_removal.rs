//! Installation-owned meaning for removal of one exact Optional entity aspect.
use std::sync::Arc;
use worth_foundational::facade::{AspectContract, CanonicalBasisReadyArtifact};
use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;
/// Only installation can issue a whole-parent removal scope.
/// ```compile_fail
/// use worth_query_installation::facade::WorthQueryOperationAspectRemovalScope;
/// let constructor = WorthQueryOperationAspectRemovalScope::new;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOperationAspectRemovalScope {
    schema: ApplicationSchemaBindingIdentity,
    entity: String,
    contract: AspectContract,
    basis: Arc<CanonicalBasisReadyArtifact>,
    material: Arc<str>,
}
impl WorthQueryOperationAspectRemovalScope {
    pub(crate) fn new(
        schema: ApplicationSchemaBindingIdentity,
        entity: String,
        installed: &crate::application_schema::WorthQueryInstalledApplicationAspectContract,
    ) -> Self {
        Self {
            schema,
            entity,
            contract: installed.contract().clone(),
            basis: installed.retain_canonical_contract_basis(),
            material: installed.retain_canonical_contract_material(),
        }
    }
    pub fn schema(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema
    }
    pub fn entity(&self) -> &str {
        &self.entity
    }
    pub fn contract(&self) -> &AspectContract {
        &self.contract
    }
    pub(crate) fn canonical_contract_basis(&self) -> &CanonicalBasisReadyArtifact {
        &self.basis
    }
    pub(crate) fn canonical_contract_material(&self) -> &str {
        &self.material
    }
}
