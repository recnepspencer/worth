//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryManagedGraphWorkSession {
    #[doc(hidden)]
    pub fn prepare_query_read_for_test(
        &self,
        basis: &WorthQueryApplicationBasisIdentity,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        select_indexes: impl FnOnce(
            &WorthQueryPrimaryGraphLayout,
        ) -> Vec<worth_relational::facade::indexes::DerivedIndexId>,
    ) -> Result<WorthQueryPreparedSessionRead, WorthQueryManagedGraphReadDenial> {
        Self::prepare_query_read(self, basis, execution, snapshot, select_indexes)
    }
}

impl WorthQueryManagedGraphWorkSession {
    #[doc(hidden)]
    pub fn prepare_query_read_from_pin_for_test(
        &self,
        basis: &WorthQueryApplicationBasisIdentity,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        pin: &std::sync::Arc<WorthQueryPreparedQueryPin>,
        select_indexes: impl FnOnce(
            &WorthQueryPrimaryGraphLayout,
        ) -> Vec<worth_relational::facade::indexes::DerivedIndexId>,
    ) -> Result<WorthQueryPreparedSessionRead, WorthQueryManagedGraphReadDenial> {
        Self::prepare_query_read_from_pin(self, basis, execution, snapshot, pin, select_indexes)
    }
}

impl WorthQueryPreparedSessionRead {
    #[doc(hidden)]
    pub fn execute_for_test<T>(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        read: impl FnOnce(
            &worth_relational::facade::runtime::RelationalPinnedRead,
            &WorthQueryPrimaryGraphLayout,
        ) -> T,
    ) -> Result<(T, WorthQueryPreparedReadCompletion), WorthQueryManagedGraphReadDenial> {
        Self::execute(self, execution, read)
    }
}
