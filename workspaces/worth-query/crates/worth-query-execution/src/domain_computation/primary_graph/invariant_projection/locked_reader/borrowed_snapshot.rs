//! Run the same projection reader without acquiring or releasing its owner's snapshot.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::{
    retained_decision_facts::{RetainedSourceFacts, StorageControl},
    snapshot_lease::WorthQueryApplicationSnapshotLease,
};

pub(in crate::domain_computation::primary_graph::invariant_projection) struct CompletedSnapshotRead<
    Output,
> {
    pub(in crate::domain_computation::primary_graph::invariant_projection) output: Output,
    pub(in crate::domain_computation::primary_graph::invariant_projection) work:
        WorthQueryInvariantProjectionWork,
    pub(in crate::domain_computation::primary_graph::invariant_projection) realized_scope:
        WorthQueryRealizedProjectionScope,
    pub(in crate::domain_computation::primary_graph::invariant_projection) consumed_outputs:
        BTreeMap<
            Arc<super::super::super::output_lineage::RecordedSettlementIdentity>,
            super::super::ConsumedOutputEvidence,
        >,
    pub(in crate::domain_computation::primary_graph::invariant_projection) dependent_source_facts:
        RetainedSourceFacts,
}

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionAuthority<Schema> {
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn project_borrowed<
        Output,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        lease: &WorthQueryApplicationSnapshotLease,
        maximum_work: Option<usize>,
        projection: impl FnOnce(
            &mut WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
        ) -> Output,
        retention_control: StorageControl<'_, '_>,
    ) -> Result<CompletedSnapshotRead<Output>, WorthQueryInvariantProjectionDenial> {
        self.read_snapshot(
            execution,
            maximum_work.map_or_else(
                WorthQueryInvariantProjectionWorkBudget::unbounded,
                WorthQueryInvariantProjectionWorkBudget::bounded,
            ),
            lease.product().relational_basis(),
            lease.snapshot(),
            projection,
            retention_control,
        )
    }

    pub(super) fn read_snapshot<Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        work_budget: WorthQueryInvariantProjectionWorkBudget,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        projection: impl FnOnce(
            &mut WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
        ) -> Output,
        retention_control: StorageControl<'_, '_>,
    ) -> Result<CompletedSnapshotRead<Output>, WorthQueryInvariantProjectionDenial> {
        let decode_work = std::cell::Cell::new(None);
        let decode =
            super::super::super::value_decode::ValueDecodeState::new(self.value_bindings.clone());
        let decoded = decode.run(execution, |value_decode| {
            self.graph.with_runtime_mut(execution, |runtime| catch_unwind(AssertUnwindSafe(|| {
                let mut reader = WorthQueryApplicationInvariantProjectionReader {
                    value_bindings: &self.value_bindings,
                    value_decode,
                    execution,
                    runtime,
                    layout: &self.layout,
                    snapshot,
                    observation: basis.observation(),
                    entity_resolution: &self.entity_resolution,
                    authority_identity: self.authority_identity,
                    program_activation: self.program_activation.as_ref(),
                    work: WorthQueryInvariantProjectionWork::default(),
                    work_budget,
                    retention_exhausted: false,
                    native_read_stop: None,
                    realized_scope: WorthQueryRealizedProjectionScope::default(),
                    aggregate_projections: Arc::clone(&self.graph.aggregate_projections),
                    output_lineage: Arc::clone(&self.graph.output_lineage),
                    invalidation_owner: Arc::clone(&self.graph.source_owner.invalidation_owner),
                    selected_product_occurrence: None,
                    selected_product_generation: None,
                    selected_source_partition_identity: None,
                    prior_output_bindings: HashMap::new(),
                    current_output_families: HashMap::new(),
                    dependent_source_facts: super::super::super::application_attempt::retained_decision_facts::AuthoringSourceFacts::vacant(),
                    retention_control,
                    consumed_outputs: BTreeMap::new(),
                    _schema: PhantomData,
                };
                let output = projection(&mut reader);
                decode_work.set(Some(reader.work));
                (
                    output,
                    reader.work,
                    reader.realized_scope,
                    reader.consumed_outputs,
                    reader.dependent_source_facts.finish(retention_control),
                    (reader.work_budget.exceeded(), reader.work_budget.overflowed()),
                    reader.retention_exhausted,
                    reader.native_read_stop,
                )
            })))
        });
        let projected = decoded.map_err(|cause| {
            WorthQueryInvariantProjectionDenial::value_decode_stopped(cause, decode_work.get())
        })?;
        let (output, work, realized_scope, consumed_outputs, source_facts, (exceeded, overflowed), exhausted, stopped) =
            match projected {
                Ok(projected) => projected,
                // Unwind only after the native runtime loan ends. The owning
                // entrance releases its snapshot; the borrowed entrance never does.
                Err(payload) => resume_unwind(payload),
            };
        let denial = if let Some(cause) = stopped {
            Some(WorthQueryInvariantProjectionDenial::execution_request_stopped(cause, Some(work)))
        } else if overflowed {
            Some(WorthQueryInvariantProjectionDenial::execution_request_stopped(
                worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkCounterOverflow,
                Some(work),
            ))
        } else if exceeded {
            Some(WorthQueryInvariantProjectionDenial::work_budget_exceeded(
                work,
            ))
        } else {
            exhausted.then(|| {
                WorthQueryInvariantProjectionDenial::from_kind(
                super::super::WorthQueryInvariantProjectionDenialKind::RetentionCapacityExhausted,
            )
            })
        };
        if let Some(denial) = denial {
            return Err(match source_facts {
                Ok(_) => denial,
                Err(retention) => denial.with_retention_denial(retention),
            });
        }
        let dependent_source_facts = source_facts.map_err(|denial| {
            WorthQueryInvariantProjectionDenial::source_retention_denied(denial, work)
        })?;
        decode.check(execution).map_err(|cause| {
            WorthQueryInvariantProjectionDenial::value_decode_handoff_stopped(cause, Some(work))
        })?;
        Ok(CompletedSnapshotRead {
            output,
            work,
            realized_scope,
            consumed_outputs,
            dependent_source_facts,
        })
    }
}
