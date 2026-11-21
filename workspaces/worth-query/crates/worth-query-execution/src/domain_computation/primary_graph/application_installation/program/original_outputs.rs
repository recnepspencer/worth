//! Whole-tree qualification at the original tree's canonically linked endpoint.
use super::program_runtime::WorthQueryProgramApplicationRuntime;
use crate::domain_computation::primary_graph::application_output_demand::{
    original_endpoint, WorthQueryCompletedOriginalReadOutputs, WorthQueryOriginalReadOutputDenial,
    WorthQueryOutputDemandSettlement,
};
use crate::domain_computation::primary_graph::{
    application_query::WorthQueryRetainedQueryDependencies, WorthQueryAdvancementPhase,
    WorthQueryApplicationReadObservation, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind,
};
use crate::publication_boundary::WorthQueryProgramPublicationAccess;
use std::sync::Arc;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_program::{
    ApplicationOutputGraphShape, ApplicationProgramDefinition,
};
use worth_query_installation::facade::ApplicationSchema;

impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>>
    WorthQueryProgramApplicationRuntime<S, P>
{
    /// Publication supplies the original closed tree's immutable root/dependent
    /// collection. World determines its endpoint before the selected typed root
    /// and original whole-set/discovery are qualified at that exact occurrence.
    pub fn qualify_original_read_outputs<'s, R: ApplicationOutputGraphShape<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        scope: &WorthQueryRequestScope,
        source: &Arc<WorthQueryApplicationReadObservation>,
        root: &'s WorthQueryOutputDemandSettlement,
        dependents: impl Iterator<Item = &'s WorthQueryOutputDemandSettlement> + Clone,
        dependencies: Option<&WorthQueryRetainedQueryDependencies>,
    ) -> Result<WorthQueryCompletedOriginalReadOutputs, WorthQueryOriginalReadOutputDenial> {
        self.qualify_original_read_outputs_with_header::<R>(
            phase,
            access,
            scope,
            source,
            root,
            dependents,
            dependencies,
            std::mem::size_of::<WorthQueryCompletedOriginalReadOutputs>() as u64,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn qualify_original_read_outputs_with_header<
        's,
        R: ApplicationOutputGraphShape<S>,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        scope: &WorthQueryRequestScope,
        source: &Arc<WorthQueryApplicationReadObservation>,
        root: &'s WorthQueryOutputDemandSettlement,
        dependents: impl Iterator<Item = &'s WorthQueryOutputDemandSettlement> + Clone,
        dependencies: Option<&WorthQueryRetainedQueryDependencies>,
        header_bytes: u64,
    ) -> Result<WorthQueryCompletedOriginalReadOutputs, WorthQueryOriginalReadOutputDenial> {
        use WorthQueryOriginalReadOutputDenial as Denial;
        let runtime = self.runtime();
        let execution = phase
            .checked_execution_request_for(&runtime.inner.runtime.product_runtime)
            .map_err(|cause| {
                Denial::Output(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ExecutionRequest(cause),
                    "original output endpoint qualification",
                ))
            })?;
        // Admit the real D before inspecting the original output collection.
        let selected_source = runtime
            .select_application_read_observation(phase, source)
            .map_err(Denial::Observation)?;
        let mut settlements = std::iter::once(root).chain(dependents.clone());
        let mut reads = std::iter::once(root).chain(dependents.clone());
        let endpoint = original_endpoint::derive(&execution, source, &mut settlements, &mut reads)
            .map_err(Denial::Output)?;
        drop(selected_source);
        self.validate_selected_output_root::<R>(phase, access, &endpoint)
            .map_err(Denial::Program)?;
        let selected = runtime
            .select_application_read_observation(phase, &endpoint)
            .map_err(Denial::Observation)?;
        let (output, memory) = selected
            .qualify_completed_output_set(
                phase,
                access,
                scope,
                root,
                dependents,
                dependencies,
                header_bytes,
            )
            .map_err(Denial::Currentness)?;
        Ok(WorthQueryCompletedOriginalReadOutputs::from_parts(
            worth_query_execution_query::facade::ports::worth_query_query_crate_seal(),
            Arc::clone(source),
            Arc::clone(root.retained_read_ref()),
            root.posture(),
            output,
            memory,
        ))
    }
}
