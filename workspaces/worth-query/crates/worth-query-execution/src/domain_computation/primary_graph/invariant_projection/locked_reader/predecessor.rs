//! An old Native basis is read through the existing loan, without another graph lock.
use super::borrowed_snapshot::{CompletedProjectionRead, NativeReadAccess};
use super::*;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl;
impl<S: ApplicationSchema> WorthQueryApplicationInvariantProjectionAuthority<S> {
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn project_borrowed_basis<
        T,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        native: &mut worth_relational::facade::runtime::RelationalRuntime,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        maximum_work: Option<usize>,
        projection: impl FnOnce(&mut WorthQueryApplicationInvariantProjectionReader<'_, S>) -> T,
        retention: StorageControl<'_, '_>,
    ) -> Result<CompletedProjectionRead<T>, WorthQueryInvariantProjectionDenial> {
        let snapshot = native
            .snapshots()
            .snapshot_for_observation(&basis.observation())
            .map_err(super::super::admission_denial::from_snapshot_admission_denial)?;
        let result = catch_unwind(AssertUnwindSafe(|| {
            self.read_native_with_access(
                execution,
                maximum_work.map_or_else(
                    WorthQueryInvariantProjectionWorkBudget::unbounded,
                    WorthQueryInvariantProjectionWorkBudget::bounded,
                ),
                basis,
                &snapshot,
                NativeReadAccess::Borrowed(native),
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
