//! An old Native basis is read through the existing loan, without another graph lock.
use super::borrowed_snapshot::{CompletedProjectionRead, NativeReadAccess};
use super::*;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl;
impl<S: ApplicationSchema> WorthQueryApplicationInvariantProjectionAuthority<S> {
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn project_borrowed_input<
        T,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
        input: &worth_relational::facade::mvcc::RelationalPredecessorInput,
        maximum_work: Option<usize>,
        projection: impl FnOnce(&mut WorthQueryApplicationInvariantProjectionReader<'_, S>) -> T,
        retention: StorageControl<'_, '_>,
    ) -> Result<CompletedProjectionRead<T>, WorthQueryInvariantProjectionDenial> {
        let basis = input.before_basis();
        let snapshot = native
            .snapshots()
            .snapshot_for_observation(&basis.observation())
            .map_err(super::super::admission_denial::from_snapshot_admission_denial)?;
        let result = catch_unwind(AssertUnwindSafe(|| {
            let before = input.applied_read().map(|_| {
                let indexes = self.layout.equality_index_ids().collect::<Vec<_>>();
                native.read_truth().pin_read_capability(&snapshot, &indexes, execution)
                    .map_err(|cause| match cause {
                        worth_relational::facade::runtime::RelationalPinnedReadAdmissionDenial::Request(stop) =>
                            WorthQueryInvariantProjectionDenial::execution_request_stopped(stop, None),
                        _ => WorthQueryInvariantProjectionDenial::from_kind(super::super::WorthQueryInvariantProjectionDenialKind::BasisUnavailable),
                    })
            }).transpose()?;
            let access = match (before.as_ref(), input.applied_read()) {
                (Some(before), Some(candidate)) => NativeReadAccess::Applied(before, candidate),
                _ => NativeReadAccess::Borrowed(native),
            };
            self.read_native_with_access(
                execution,
                maximum_work.map_or_else(
                    WorthQueryInvariantProjectionWorkBudget::unbounded,
                    WorthQueryInvariantProjectionWorkBudget::bounded,
                ),
                basis,
                &snapshot,
                access,
                projection,
                retention,
            )
        }));
        crate::relational_snapshot_release::release_query_snapshot(native, &snapshot);
        match result {
            Ok(result) => result,
            Err(payload) => resume_unwind(payload),
        }
    }
}
