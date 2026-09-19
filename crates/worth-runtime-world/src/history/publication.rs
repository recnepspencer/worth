//! Canonical facts of a performed publication. The history entry owns this
//! allocation; a caller's linear delivery claim never becomes its truth source.

mod delivery;

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use crate::branch::{ProductBranchReferenceMovement, ProductBranchReferenceSnapshot};
use crate::identity::{CompositeCommitIdentity, CompositePublicationAttemptIdentity};
use crate::publication::{
    CompositeLateCancellationPosture, CompositeOwnerExecutionResults,
    CompositePublicationCostCounters,
};

pub(crate) use delivery::PublicationDeliveryClaim;

/// Preallocated before owner effects and charged with its exact history slot.
/// Facts become visible only at the successful branch-cell replacement.
#[derive(Debug)]
pub(crate) struct CanonicalPublicationEnvelope {
    commit_identity: CompositeCommitIdentity,
    attempt_identity: CompositePublicationAttemptIdentity,
    expected: ProductBranchReferenceSnapshot,
    facts: OnceLock<PerformedPublicationFacts>,
    committed: AtomicBool,
    delivery: AtomicU8,
    successor_observation: Mutex<Option<crate::branch::ProductBranchObservation>>,
}

#[derive(Debug)]
pub(crate) struct PerformedPublicationFacts {
    pub(crate) movement: ProductBranchReferenceMovement,
    pub(crate) component_results: CompositeOwnerExecutionResults,
    pub(crate) late_cancellation: CompositeLateCancellationPosture,
    pub(crate) cost_counters: CompositePublicationCostCounters,
}

/// Read-only owner evidence prepared before entering the branch critical
/// section. It cannot authorize a performed publication by itself.
pub(crate) struct PreparedPublicationRecord {
    cutoff: Option<crate::publication::ProductMovementCutoff>,
    envelope: Arc<CanonicalPublicationEnvelope>,
    commit: Arc<crate::history::CompositeRuntimeWorldCommit>,
    component_results: CompositeOwnerExecutionResults,
    late_cancellation: CompositeLateCancellationPosture,
    cost_counters: CompositePublicationCostCounters,
}

impl CanonicalPublicationEnvelope {
    pub(crate) fn reserve(
        commit_identity: CompositeCommitIdentity,
        attempt_identity: CompositePublicationAttemptIdentity,
        expected: ProductBranchReferenceSnapshot,
    ) -> Arc<Self> {
        Arc::new(Self {
            commit_identity,
            attempt_identity,
            expected,
            facts: OnceLock::new(),
            committed: AtomicBool::new(false),
            delivery: AtomicU8::new(delivery::AVAILABLE),
            successor_observation: Mutex::new(None),
        })
    }

    pub(crate) fn prepare(
        self: &Arc<Self>,
        commit: &Arc<crate::history::CompositeRuntimeWorldCommit>,
        component_results: &CompositeOwnerExecutionResults,
        late_cancellation: CompositeLateCancellationPosture,
        cost_counters: CompositePublicationCostCounters,
    ) -> PreparedPublicationRecord {
        assert_eq!(commit.identity(), &self.commit_identity);
        assert_eq!(
            commit.provenance(),
            &crate::history::CompositeCommitProvenance::Publication(self.attempt_identity.clone())
        );
        assert!(commit.matches_owner_results(self.expected.basis(), component_results));
        PreparedPublicationRecord {
            cutoff: None,
            envelope: Arc::clone(self),
            commit: Arc::clone(commit),
            component_results: component_results.evidence_image(),
            late_cancellation,
            cost_counters,
        }
    }

    pub(crate) fn commit_identity(&self) -> &CompositeCommitIdentity {
        &self.commit_identity
    }

    pub(crate) fn branch_name(&self) -> &crate::branch::ProductBranchName {
        self.expected.branch().name()
    }

    pub(crate) fn attempt_identity(&self) -> &CompositePublicationAttemptIdentity {
        &self.attempt_identity
    }

    pub(crate) fn facts(&self) -> Option<&PerformedPublicationFacts> {
        self.committed
            .load(Ordering::Acquire)
            .then(|| self.facts.get())
            .flatten()
    }

    pub(crate) fn install_successor_observation(
        &self,
        observation: crate::branch::ProductBranchObservation,
    ) {
        let mut slot = self
            .successor_observation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(slot.replace(observation).is_none());
    }

    fn take_successor_observation(&self) -> Option<crate::branch::ProductBranchObservation> {
        self.successor_observation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    }

    fn restore_successor_observation(&self, observation: crate::branch::ProductBranchObservation) {
        let mut slot = self
            .successor_observation
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(slot.replace(observation).is_none());
    }
}

pub(crate) struct StagedPublicationRecord {
    envelope: Arc<CanonicalPublicationEnvelope>,
    facts: PerformedPublicationFacts,
    cutoff: Option<crate::publication::ProductMovementCutoff>,
}
impl PreparedPublicationRecord {
    pub(crate) fn with_cutoff(
        mut self,
        cutoff: Option<crate::publication::ProductMovementCutoff>,
    ) -> Self {
        self.cutoff = cutoff;
        self
    }
    pub(crate) fn check_cutoff(
        &self,
    ) -> Result<(), crate::publication::ProductMovementCutoffDenial> {
        self.cutoff.as_ref().map_or(Ok(()), |cutoff| cutoff.check())
    }

    /// Validate all bindings and fill the preallocated record before the cell
    /// swaps. The caller must then swap and mark committed without a fallible
    /// operation, allocation, callback, or destructor between those steps.
    pub(crate) fn stage(
        mut self,
        movement: &ProductBranchReferenceMovement,
    ) -> StagedPublicationRecord {
        assert_eq!(movement.before(), &self.envelope.expected);
        assert_eq!(
            movement.after().selected_commit(),
            &self.envelope.commit_identity
        );
        assert!(!self.envelope.committed.load(Ordering::Acquire));
        assert!(
            std::ptr::eq(movement.after().commit(), self.commit.as_ref()),
            "the movement installs the immutable commit whose full owner evidence was validated"
        );
        self.cost_counters.record_cas_win();
        let facts = PerformedPublicationFacts {
            movement: movement.clone(),
            component_results: self.component_results,
            late_cancellation: self.late_cancellation,
            cost_counters: self.cost_counters,
        };
        StagedPublicationRecord {
            envelope: self.envelope,
            facts,
            cutoff: self.cutoff,
        }
    }
}

impl StagedPublicationRecord {
    /// The cell has swapped while readers remain excluded. Record the atomic
    /// cancellation observation and expose this same canonical envelope.
    pub(crate) fn mark_committed(mut self) {
        if self
            .cutoff
            .as_ref()
            .is_some_and(|cutoff| cutoff.cancellation_after_movement())
        {
            self.facts.late_cancellation =
                CompositeLateCancellationPosture::RequestedAfterProductMovement;
            self.facts.cost_counters.record_cancellation_observation();
        }
        self.envelope
            .facts
            .set(self.facts)
            .expect("one reserved entry records one movement");
        self.envelope.committed.store(true, Ordering::Release);
    }
}
