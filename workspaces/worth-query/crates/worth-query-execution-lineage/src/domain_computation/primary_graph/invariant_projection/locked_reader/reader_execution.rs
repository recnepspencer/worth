//! The one reader execution and fact handoff for both Native phases.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::{
    RetainedSourceFacts, StorageControl,
};
pub struct ProjectionExecution<Output> {
    pub output: Output,
    pub work: WorthQueryInvariantProjectionWork,
    pub realized_scope: WorthQueryRealizedProjectionScope,
    pub applied_consumptions: Vec<crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence>,
    pub consumed_outputs: BTreeMap<Arc<super::super::super::output_lineage::RecordedSettlementIdentity>,super::super::ConsumedOutputEvidence>,
    pub source_facts: Result<RetainedSourceFacts, crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial>,
    pub exceeded: bool,
    pub overflowed: bool,
    pub exhausted: bool,
    pub stopped: Option<worth_relational::facade::transactions::RelationalExecutionDenialCause>,
}
impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionAuthority<Schema> {
    pub(crate) fn execute_reader<Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        native: NativeProjectionRead<'_>,
        value_decode: &super::super::super::value_decode::ValueDecodeSession<'_>,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        work_budget: WorthQueryInvariantProjectionWorkBudget,
        projection: impl FnOnce(
            &mut WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
        ) -> Output,
        retention_control: StorageControl<'_, '_>,
        decode_work: &std::cell::Cell<Option<WorthQueryInvariantProjectionWork>>,
    ) -> ProjectionExecution<Output> {
        let applied = native.candidate_identity().is_some();
        let mut reader = WorthQueryApplicationInvariantProjectionReader {
                    value_bindings: &self.value_bindings,
                    value_decode,
                    execution,
                    native,
                    layout: &self.layout,
                    snapshot,
                    observation: basis.observation(),
                    entity_resolution: &self.entity_resolution,
                    authority_identity: self.authority_identity,
                    binding_identity: &self.binding_identity,
                    program_activation: self.program_activation.as_ref(),
                    work: WorthQueryInvariantProjectionWork::default(),
                    work_budget,
                    retention_exhausted: false,
                    native_read_stop: None,
                    realized_scope: WorthQueryRealizedProjectionScope::default(),
                    aggregate_projections: if applied { Arc::new(std::sync::Mutex::new(Default::default())) } else { Arc::clone(&self.graph.aggregate_projections) },
                    output_lineage: Arc::clone(&self.graph.output_lineage),
                    invalidation_owner: Arc::clone(&self.graph.source_owner.invalidation_owner),
                    managed_views: self.graph.managed_derived_views(),
                    selected_product_occurrence: None,
                    selected_product_generation: None,
                    selected_source_partition_identity: None,
                    prior_output_bindings: HashMap::new(),
                    current_output_families: HashMap::new(),
                    dependent_source_facts: super::super::super::application_attempt::retained_decision_facts::AuthoringSourceFacts::vacant(),
                    retention_control,
                    consumed_outputs: BTreeMap::new(),
                    applied_consumptions: Vec::new(),
                    _schema: PhantomData,
                };
        let output = projection(&mut reader);
        decode_work.set(Some(reader.work));
        ProjectionExecution {
            output,
            work: reader.work,
            realized_scope: reader.realized_scope,
            consumed_outputs: reader.consumed_outputs,
            applied_consumptions: reader.applied_consumptions,
            source_facts: reader.dependent_source_facts.finish(retention_control),
            exceeded: reader.work_budget.exceeded(),
            overflowed: reader.work_budget.overflowed(),
            exhausted: reader.retention_exhausted,
            stopped: reader.native_read_stop,
        }
    }
}
