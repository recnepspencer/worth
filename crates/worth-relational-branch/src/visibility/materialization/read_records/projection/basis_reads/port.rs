//! Cross-crate access to the exact basis reads.
use super::{EntityReadRecord, KindId, PartitionId, RelationReadRecord, VisibilityProjectionView};

/// Receiver-scoped fixtures for the Relational unit courts.
#[doc(hidden)]
pub trait VisibilityProjectionViewFixturePort: Sized {
    fn all_authoritative_entity_records_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::denial_cause::RelationalExecutionDenialCause>;
    fn all_authoritative_relation_records_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        Vec<RelationReadRecord>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    >;
}

impl VisibilityProjectionViewFixturePort for VisibilityProjectionView {
    fn all_authoritative_entity_records_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::denial_cause::RelationalExecutionDenialCause>
    {
        self.all_authoritative_entity_records(execution)
    }

    fn all_authoritative_relation_records_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        Vec<RelationReadRecord>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        self.all_authoritative_relation_records(execution)
    }
}

impl VisibilityProjectionView {
    #[doc(hidden)]
    pub(crate) fn authoritative_entity_records_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        kind_id: KindId,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::denial_cause::RelationalExecutionDenialCause>
    {
        self.authoritative_entity_records(execution, kind_id)
    }
}

impl VisibilityProjectionView {
    #[doc(hidden)]
    pub(crate) fn authoritative_entity_records_in_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition_id: PartitionId,
        kind_id: KindId,
    ) -> Result<Vec<EntityReadRecord>, crate::execution::denial_cause::RelationalExecutionDenialCause>
    {
        self.authoritative_entity_records_in(execution, partition_id, kind_id)
    }
}

impl VisibilityProjectionView {
    #[doc(hidden)]
    pub(crate) fn authoritative_relation_records_in_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        partition_id: PartitionId,
        kind_id: KindId,
    ) -> Result<
        Vec<RelationReadRecord>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        self.authoritative_relation_records_in(execution, partition_id, kind_id)
    }
}
