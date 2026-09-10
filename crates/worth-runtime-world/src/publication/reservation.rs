use crate::branch::ProductBranchObservation;
use crate::identity::CompositePublicationAttemptIdentity;
use crate::lifecycle::RuntimeWorldInstant;

use super::{
    CompositeAttemptProgress, CompositePublicationCostCounters, LoweredOwnerComponentPlan,
    NoEffectCompositePublication, OwnerExecutionSettlement,
};

#[path = "reservation/capacities.rs"]
mod capacities;

pub(crate) use capacities::{ReservedAttemptCapacities, ReservedAttemptCapacityInputs};

/// Component calls have one fixed order. There is no cross-owner lock held
/// across those calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositePublicationOrder {
    RelationalThenSignal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositeAttemptCancellationPosture {
    Open,
    CancellationObserved,
}

/// Reserved, attempt-affine state between lowering and owner execution. All
/// capacity fields are live owner-issued reservations; no copied limit can
/// authorize the later phases.
#[must_use = "a reserved publication attempt must be executed or cleaned up"]
pub struct ReservedCompositePublicationAttempt {
    identity: CompositePublicationAttemptIdentity,
    expected_head: ProductBranchObservation,
    predecessor_basis: crate::basis::AdmittedCompositeRuntimeWorldBasis,
    plan: LoweredOwnerComponentPlan,
    custody: super::ActiveAttemptCustody,
    cancellation: CompositeAttemptCancellationPosture,
    deadline: Option<RuntimeWorldInstant>,
    order: CompositePublicationOrder,
    progress: std::sync::Arc<CompositeAttemptProgress>,
    counters: CompositePublicationCostCounters,
}

impl std::fmt::Debug for ReservedCompositePublicationAttempt {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ReservedCompositePublicationAttempt")
            .field("identity", &self.identity)
            .field("expected_head", &self.expected_head)
            .field("cancellation", &self.cancellation)
            .field("deadline", &self.deadline)
            .field("order", &self.order)
            .field("progress", &self.progress)
            .field("counters", &self.counters)
            .finish_non_exhaustive()
    }
}

impl ReservedCompositePublicationAttempt {
    pub(crate) fn reserve_conditional_definition_custody(
        &mut self,
    ) -> std::sync::Arc<super::ConditionalDefinitionAttemptCustody> {
        self.custody.reserve_conditional_definition_custody()
    }

    pub(crate) fn new(
        identity: CompositePublicationAttemptIdentity,
        expected_head: ProductBranchObservation,
        predecessor_basis: crate::basis::AdmittedCompositeRuntimeWorldBasis,
        plan: LoweredOwnerComponentPlan,
        capacities: ReservedAttemptCapacities,
        deadline: Option<RuntimeWorldInstant>,
    ) -> Self {
        let custody = super::ActiveAttemptCustody::register(
            identity.clone(),
            expected_head.clone(),
            deadline,
            capacities,
        );
        let progress = custody.progress();
        Self {
            identity,
            expected_head,
            predecessor_basis,
            plan,
            custody,
            cancellation: CompositeAttemptCancellationPosture::Open,
            deadline,
            order: CompositePublicationOrder::RelationalThenSignal,
            progress,
            counters: {
                let mut costs = CompositePublicationCostCounters::zero();
                costs.record_history_slot_reserved();
                costs
            },
        }
    }

    pub fn expected_head(&self) -> &ProductBranchObservation {
        &self.expected_head
    }

    pub fn predecessor_basis(&self) -> &crate::basis::AdmittedCompositeRuntimeWorldBasis {
        &self.predecessor_basis
    }

    pub fn plan(&self) -> &LoweredOwnerComponentPlan {
        &self.plan
    }

    pub fn deadline(&self) -> Option<RuntimeWorldInstant> {
        self.deadline
    }

    #[cfg(test)]
    pub fn order(&self) -> CompositePublicationOrder {
        self.order
    }

    pub fn progress(&self) -> &CompositeAttemptProgress {
        &self.progress
    }

    pub(crate) fn unpublished_recovery_handle(
        &self,
    ) -> crate::recovery::ProductUnpublishedRecoveryHandle {
        self.custody.unpublished_recovery_handle()
    }

    pub(crate) fn record_progress(&mut self, progress: &CompositeAttemptProgress) {
        self.progress = self.custody.record_progress(progress.retained_image());
    }

    /// Structural counters are initialized when the attempt is reserved, before
    /// the first owner effect, so a caller can read an honest zeroed scope.
    #[cfg(test)]
    pub fn counters(&self) -> &CompositePublicationCostCounters {
        &self.counters
    }

    pub(crate) fn counters_mut(&mut self) -> &mut CompositePublicationCostCounters {
        &mut self.counters
    }

    #[cfg(test)]
    pub fn cancellation_posture(&self) -> CompositeAttemptCancellationPosture {
        self.cancellation
    }

    pub(crate) fn observe_cancellation(&mut self) {
        self.cancellation = CompositeAttemptCancellationPosture::CancellationObserved;
    }

    pub(crate) fn begin_owner_execution(&mut self) {
        self.custody.begin_owner_execution();
    }

    pub(crate) fn take_relational_candidate(
        &mut self,
    ) -> Option<worth_relational::facade::mvcc::PreparedRelationalCommitCandidate> {
        self.plan.take_relational_candidate()
    }

    /// Consume a still-pre-effect reservation into the only no-effect
    /// cancellation terminal. Dropping the attempt releases every capacity.
    pub fn cancel(self) -> NoEffectCompositePublication {
        assert_eq!(
            self.progress.owner_effect_count(),
            0,
            "only a pre-effect phase can cancel as no-effect"
        );
        NoEffectCompositePublication::new(
            super::NoEffectCause::CancelledBeforeEffect,
            Some(self.expected_head),
        )
    }

    pub(crate) fn settle(mut self, progress: CompositeAttemptProgress) -> OwnerExecutionSettlement {
        self.record_progress(&progress);
        self.custody.begin_publication();
        OwnerExecutionSettlement::new(self, progress)
    }

    pub(crate) fn settle_with_successor_basis(
        mut self,
        progress: CompositeAttemptProgress,
        successor_basis: crate::basis::AdmittedCompositeRuntimeWorldBasis,
    ) -> OwnerExecutionSettlement {
        self.record_progress(&progress);
        self.custody.record_successor(successor_basis.clone());
        self.custody.begin_publication();
        OwnerExecutionSettlement::with_successor_basis(self, progress, successor_basis)
    }

    pub(crate) fn into_parts(self) -> ReservedPublicationAttemptParts {
        ReservedPublicationAttemptParts {
            identity: self.identity,
            expected_head: self.expected_head,
            plan: self.plan,
            custody: self.custody,
            cancellation: self.cancellation,
            deadline: self.deadline,
            counters: self.counters,
        }
    }
}

/// The exact linear contents of a consumed publication attempt. Every field is
/// a live owner-issued reservation or the evidence bound to one.
pub(crate) struct ReservedPublicationAttemptParts {
    pub(crate) identity: CompositePublicationAttemptIdentity,
    pub(crate) expected_head: ProductBranchObservation,
    pub(crate) plan: LoweredOwnerComponentPlan,
    pub(crate) custody: super::ActiveAttemptCustody,
    pub(crate) cancellation: CompositeAttemptCancellationPosture,
    pub(crate) deadline: Option<RuntimeWorldInstant>,
    pub(crate) counters: CompositePublicationCostCounters,
}

mod creation;
pub(crate) use creation::{ReservedBranchCreationAttempt, ReservedBranchCreationInputs};
