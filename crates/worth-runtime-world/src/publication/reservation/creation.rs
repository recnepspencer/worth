use crate::branch::{LoweredBranchCreationPlan, ProductBranchObservation, ReservedCustodySlot};
use crate::identity::CompositePublicationAttemptIdentity;
use crate::lifecycle::RuntimeWorldInstant;
use crate::publication::{
    ActiveAttemptCustody, CompositeAttemptCancellationPosture, CompositeAttemptProgress,
    CompositePublicationCostCounters, CompositePublicationOrder, ReservedAttemptCapacities,
};
#[cfg(test)]
use crate::publication::{NoEffectCause, NoEffectCompositePublication};

/// Branch creation reserves the same resources under the same rules and
/// consumes into a creation terminal. Custody slots for every owner fork this
/// creation will perform are charged here, before the first owner effect.
#[must_use = "a reserved branch-creation attempt must be executed or cleaned up"]
pub(crate) struct ReservedBranchCreationAttempt {
    identity: CompositePublicationAttemptIdentity,
    source: ProductBranchObservation,
    plan: LoweredBranchCreationPlan,
    custody: ActiveAttemptCustody,
    relational_custody: Option<ReservedCustodySlot>,
    signal_custody: Option<ReservedCustodySlot>,
    cancellation: CompositeAttemptCancellationPosture,
    deadline: Option<RuntimeWorldInstant>,
    order: CompositePublicationOrder,
    progress: std::sync::Arc<CompositeAttemptProgress>,
    counters: CompositePublicationCostCounters,
}

impl std::fmt::Debug for ReservedBranchCreationAttempt {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ReservedBranchCreationAttempt")
            .field("identity", &self.identity)
            .field("source", &self.source)
            .field("plan", &self.plan)
            .field("cancellation", &self.cancellation)
            .field("deadline", &self.deadline)
            .field("order", &self.order)
            .field("progress", &self.progress)
            .field("counters", &self.counters)
            .finish_non_exhaustive()
    }
}

pub(crate) struct ReservedBranchCreationInputs {
    pub(crate) identity: CompositePublicationAttemptIdentity,
    pub(crate) source: ProductBranchObservation,
    pub(crate) plan: LoweredBranchCreationPlan,
    pub(crate) capacities: ReservedAttemptCapacities,
    pub(crate) observation_capacity: crate::retention::ReservedObservationCapacity,
    pub(crate) relational_custody: Option<ReservedCustodySlot>,
    pub(crate) signal_custody: Option<ReservedCustodySlot>,
    pub(crate) deadline: Option<RuntimeWorldInstant>,
}

impl ReservedBranchCreationAttempt {
    pub(crate) fn new(inputs: ReservedBranchCreationInputs) -> Self {
        let ReservedBranchCreationInputs {
            identity,
            source,
            plan,
            capacities,
            observation_capacity,
            relational_custody,
            signal_custody,
            deadline,
        } = inputs;
        let mut custody =
            ActiveAttemptCustody::register(identity.clone(), source.clone(), deadline, capacities);
        custody.configure_creation(observation_capacity);
        let progress = custody.progress();
        Self {
            identity,
            source,
            plan,
            custody,
            relational_custody,
            signal_custody,
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

    #[cfg(test)]
    pub(crate) const fn identity(&self) -> &CompositePublicationAttemptIdentity {
        &self.identity
    }

    pub(crate) const fn source(&self) -> &ProductBranchObservation {
        &self.source
    }

    pub(crate) const fn plan(&self) -> &LoweredBranchCreationPlan {
        &self.plan
    }

    #[cfg(test)]
    pub(crate) const fn order(&self) -> CompositePublicationOrder {
        self.order
    }

    /// The reservation-time deadline, if the caller set one. Creation checks it
    /// before each owner effect, exactly as publication does.
    pub(crate) const fn deadline(&self) -> Option<RuntimeWorldInstant> {
        self.deadline
    }

    /// Structural counters are initialized when the attempt is reserved, before
    /// the first owner effect.
    #[cfg(test)]
    pub(crate) const fn counters(&self) -> &CompositePublicationCostCounters {
        &self.counters
    }

    /// The counters an owner leg records its contact on, taken before that
    /// owner is asked for anything. Creation and publication charge the same
    /// counter type, so a creation's cost is readable on the same axes.
    pub(crate) fn counters_mut(&mut self) -> &mut CompositePublicationCostCounters {
        &mut self.counters
    }

    pub(crate) fn take_relational_custody(&mut self) -> Option<ReservedCustodySlot> {
        self.relational_custody.take()
    }

    pub(crate) fn take_signal_custody(&mut self) -> Option<ReservedCustodySlot> {
        self.signal_custody.take()
    }

    pub(crate) fn begin_owner_execution(&mut self) {
        self.custody.begin_owner_execution();
    }

    /// A creation leaves its owner-effect phase when the last fork it will ever
    /// attempt has returned. Installing the product reference, and retaining
    /// the forks when installation cannot happen, are both publication-phase
    /// work, so they share the publication posture with an ordinary attempt.
    pub(crate) fn begin_publication(&mut self) {
        self.custody.begin_publication();
    }

    #[cfg(test)]
    pub(crate) fn cancel(self) -> NoEffectCompositePublication {
        assert_eq!(self.progress.owner_effect_count(), 0);
        NoEffectCompositePublication::new(NoEffectCause::CancelledBeforeEffect, Some(self.source))
    }

    pub(crate) fn into_parts(self) -> ReservedBranchCreationParts {
        ReservedBranchCreationParts {
            plan: self.plan,
            custody: self.custody,
        }
    }
}

/// The admitted plan and original custody needed by creation finalization.
pub(crate) struct ReservedBranchCreationParts {
    pub(crate) plan: LoweredBranchCreationPlan,
    pub(crate) custody: ActiveAttemptCustody,
}
impl ReservedBranchCreationAttempt {
    pub(crate) fn bind_destination(
        &mut self,
        witness: std::sync::Arc<crate::branch::registry::ProductBranchInstallationWitness>,
    ) {
        self.custody.bind_creation_destination(witness);
    }

    pub(crate) fn progress(&self) -> &CompositeAttemptProgress {
        &self.progress
    }

    pub(crate) fn record_progress(&mut self, progress: &CompositeAttemptProgress) {
        self.progress = self.custody.record_progress(progress.retained_image());
    }

    pub(crate) fn record_successor(
        &mut self,
        successor: crate::basis::AdmittedCompositeRuntimeWorldBasis,
    ) {
        self.custody.record_successor(successor);
    }
}
