//! Retrospective complete-tree qualification with genuine read-origin provenance.
use super::{WorthQueryOutputDemandSettlement, WorthQueryOutputSettlementPosture};
use crate::publication_boundary::{
    discovery_currentness::{
        WorthQueryRequiredSetCurrentnessDenial, WorthQueryRetainedQueryDependencies,
    },
    WorthQueryProgramPublicationAccess,
};
use std::sync::Arc;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::CompositeCommitIdentity;
use {
    crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial,
    crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind,
    crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
    crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation,
    crate::domain_computation::request_execution::WorthQueryAdvancementPhase,
};

/// The original read tree was qualified at one exact immutable output basis.
/// Source provenance does not assert that every producer executed or consumed
/// that exact source: Native may legitimately reuse retained output lineage.
/// This retains readonly history, not current permission or publication custody.
pub struct WorthQueryCompletedReadOutputs {
    source: Arc<WorthQueryApplicationReadObservation>,
    root: CompositeCommitIdentity,
    posture: WorthQueryOutputSettlementPosture,
    output: Arc<WorthQueryApplicationReadObservation>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl WorthQueryCompletedReadOutputs {
    pub fn source_commit(&self) -> &CompositeCommitIdentity {
        self.source.selected_commit()
    }
    pub fn root_output_commit(&self) -> &CompositeCommitIdentity {
        &self.root
    }
    pub fn output_commit(&self) -> &CompositeCommitIdentity {
        self.output.selected_commit()
    }
    pub fn root_posture(&self) -> WorthQueryOutputSettlementPosture {
        self.posture
    }
}

impl<S: ApplicationSchema> WorthQuerySelectedProductOperation<'_, S> {
    /// The SDK supplies its original read source, root, dependent set and
    /// discovery proof. Native checks every member at this exact selected basis.
    /// No performed source receipt or required-root fulfillment is fabricated.
    pub fn qualify_completed_read_outputs<'s>(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        scope: &WorthQueryRequestScope,
        source: &Arc<WorthQueryApplicationReadObservation>,
        root: &'s WorthQueryOutputDemandSettlement,
        dependents: impl IntoIterator<Item = &'s WorthQueryOutputDemandSettlement>,
        dependencies: Option<&WorthQueryRetainedQueryDependencies>,
    ) -> Result<WorthQueryCompletedReadOutputs, WorthQueryRequiredSetCurrentnessDenial> {
        if !source.belongs_to_selected_occurrence(self.application(), self.product().observation())
        {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSettlement,
                "completed read-output source occurrence",
            )
            .into());
        }
        let (output, memory) = self.qualify_completed_output_set(
            phase,
            access,
            scope,
            root,
            dependents,
            dependencies,
            std::mem::size_of::<WorthQueryCompletedReadOutputs>() as u64,
        )?;
        Ok(WorthQueryCompletedReadOutputs {
            source: Arc::clone(source),
            root: root.retained_read_ref().selected_commit().clone(),
            posture: root.posture(),
            output,
            _memory: memory,
        })
    }
}
