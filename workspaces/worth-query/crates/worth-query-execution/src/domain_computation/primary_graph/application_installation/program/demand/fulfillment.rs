//! The installed root binds its prepared source to the actual admitted demand.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryFulfilledRequiredRoot, WorthQueryOutputDemandDenialKind as Kind,
    WorthQueryOutputSourceEvidence,
};
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
impl<Schema, Program, Demand> WorthQueryAdmittedProgramOutput<Schema, Program, Demand>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    pub fn retain_fulfilled_required_root<Root>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        program: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        prepared: &WorthQueryPreparedRequiredOutputSource,
        receipt: &WorthQueryOutputSourceEvidence,
        settlement: &WorthQueryOutputDemandSettlement,
    ) -> Result<WorthQueryFulfilledRequiredRoot, WorthQueryOutputDemandDenial>
    where
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
        RootConnection<Schema, Root>:
            WorthQueryApplicationRequiredOutputConnection<Schema, Demand = Demand>,
    {
        let runtime = program.backing.runtime();
        let execution = phase
            .execution_request_for(&runtime.inner.runtime.product_runtime)
            .map_err(|cause| {
                WorthQueryOutputDemandDenial::new(Kind::ExecutionRequest(cause.into()), "")
            })?;
        if !program.contains_output_root::<Root>() || self.target_feature != TypeId::of::<
            <RootConnectionRef<Schema, Root> as ApplicationConnectionShape<Schema>>::TargetFeature>() {
            return Err(WorthQueryOutputDemandDenial::new(Kind::ForeignDemand, ""));
        }
        runtime.inner.runtime.output_demands.validate_owned_root_kind(prepared,
            crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind::Required(TypeId::of::<Root>()))?;
        self.admitted.retain_fulfilled_root(
            runtime.inner(),
            &execution,
            prepared,
            receipt,
            settlement,
        )
    }
}
