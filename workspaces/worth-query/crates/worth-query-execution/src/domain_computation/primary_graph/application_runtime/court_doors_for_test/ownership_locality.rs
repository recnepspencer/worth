//! Forward the installed ownership-locality observation.
use crate::basis::WorthQueryProductBranch;
use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::WorthQueryAdvancementDenial;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::{
    identity::EntityId, runtime::CurrentOwnershipLocalityForTest, transactions::AspectFieldPatch,
};

impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    #[doc(hidden)]
    pub fn probe_current_ownership_locality_for_test(
        &self,
        branch: WorthQueryProductBranch,
        owner: EntityId,
        request: &WorthQueryRequestScope,
        seed_fields: impl Fn(u64) -> AspectFieldPatch,
        value_fields: impl Fn(u64) -> AspectFieldPatch,
    ) -> Result<CurrentOwnershipLocalityForTest, WorthQueryAdvancementDenial> {
        self.inner.probe_current_ownership_locality_for_test(
            branch,
            owner,
            request,
            seed_fields,
            value_fields,
        )
    }
}
