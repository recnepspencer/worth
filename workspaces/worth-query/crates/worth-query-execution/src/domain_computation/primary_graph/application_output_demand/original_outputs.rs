//! Whole-tree qualification at the original tree's canonically linked endpoint.
use super::{WorthQueryOutputDemandSettlement, WorthQueryOutputSettlementPosture};
use crate::domain_computation::primary_graph::application_installation::{
    WorthQueryProgramApplicationRuntime, WorthQuerySelectedProgramOwnerDenial,
};
use crate::publication_boundary::{
    discovery_currentness::{
        WorthQueryRequiredSetCurrentnessDenial, WorthQueryRetainedQueryDependencies,
    },
    WorthQueryProgramPublicationAccess,
};
use std::sync::Arc;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_program::{
    ApplicationOutputGraphShape, ApplicationProgramDefinition,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::CompositeCommitIdentity;
use {crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial, crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind, crate::domain_computation::request_execution::WorthQueryAdvancementPhase, crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation, crate::domain_computation::primary_graph::WorthQueryProductBranchAdmissionDenial};

/// Original read-source provenance and a qualified endpoint connected only by
/// canonical publications retained by the original complete tree. Shared and
/// recovered output receipts remain lawful; this does not assert exclusive work.
/// The genuine root read is retained separately from E and keeps its original
/// producer correspondence. Its seed is not relabeled as an admission at E.
/// Neither this fact nor its descriptions grant current permission or selection.
pub struct WorthQueryCompletedOriginalReadOutputs {
    source: Arc<WorthQueryApplicationReadObservation>,
    root: Arc<WorthQueryApplicationReadObservation>,
    posture: WorthQueryOutputSettlementPosture,
    output: Arc<WorthQueryApplicationReadObservation>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl WorthQueryCompletedOriginalReadOutputs {
    pub(in crate::domain_computation::primary_graph) fn original_source(
        &self,
    ) -> &Arc<WorthQueryApplicationReadObservation> {
        &self.source
    }
    pub(in crate::domain_computation::primary_graph) fn original_endpoint(
        &self,
    ) -> &Arc<WorthQueryApplicationReadObservation> {
        &self.output
    }
    pub fn source_commit(&self) -> &CompositeCommitIdentity {
        self.source.selected_commit()
    }
    pub fn root_output_commit(&self) -> &CompositeCommitIdentity {
        self.root.selected_commit()
    }
    pub fn output_commit(&self) -> &CompositeCommitIdentity {
        self.output.selected_commit()
    }
    pub fn root_posture(&self) -> WorthQueryOutputSettlementPosture {
        self.posture
    }
}

#[derive(Debug)]
pub enum WorthQueryOriginalReadOutputDenial {
    Output(WorthQueryOutputDemandDenial),
    Observation(WorthQueryProductBranchAdmissionDenial),
    Program(WorthQuerySelectedProgramOwnerDenial),
    Currentness(WorthQueryRequiredSetCurrentnessDenial),
}

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
            .checked_execution_request_for(&runtime.runtime.product_runtime)
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
        let endpoint =
            super::original_endpoint::derive(&execution, source, &mut settlements, &mut reads)
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
        Ok(WorthQueryCompletedOriginalReadOutputs {
            source: Arc::clone(source),
            root: Arc::clone(root.retained_read_ref()),
            posture: root.posture(),
            output,
            _memory: memory,
        })
    }
}
