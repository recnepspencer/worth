//! Sealed test doors over this module's private members.
use super::*;

impl<Schema, Family> TypedRequiredSuccessor<Schema, Family>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
    FamilySourceValue<Schema, Family>:
        WorthQueryApplicationProjection<Schema, FamilySourceQuery<Schema, Family>> + 'static,
    FamilySourceQuery<Schema, Family>: 'static,
{
    #[doc(hidden)]
    pub fn run_again_for_test(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &dyn crate::ports::ApplicationRuntimeRef<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        branch: crate::basis::WorthQueryProductBranch,
        invocation: &worth_query_execution_lineage::facade::domain_computation::primary_graph::application_contribution::WorthQueryProducerCommitAuthority,
        performed: &std::cell::RefCell<PerformedMembers>,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial> {
        Self::run_again(
            self,
            execution,
            phase,
            runtime.runtime_inner(),
            principal,
            request,
            branch,
            invocation,
            performed,
        )
    }
}
