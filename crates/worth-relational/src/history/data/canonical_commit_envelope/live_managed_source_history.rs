//! Disposable live metadata; never canonical serialized truth or Source authority.
use super::{CanonicalCommitAuthorityKind, CanonicalCommitEnvelope};
use crate::validation::candidate_source::authored_history::PreparedMembershipHistory;
use std::sync::{Arc, Weak};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

#[derive(Default)]
pub(crate) struct LiveManagedSourceHistory(Option<Arc<BoundHistory>>);
struct BoundHistory {
    prepared: Arc<PreparedMembershipHistory>,
    owner: Weak<CanonicalCommitEnvelope>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl Clone for LiveManagedSourceHistory {
    fn clone(&self) -> Self {
        // Only Arc clones of the actual frozen envelope retain its live data.
        // Deep canonical-value cloning must not copy issuance or Weak custody.
        Self::default()
    }
}
impl std::fmt::Debug for LiveManagedSourceHistory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("LiveManagedSourceHistory")
            .field(&self.0.is_some())
            .finish()
    }
}
impl PartialEq for LiveManagedSourceHistory {
    fn eq(&self, _: &Self) -> bool {
        // Canonical value equality deliberately excludes live acceleration.
        true
    }
}
impl Eq for LiveManagedSourceHistory {}
impl LiveManagedSourceHistory {
    pub(crate) fn verified<'a>(
        envelope: &'a Arc<CanonicalCommitEnvelope>,
    ) -> Option<&'a PreparedMembershipHistory> {
        let live = envelope.live_managed_source_history.0.as_ref()?;
        let owner = live.owner.upgrade()?;
        if !Arc::ptr_eq(&owner, envelope) {
            return None;
        }
        Some(&live.prepared)
    }
}
impl CanonicalCommitEnvelope {
    /// Only actual performed publication supplies this prepared Source packet.
    /// The final immutable envelope binds its entire receipt/schema/plan at once.
    pub(crate) fn freeze_with_managed_source_history(
        mut self,
        prepared: Option<Arc<PreparedMembershipHistory>>,
        execution: ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, crate::transactions::data::TransactionCommitError> {
        let prepared = prepared.filter(|packet| packet_matches(packet, &self));
        let Some(prepared) = prepared else {
            self.live_managed_source_history = LiveManagedSourceHistory::default();
            return Ok(Arc::new(self));
        };
        let layout = std::alloc::Layout::new::<BoundHistory>()
            .extend(std::alloc::Layout::new::<[usize; 2]>())
            .map_err(|_| {
                storage(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
            })?
            .0
            .pad_to_align();
        let memory = execution
            .reserve_retained_memory(layout.size() as u64)
            .map_err(|cause| storage(cause.into()))?;
        execution.consult().map_err(|cause| storage(cause.into()))?;
        Ok(Arc::new_cyclic(|owner| {
            self.live_managed_source_history =
                LiveManagedSourceHistory(Some(Arc::new(BoundHistory {
                    prepared,
                    owner: owner.clone(),
                    _memory: memory,
                })));
            self
        }))
    }
}
fn packet_matches(packet: &PreparedMembershipHistory, envelope: &CanonicalCommitEnvelope) -> bool {
    let parent_matches = match packet.parent {
        None => envelope.commit.parents.is_empty(),
        Some(parent) => envelope.commit.parents.as_slice() == [parent],
    };
    parent_matches
        && packet.branch == envelope.branch_context
        && packet.branch == envelope.commit.branch_id
        && packet.proposed == envelope.commit.version_id
        && envelope.authority_kind == CanonicalCommitAuthorityKind::VersionedTransaction
        && envelope.schema_transition.is_none()
        && envelope.merge_execution_authority.is_none()
        && envelope.merge_parent_branches.is_empty()
        && envelope.merge_base_commits.is_empty()
        && envelope.required_output_materialization.is_none()
        && envelope.required_output_codec_retirements.is_empty()
}
fn storage(
    cause: crate::execution::RelationalExecutionDenialCause,
) -> crate::transactions::data::TransactionCommitError {
    use crate::transactions::data::{
        CommitExecutionDenial, CommitExecutionDenialKind, TransactionCommitError,
    };
    TransactionCommitError::execution(CommitExecutionDenial {
        kind: CommitExecutionDenialKind::Cause(cause),
        partition_identity: None,
    })
}
