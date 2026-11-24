//! Obtain current publication custody under the installed model-ownership law.
use super::{current_publications::RelationalOwnedModelPublicationRead, *};
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::mvcc::RelationalInstalledApplicationPublicationAuthority;
use crate::runtime::RelationalRuntime;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;

impl RelationalRuntime {
    pub fn observe_owned_model_publications(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        selected: &AdmittedRelationalBranchBasis,
        ownership: &RelationalModelOwnership,
        model: EntityId,
    ) -> Result<RelationalOwnedModelPublicationRead, RelationalModelOwnershipDenial> {
        use RelationalModelOwnershipDenial as Denial;
        if selected.identity().runtime_instance_id() != self.runtime_instance_id()
            || installed.runtime_instance_id != self.runtime_instance_id()
            || installed.schema_digest
                != (*selected.owner_inner())
                    .root
                    .schema_authority()
                    .authority_digest()
        {
            return Err(Denial::ForeignBasis);
        }
        installed
            .require_current_ownership_storage(selected, model)
            .map_err(Denial::Custody)?;
        super::current_publications::observe(
            execution,
            self.runtime_instance_id(),
            selected,
            ownership,
            model,
        )
    }
}
