//! Native negative completion of a fixed leaf root, never a partial output tree.
use super::*;
use crate::equality_type_id::EqualityTypeId as TypeId;
use worth_query_declaration::facade::application_program::{
    ApplicationOutputLeaf, ApplicationRequiredOutputRoot,
};
impl<Schema: ApplicationSchema + 'static, Program: ApplicationProgramDefinition<Schema>>
    WorthQueryProgramApplicationRuntime<Schema, Program>
{
    /// Certifies the original failed root and releases only its source lifecycle.
    /// No successful output or publication is created by this operation.
    pub fn finish_unavailable_program_output<Root>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        demand: &mut WorthQueryAdmittedProgramOutput<
            Schema,
            Program,
            WorthQueryProgramRootDemand<Schema, Root>,
        >,
        prepared: &WorthQueryPreparedRequiredOutputSource,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: &WorthQueryRequestScope,
        branch: crate::basis::WorthQueryProductBranch,
        disclosure: WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, WorthQueryProgramRootDemand<Schema, Root>>,
            SourceValue<Schema, WorthQueryProgramRootDemand<Schema, Root>>,
        >,
    ) -> Result<WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenial>
    where
        Root: ApplicationOutputGraphShape<Schema, Dependents = ApplicationOutputLeaf>
            + ApplicationRequiredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    {
        let phase_request =
            admit_unavailable_completion(phase, &self.runtime.product_runtime, || {
                self.contains_output_root::<Root>()
                    && demand.target_feature
                        == TypeId::of::<
                            <RootConnectionRef<Schema, Root> as ApplicationConnectionShape<
                                Schema,
                            >>::TargetFeature,
                        >()
            })?;
        let execution = &phase_request;
        self.runtime.output_demands.validate_owned_root_kind(prepared,
            crate::domain_computation::primary_graph::application_output_demand::PreparedOutputRootKind::Required(TypeId::of::<Root>()))?;
        self.runtime.finish_unavailable_output_demand(
            execution,
            phase,
            &mut demand.admitted,
            prepared,
            principal,
            scope,
            branch,
            disclosure,
        )
    }
}

/// Phase custody precedes the installed root's demand-affinity admission.
fn admit_unavailable_completion<'scope>(
    phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'scope>,
    runtime: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime,
    owns_demand: impl FnOnce() -> bool,
) -> Result<worth_execution::ExecutionRequest<'scope, 'scope>, WorthQueryOutputDemandDenial> {
    let request = phase.execution_request_for(runtime).map_err(|cause| {
        WorthQueryOutputDemandDenial::new(
            crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()),
            "program-source-phase",
        )
    })?;
    if !owns_demand() {
        return Err(WorthQueryOutputDemandDenial::new(
            crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignDemand,
            "unavailable completion differs from the installed root",
        ));
    }
    Ok(request)
}

#[cfg(test)]
#[path = "unavailable/custody_order.rs"]
mod custody_order;
