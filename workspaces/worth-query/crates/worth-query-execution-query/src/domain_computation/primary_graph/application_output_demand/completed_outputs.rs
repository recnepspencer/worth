//! One genuine retained basis qualified against the completed original output set.
use super::{
    WorthQueryFulfilledRequiredRoot, WorthQueryOutputDemandSettlement,
    WorthQueryOutputSettlementPosture,
};
use crate::discovery_boundary::discovery_currentness::{
    WorthQueryRequiredSetCurrentnessDenial, WorthQueryRetainedQueryDependencies,
};
use crate::domain_computation::primary_graph::ReadObservationRuntimeExt;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationReadObservation, WorthQuerySelectedProductOperation,
};
use std::sync::Arc;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccess;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::CompositeCommitIdentity;

/// Retrospective complete-output qualification at one immutable source. This
/// retains no current permission and has no query or mutation conversion.
pub struct WorthQueryCompletedRequiredOutputs {
    source: CompositeCommitIdentity,
    intent: WorthQueryApplicationIdempotencyBinding,
    root: CompositeCommitIdentity,
    posture: WorthQueryOutputSettlementPosture,
    output: Arc<WorthQueryApplicationReadObservation>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl WorthQueryCompletedRequiredOutputs {
    pub fn source_commit(&self) -> &CompositeCommitIdentity {
        &self.source
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
    pub fn matches_source(&self, source: &WorthQueryApplicationCommitReceipt) -> bool {
        source.idempotency_binding() == self.intent
            && source.committed_product_publication().composite_commit() == &self.source
    }
    pub(crate) fn retained_output(&self) -> &Arc<WorthQueryApplicationReadObservation> {
        &self.output
    }
}

/// Crate-boundary internal port for `WorthQueryCompletedRequiredOutputs`.
/// Never exported on `worth-query-execution::facade` or Host.
pub trait CompletedRequiredOutputsInternalPort {
    fn retained_output(&self) -> &Arc<WorthQueryApplicationReadObservation>;
}

impl CompletedRequiredOutputsInternalPort for WorthQueryCompletedRequiredOutputs {
    fn retained_output(&self) -> &Arc<WorthQueryApplicationReadObservation> {
        Self::retained_output(self)
    }
}

impl<S: ApplicationSchema> WorthQuerySelectedProductOperation<'_, S> {
    /// The proposed basis is not evidence that the tree is complete. The SDK
    /// supplies the original completed root, dependent set and discovery proof;
    /// Native validates every member at this exact basis before sealing it.
    pub fn qualify_completed_required_outputs<'s>(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        scope: &WorthQueryRequestScope,
        source: &WorthQueryApplicationCommitReceipt,
        fulfilled: &WorthQueryFulfilledRequiredRoot,
        root: &'s WorthQueryOutputDemandSettlement,
        dependents: impl IntoIterator<Item = &'s WorthQueryOutputDemandSettlement>,
        dependencies: Option<&WorthQueryRetainedQueryDependencies>,
    ) -> Result<WorthQueryCompletedRequiredOutputs, WorthQueryRequiredSetCurrentnessDenial> {
        use crate::domain_computation::primary_graph::{
            WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind as Kind,
        };
        let refused = |kind| {
            WorthQueryOutputDemandDenial::new(kind, "completed required-output qualification")
        };
        if !fulfilled.matches_source(source)
            || !fulfilled
                .retained_output()
                .belongs_to_selected_occurrence(self.application(), self.product().observation())
            || !Arc::ptr_eq(fulfilled.retained_output(), root.retained_read_ref())
            || fulfilled.posture() != root.posture()
        {
            return Err(refused(Kind::ForeignSettlement).into());
        }
        let (output, memory) = self.qualify_completed_output_set(
            phase,
            access,
            scope,
            root,
            dependents,
            dependencies,
            std::mem::size_of::<WorthQueryCompletedRequiredOutputs>() as u64,
        )?;
        Ok(WorthQueryCompletedRequiredOutputs {
            source: fulfilled.source_commit().clone(),
            intent: source.idempotency_binding(),
            root: fulfilled.output_commit().clone(),
            posture: fulfilled.posture(),
            output,
            _memory: memory,
        })
    }
}
