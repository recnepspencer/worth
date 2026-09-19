use worth_proof::AuthorityWitness;

use crate::branch::ProductBranchReferenceSnapshot;
use crate::history::{CompositeRuntimeWorldCommit, PublicationDeliveryClaim};
use crate::identity::CompositePublicationAttemptIdentity;

use super::{CompositeOwnerExecutionResults, CompositePublicationCostCounters};

worth_proof::authority_marker!(pub(crate) CompositePublicationAuthorityMarker);

/// Cancellation observed after the branch reference moved is retained as
/// evidence; it cannot turn a performed movement back into no-effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositeLateCancellationPosture {
    NotRequested,
    RequestedBeforeProductMovement,
    RequestedAfterProductMovement,
}

/// Linear delivery of one canonical performed publication. Inspection borrows
/// the history-owned facts. Dropping this unconsumed claim lets the live owner
/// recover it through the same exclusive delivery lane.
#[must_use = "a performed publication must be handed to the product owner"]
pub struct PerformedCompositePublication {
    delivery: PublicationDeliveryClaim,
    _authority: AuthorityWitness<CompositePublicationAuthorityMarker>,
}

impl std::fmt::Debug for PerformedCompositePublication {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PerformedCompositePublication")
            .field("commit", self.commit().identity())
            .field("old_product_head", self.old_product_head())
            .field("new_product_head", self.new_product_head())
            .field("attempt_identity", self.attempt_identity())
            .field("component_results", self.component_results())
            .field("late_cancellation", &self.late_cancellation())
            .field("cost_counters", &self.cost_counters())
            .finish_non_exhaustive()
    }
}

impl PerformedCompositePublication {
    pub(crate) fn owner_issued(delivery: PublicationDeliveryClaim) -> Self {
        assert!(
            delivery.envelope().facts().is_some(),
            "only a committed envelope authorizes performed delivery"
        );
        Self {
            delivery,
            _authority: AuthorityWitness::from_authority_marker(
                CompositePublicationAuthorityMarker::seal(),
            ),
        }
    }

    pub fn old_product_head(&self) -> &ProductBranchReferenceSnapshot {
        self.facts().movement.before()
    }

    pub fn new_product_head(&self) -> &ProductBranchReferenceSnapshot {
        self.facts().movement.after()
    }

    pub fn commit(&self) -> &CompositeRuntimeWorldCommit {
        self.new_product_head().commit()
    }

    pub fn product_head(&self) -> &ProductBranchReferenceSnapshot {
        self.new_product_head()
    }

    pub fn attempt_identity(&self) -> &CompositePublicationAttemptIdentity {
        self.delivery.envelope().attempt_identity()
    }

    pub fn component_results(&self) -> &CompositeOwnerExecutionResults {
        &self.facts().component_results
    }

    pub fn late_cancellation(&self) -> CompositeLateCancellationPosture {
        self.facts().late_cancellation
    }

    pub fn cost_counters(&self) -> CompositePublicationCostCounters {
        self.facts().cost_counters
    }

    /// The product handoff consumes the delivery capability. Read-only facts
    /// retained by history cannot reopen a consumed delivery.
    pub fn consume(self) -> ConsumedCompositePublication {
        ConsumedCompositePublication {
            delivery: self.delivery.consume(),
        }
    }

    fn facts(&self) -> &crate::history::PerformedPublicationFacts {
        self.delivery
            .envelope()
            .facts()
            .expect("a performed delivery has canonical committed facts")
    }
}

/// Descriptive facts from a permanently consumed delivery. This receipt cannot
/// authorize another product terminal or reopen delivery.
#[derive(Debug)]
pub struct ConsumedCompositePublication {
    delivery: PublicationDeliveryClaim,
}
impl ConsumedCompositePublication {
    pub fn take_successor_observation(
        &mut self,
    ) -> Option<crate::branch::ProductBranchObservation> {
        self.delivery.take_successor_observation()
    }
    pub fn old_product_head(&self) -> &ProductBranchReferenceSnapshot {
        self.facts().movement.before()
    }
    pub fn new_product_head(&self) -> &ProductBranchReferenceSnapshot {
        self.facts().movement.after()
    }
    pub fn commit(&self) -> &CompositeRuntimeWorldCommit {
        self.new_product_head().commit()
    }
    pub fn attempt_identity(&self) -> &CompositePublicationAttemptIdentity {
        self.delivery.envelope().attempt_identity()
    }
    pub fn component_results(&self) -> &CompositeOwnerExecutionResults {
        &self.facts().component_results
    }
    pub fn late_cancellation(&self) -> CompositeLateCancellationPosture {
        self.facts().late_cancellation
    }
    pub fn cost_counters(&self) -> CompositePublicationCostCounters {
        self.facts().cost_counters
    }
    fn facts(&self) -> &crate::history::PerformedPublicationFacts {
        self.delivery
            .envelope()
            .facts()
            .expect("consumption preserves canonical facts")
    }
}
