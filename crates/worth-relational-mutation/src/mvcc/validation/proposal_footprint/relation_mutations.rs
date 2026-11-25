//! Projected relation changes and their funded correspondence.
use super::RecordedMemberRestorationRefusal;
use crate::contracts::identity::data::primitives::KindId;
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use crate::contracts::transactions::data::outcomes::transaction_commit_error::{
    CommitExecutionDenial, CommitExecutionDenialKind, TransactionCommitError,
    TransactionCommitErrorOwnerPort as _,
};
use worth_execution::ExecutionMemoryReservation;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidatedRelationMutation {
    pub(super) relation: RelationId,
    pub(super) kind: KindId,
    pub(super) audit_retained: bool,
    pub(super) before: Option<(EntityId, EntityId)>,
    pub(super) after: Option<(EntityId, EntityId)>,
    pub(super) allocated_endpoint: bool,
}
impl ValidatedRelationMutation {
    pub const fn relation(&self) -> RelationId {
        self.relation
    }
    pub const fn kind(&self) -> KindId {
        self.kind
    }
    pub const fn is_audit_retained(&self) -> bool {
        self.audit_retained
    }
    pub const fn before(&self) -> Option<(EntityId, EntityId)> {
        self.before
    }
    pub const fn after(&self) -> Option<(EntityId, EntityId)> {
        self.after
    }
    /// A born relation with an endpoint this same draft allocated. Its prior
    /// absence is structural: no earlier relation can name an entity that did
    /// not exist, so no read is needed to know it had no prior endpoints.
    pub const fn has_allocated_endpoint(&self) -> bool {
        self.allocated_endpoint
    }
}
/// Privately constructed from one invariant-approved draft; selectors grant no effects.
#[derive(Debug)]
pub struct ValidatedRelationMutationFootprint {
    pub(super) relations: Vec<ValidatedRelationMutation>,
    pub(super) restored_entities:
        Result<Vec<(EntityId, EntityId)>, RecordedMemberRestorationRefusal>,
    pub(super) _memory: Option<ExecutionMemoryReservation>,
}
impl ValidatedRelationMutationFootprint {
    pub fn restored_entities(
        &self,
    ) -> Result<&[(EntityId, EntityId)], RecordedMemberRestorationRefusal> {
        self.restored_entities
            .as_deref()
            .map_err(|refusal| *refusal)
    }
    pub fn relations(&self) -> &[ValidatedRelationMutation] {
        &self.relations
    }
}
pub(super) fn stop(
    cause: impl Into<crate::contracts::execution::denial_cause::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(CommitExecutionDenial {
        kind: CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}
