//! Run the same projection reader without acquiring or releasing its owner's snapshot.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::{
    retained_decision_facts::{RetainedSourceFacts, StorageControl},
    snapshot_lease::WorthQueryApplicationSnapshotLease,
};

pub(in crate::domain_computation::primary_graph::invariant_projection) struct CompletedProjectionRead<
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

pub(in crate::domain_computation::primary_graph::invariant_projection) enum NativeReadAccess<'a> {
    Acquire,
    Borrowed(&'a mut worth_relational::facade::runtime::RelationalRuntime),
    Applied(
        &'a worth_relational::facade::runtime::RelationalPinnedRead,
        &'a worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    ),
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
    ) -> Result<CompletedProjectionRead<Output>, WorthQueryInvariantProjectionDenial> {
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
    ) -> Result<CompletedProjectionRead<Output>, WorthQueryInvariantProjectionDenial> {
        self.read_native(
            execution,
            work_budget,
            basis,
            snapshot,
            None,
            projection,
            retention_control,
        )
    }

    pub(in crate::domain_computation::primary_graph::invariant_projection) fn read_native<
        Output,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        work_budget: WorthQueryInvariantProjectionWorkBudget,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        applied: Option<(
            &worth_relational::facade::runtime::RelationalPinnedRead,
            &worth_relational::facade::runtime::RelationalPinnedRead<
                worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
            >,
        )>,
        projection: impl FnOnce(
            &mut WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
        ) -> Output,
        retention_control: StorageControl<'_, '_>,
    ) -> Result<CompletedProjectionRead<Output>, WorthQueryInvariantProjectionDenial> {
        let access = match applied {
            Some((before, candidate)) => NativeReadAccess::Applied(before, candidate),
            None => NativeReadAccess::Acquire,
        };
        self.read_native_with_access(
            execution,
            work_budget,
            basis,
            snapshot,
            access,
            projection,
            retention_control,
        )
    }
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn read_native_with_access<
        Output,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        work_budget: WorthQueryInvariantProjectionWorkBudget,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        access: NativeReadAccess<'_>,
        projection: impl FnOnce(
            &mut WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
        ) -> Output,
        retention_control: StorageControl<'_, '_>,
    ) -> Result<CompletedProjectionRead<Output>, WorthQueryInvariantProjectionDenial> {
        // The applied reader's aggregate cache is private to this image. Its
        // backing must not enter the committed version cache before publication.
        let _applied_cache_memory = if matches!(&access, NativeReadAccess::Applied(..)) {
            let bytes = (std::mem::size_of::<
                std::sync::Mutex<
                    super::super::super::aggregate_projection::WorthQueryAggregateProjections,
                >,
            >() + 2 * std::mem::size_of::<usize>()) as u64;
            Some(
                retention_control
                    .policy()
                    .reserve_memory(bytes)
                    .map_err(|cause| {
                        WorthQueryInvariantProjectionDenial::source_retention_denied(
                            cause.into(),
                            WorthQueryInvariantProjectionWork::default(),
                        )
                    })?,
            )
        } else {
            None
        };
        let decode_work = std::cell::Cell::new(None);
        let decode =
            super::super::super::value_decode::ValueDecodeState::new(self.value_bindings.clone());
        let decoded = decode.run(execution, |value_decode| match access {
            NativeReadAccess::Acquire => self.graph.with_runtime_mut(execution, |runtime| {
                catch_unwind(AssertUnwindSafe(|| {
                    self.execute_reader(
                        execution,
                        NativeProjectionRead::Committed(runtime),
                        value_decode,
                        basis,
                        snapshot,
                        work_budget,
                        projection,
                        retention_control,
                        &decode_work,
                    )
                }))
            }),
            NativeReadAccess::Borrowed(native) => catch_unwind(AssertUnwindSafe(|| {
                self.execute_reader(
                    execution,
                    NativeProjectionRead::Committed(native),
                    value_decode,
                    basis,
                    snapshot,
                    work_budget,
                    projection,
                    retention_control,
                    &decode_work,
                )
            })),
            NativeReadAccess::Applied(before, candidate) => catch_unwind(AssertUnwindSafe(|| {
                self.execute_reader(
                    execution,
                    NativeProjectionRead::Applied { before, candidate },
                    value_decode,
                    basis,
                    snapshot,
                    work_budget,
                    projection,
                    retention_control,
                    &decode_work,
                )
            })),
        });
        let projected = decoded.map_err(|cause| {
            WorthQueryInvariantProjectionDenial::value_decode_stopped(cause, decode_work.get())
        })?;
        let super::reader_execution::ProjectionExecution {
            output,
            work,
            realized_scope,
            consumed_outputs,
            source_facts,
            exceeded,
            overflowed,
            exhausted,
            stopped,
        } = match projected {
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
        Ok(CompletedProjectionRead {
            output,
            work,
            realized_scope,
            consumed_outputs,
            dependent_source_facts,
        })
    }
}
