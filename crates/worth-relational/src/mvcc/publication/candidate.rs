use std::cell::Cell;
use std::marker::PhantomData;

use crate::history::data::BranchId;
use crate::transactions::data::TransactionId;

/// Opaque, single-use owner artifact whose fallible commit preparation is complete.
///
/// A candidate has no publication method. Only the Relational runtime owner can
/// consume it, either through publication or explicit discard.
pub struct PreparedRelationalCommitCandidate {
    runtime_instance_id: u64,
    custom_invariant_generation: u64,
    publication_binding: crate::runtime::RelationalRuntimePublicationBinding,
    transaction_id: TransactionId,
    branch_id: BranchId,
    candidate_id: u64,
    _payload: std::sync::Arc<std::sync::Mutex<Option<CandidatePayload>>>,
    expires_at: std::time::Instant,
    pub(crate) maximum_lifetime_millis: u64,
    reservation_count: usize,
    consumed: bool,
    _not_sync: PhantomData<Cell<()>>,
}

pub(crate) struct CandidatePayload {
    expected_basis: crate::branch::RelationalBranchBasisDescriptor,
    expected_root: std::sync::Arc<crate::branch::RelationalBranchRoot>,
    publication_cell: crate::branch::RelationalBranchPublicationCell,
    execution: crate::authority::commit::pipeline::PreparedCommitPublicationExecution,
    retention: crate::history::retention::RelationalCandidateRetentionObligation,
    published_snapshot_slot: crate::runtime::PublishedSnapshotSlotReservation,
    control: crate::mvcc::RelationalOperationControl,
}

pub(super) struct PreparedRelationalPublicationParts {
    pub(super) runtime_instance_id: u64,
    pub(super) publication_binding: crate::runtime::RelationalRuntimePublicationBinding,
    pub(super) expected: crate::branch::RelationalBranchBasisDescriptor,
    pub(super) expected_root: std::sync::Arc<crate::branch::RelationalBranchRoot>,
    pub(super) publication_cell: crate::branch::RelationalBranchPublicationCell,
    pub(super) movement: super::PreparedCanonicalBranchMovement,
    pub(super) completion: crate::authority::commit::pipeline::PreparedCommitPublicationCompletion,
    pub(super) candidate_retention:
        crate::history::retention::RelationalCandidateRetentionObligation,
    pub(super) control: crate::mvcc::RelationalOperationControl,
    pub(super) expires_at: std::time::Instant,
    pub(super) maximum_lifetime_millis: u64,
}

pub(crate) enum PreparedRelationalCandidateAdmissionStop {
    Deferred(super::RelationalPublicationDeferred),
    Failed(super::RelationalPublicationFailure),
}

impl PreparedRelationalCommitCandidate {
    pub(crate) fn new(
        runtime_instance_id: u64,
        custom_invariant_generation: u64,
        publication_binding: crate::runtime::RelationalRuntimePublicationBinding,
        transaction_id: TransactionId,
        branch_id: BranchId,
        expected_basis: crate::branch::RelationalBranchBasisDescriptor,
        expected_root: std::sync::Arc<crate::branch::RelationalBranchRoot>,
        publication_cell: crate::branch::RelationalBranchPublicationCell,
        execution: crate::authority::commit::pipeline::PreparedCommitPublicationExecution,
        retention: crate::history::retention::RelationalCandidateRetentionObligation,
        published_snapshot_slot: crate::runtime::PublishedSnapshotSlotReservation,
        control: crate::mvcc::RelationalOperationControl,
        maximum_lifetime_millis: u64,
        maximum_candidates: usize,
    ) -> Result<Self, PreparedRelationalCandidateAdmissionStop> {
        let expires_at = std::time::Instant::now()
            .checked_add(std::time::Duration::from_millis(maximum_lifetime_millis))
            .unwrap_or_else(std::time::Instant::now);
        let reservation_count = execution.reservation_count();
        let payload = std::sync::Arc::new(std::sync::Mutex::new(Some(CandidatePayload {
            expected_basis,
            expected_root,
            publication_cell,
            execution,
            retention,
            published_snapshot_slot,
            control,
        })));
        let candidate_id = publication_binding
            .register_candidate(expires_at, maximum_candidates, &payload)
            .map_err(|denial| match denial {
                crate::runtime::RelationalCandidateRegistrationDenial::CapacityExhausted => {
                    PreparedRelationalCandidateAdmissionStop::Deferred(
                        super::RelationalPublicationDeferred::CandidateCapacityExhausted {
                            maximum_candidates,
                        },
                    )
                }
                crate::runtime::RelationalCandidateRegistrationDenial::IdentityExhausted => {
                    PreparedRelationalCandidateAdmissionStop::Failed(
                        super::RelationalPublicationFailure::new(
                            super::RelationalPublicationFailureKind::CandidateIdentityExhausted,
                            "prepared candidate identity space is exhausted",
                        ),
                    )
                }
            })?;
        Ok(Self {
            runtime_instance_id,
            custom_invariant_generation,
            publication_binding,
            transaction_id,
            branch_id,
            candidate_id,
            _payload: payload,
            expires_at,
            maximum_lifetime_millis,
            reservation_count,
            consumed: false,
            _not_sync: PhantomData,
        })
    }

    pub fn transaction_id(&self) -> TransactionId {
        self.transaction_id
    }

    pub fn branch(&self) -> &BranchId {
        &self.branch_id
    }

    pub(crate) fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    pub(crate) fn custom_invariant_generation(&self) -> u64 {
        self.custom_invariant_generation
    }

    pub(crate) fn belongs_to_publication_owner(
        &self,
        binding: &crate::runtime::RelationalRuntimePublicationBinding,
    ) -> bool {
        self.publication_binding.belongs_to_same_owner(binding)
    }

    pub(crate) fn reservation_count(&self) -> usize {
        self.reservation_count
    }

    pub(crate) fn lifetime_expired(&self) -> bool {
        std::time::Instant::now() >= self.expires_at
    }

    pub(super) fn into_publication_parts(
        mut self,
    ) -> Result<PreparedRelationalPublicationParts, super::RelationalPublicationDeferred> {
        let payload = self
            .publication_binding
            .take_candidate(self.candidate_id)
            .ok_or(
                super::RelationalPublicationDeferred::CandidateLifetimeExpired {
                    maximum_lifetime_millis: self.maximum_lifetime_millis,
                },
            )?;
        self.consumed = true;
        let (movement, completion) = payload.execution.split(payload.published_snapshot_slot);
        Ok(PreparedRelationalPublicationParts {
            runtime_instance_id: self.runtime_instance_id,
            publication_binding: self.publication_binding.clone(),
            expected: payload.expected_basis,
            expected_root: payload.expected_root,
            publication_cell: payload.publication_cell,
            movement,
            completion,
            candidate_retention: payload.retention,
            control: payload.control,
            expires_at: self.expires_at,
            maximum_lifetime_millis: self.maximum_lifetime_millis,
        })
    }

    #[cfg(test)]
    pub(crate) fn materialization_counts(&self) -> (u64, u64) {
        self._payload
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .map_or((0, 0), |payload| payload.execution.materialization_counts())
    }

    #[cfg(test)]
    pub(crate) fn publication_cell_for_test(
        &self,
    ) -> crate::branch::RelationalBranchPublicationCell {
        self._payload
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .expect("live candidate retains its publication payload")
            .publication_cell
            .clone()
    }

    #[cfg(test)]
    pub(crate) fn expected_root_for_test(
        &self,
    ) -> std::sync::Arc<crate::branch::RelationalBranchRoot> {
        std::sync::Arc::clone(
            &self
                ._payload
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .as_ref()
                .expect("live candidate retains its publication payload")
                .expected_root,
        )
    }
}

impl Drop for PreparedRelationalCommitCandidate {
    fn drop(&mut self) {
        if !self.consumed {
            self.publication_binding
                .discard_candidate(self.candidate_id);
        }
    }
}

impl std::fmt::Debug for PreparedRelationalCommitCandidate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PreparedRelationalCommitCandidate")
            .field("transaction_id", &self.transaction_id)
            .field("branch_id", &self.branch_id)
            .field("reservation_count", &self.reservation_count())
            .finish_non_exhaustive()
    }
}

/// Evidence returned when a prepared candidate is consumed without publication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscardedRelationalCommitCandidate {
    transaction_id: TransactionId,
    branch_id: BranchId,
    released_record_reservation_count: usize,
}

impl DiscardedRelationalCommitCandidate {
    pub(crate) fn from_candidate(candidate: &PreparedRelationalCommitCandidate) -> Self {
        Self {
            transaction_id: candidate.transaction_id,
            branch_id: candidate.branch_id.clone(),
            released_record_reservation_count: candidate.reservation_count(),
        }
    }

    pub fn transaction_id(&self) -> TransactionId {
        self.transaction_id
    }

    pub fn branch(&self) -> &BranchId {
        &self.branch_id
    }

    pub fn released_record_reservation_count(&self) -> usize {
        self.released_record_reservation_count
    }
}
