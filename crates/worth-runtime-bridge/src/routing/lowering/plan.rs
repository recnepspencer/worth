use crate::error::{BridgeRouteError, BridgeRouteErrorKind};
use crate::input::envelope::{TruthBranchIdentity, TruthCommitIdentity, TruthPatchIdentity};
use crate::routing::canonicalization::{
    digest_string, lowering_provenance_digest_basis, lowering_summary_digest_basis,
    subscription_slice_digest_basis,
};
use crate::routing::planning::{
    BridgeExecutionCounts, BridgeRouteIdentity, BridgeRouteSourceSummary,
};
use crate::snapshot::TruthSnapshotIdentity;

use super::summaries::{
    BridgeLoweringPlanSummary, BridgeLoweringProvenance, BridgeLoweringSummary,
};
use super::{
    BridgeInvalidationTarget, BridgeSubscriptionSlice, BridgeSubscriptionSliceIdentity,
    CanonicalInvalidationTargets, CanonicalSubscriptionSlices,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BridgeLoweringPlan {
    route_identity: BridgeRouteIdentity,
    source_branch: TruthBranchIdentity,
    source_commit: TruthCommitIdentity,
    source_patch: TruthPatchIdentity,
    source_snapshot: TruthSnapshotIdentity,
    invalidation_targets: CanonicalInvalidationTargets,
    subscription_slices: CanonicalSubscriptionSlices,
    subscription_slice_identity: BridgeSubscriptionSliceIdentity,
    planned_read_count: usize,
    provenance: BridgeLoweringProvenance,
    lowering_summary: BridgeLoweringSummary,
    summary: BridgeLoweringPlanSummary,
    digest_computation_count: usize,
    digest_input_bytes: usize,
}

impl BridgeLoweringPlan {
    pub(crate) fn new(
        route_identity: BridgeRouteIdentity,
        source_branch: TruthBranchIdentity,
        source_commit: TruthCommitIdentity,
        source_patch: TruthPatchIdentity,
        source_snapshot: TruthSnapshotIdentity,
        invalidation_targets: Vec<BridgeInvalidationTarget>,
        subscription_slices: Vec<BridgeSubscriptionSlice>,
        planned_read_count: usize,
        provenance: BridgeLoweringProvenance,
    ) -> Self {
        let invalidation_targets = CanonicalInvalidationTargets::new(invalidation_targets);
        let subscription_slices = CanonicalSubscriptionSlices::new(subscription_slices);
        let subscription_slice_basis = subscription_slice_digest_basis(
            &source_snapshot.canonical_label(),
            subscription_slices.slices(),
        );
        let subscription_slice_identity = BridgeSubscriptionSliceIdentity::admit_bridge_owned(
            digest_string("subscription-slices", &subscription_slice_basis),
        );
        let execution_counts = BridgeExecutionCounts::new(
            invalidation_targets.len(),
            subscription_slices.len(),
            planned_read_count,
        );
        let lowering_summary_basis = lowering_summary_digest_basis(
            &route_identity,
            invalidation_targets.targets(),
            subscription_slices.slices(),
            planned_read_count,
        );
        let lowering_summary = BridgeLoweringSummary::new(
            route_identity.clone(),
            execution_counts.clone(),
            digest_string("lowering-summary", &lowering_summary_basis),
        );
        let summary = BridgeLoweringPlanSummary::new(
            route_identity.clone(),
            BridgeRouteSourceSummary::new(
                source_branch.clone(),
                source_commit.clone(),
                source_patch.clone(),
                source_snapshot.clone(),
            ),
            execution_counts,
            subscription_slice_identity.clone(),
        );

        Self {
            route_identity,
            source_branch,
            source_commit,
            source_patch,
            source_snapshot,
            invalidation_targets,
            subscription_slices,
            subscription_slice_identity,
            planned_read_count,
            provenance,
            lowering_summary,
            summary,
            digest_computation_count: 2,
            digest_input_bytes: subscription_slice_basis.len() + lowering_summary_basis.len(),
        }
    }

    pub(crate) fn summary(&self) -> &BridgeLoweringPlanSummary {
        &self.summary
    }

    pub(crate) fn route_identity(&self) -> &BridgeRouteIdentity {
        &self.route_identity
    }

    pub(crate) fn source_commit(&self) -> &TruthCommitIdentity {
        &self.source_commit
    }

    pub(crate) fn source_branch(&self) -> &TruthBranchIdentity {
        &self.source_branch
    }

    pub(crate) fn source_patch(&self) -> &TruthPatchIdentity {
        &self.source_patch
    }

    pub(crate) fn source_snapshot(&self) -> &TruthSnapshotIdentity {
        &self.source_snapshot
    }

    pub(crate) fn invalidation_targets(&self) -> &CanonicalInvalidationTargets {
        &self.invalidation_targets
    }

    pub(crate) fn subscription_slices(&self) -> &CanonicalSubscriptionSlices {
        &self.subscription_slices
    }

    pub(crate) fn subscription_slice_identity(&self) -> &BridgeSubscriptionSliceIdentity {
        &self.subscription_slice_identity
    }

    pub(crate) fn provenance(&self) -> &BridgeLoweringProvenance {
        &self.provenance
    }

    pub(crate) fn lowering_summary(&self) -> &BridgeLoweringSummary {
        &self.lowering_summary
    }

    pub(crate) fn digest_computation_count(&self) -> usize {
        self.digest_computation_count
    }

    pub(crate) fn digest_input_bytes(&self) -> usize {
        self.digest_input_bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValidatedBridgeLoweringPlan {
    plan: BridgeLoweringPlan,
}

impl ValidatedBridgeLoweringPlan {
    pub(crate) fn from_plan(plan: &BridgeLoweringPlan) -> Result<Self, BridgeRouteError> {
        if plan.provenance.route_identity() != plan.route_identity() {
            return Err(BridgeRouteError::new(
                BridgeRouteErrorKind::InvalidLoweringContract,
                "Bridge lowering provenance did not agree with the planned route identity.",
            ));
        }
        if plan.provenance.planning_provenance().route_identity() != plan.route_identity() {
            return Err(BridgeRouteError::new(
                BridgeRouteErrorKind::InvalidLoweringContract,
                "Bridge lowering provenance carried planning provenance for a different route identity.",
            ));
        }
        if plan.lowering_summary.route_identity() != plan.route_identity() {
            return Err(BridgeRouteError::new(
                BridgeRouteErrorKind::InvalidLoweringContract,
                "Bridge lowering summary did not agree with the planned route identity.",
            ));
        }
        if plan.summary.subscription_slice_identity() != plan.subscription_slice_identity() {
            return Err(BridgeRouteError::new(
                BridgeRouteErrorKind::InvalidLoweringContract,
                "Bridge lowering plan summary did not agree with the canonical subscription slice identity.",
            ));
        }
        if plan.summary.invalidation_target_count() != plan.invalidation_targets().len() {
            return Err(BridgeRouteError::new(
                BridgeRouteErrorKind::InvalidLoweringContract,
                "Bridge lowering plan summary invalidation-target count did not match the canonical lowering targets.",
            ));
        }
        if plan.summary.subscription_slice_count() != plan.subscription_slices().len() {
            return Err(BridgeRouteError::new(
                BridgeRouteErrorKind::InvalidLoweringContract,
                "Bridge lowering plan summary subscription-slice count did not match the canonical lowering slices.",
            ));
        }
        if plan.lowering_summary.planned_read_count() != plan.planned_read_count {
            return Err(BridgeRouteError::new(
                BridgeRouteErrorKind::InvalidLoweringContract,
                "Bridge lowering summary planned-read count did not match the canonical read packet breadth.",
            ));
        }
        let expected_provenance_digest = digest_string(
            "lowering-provenance",
            &lowering_provenance_digest_basis(
                plan.route_identity(),
                plan.provenance.planning_provenance().digest(),
                &plan.source_commit().canonical_label(),
                &plan.source_patch().canonical_label(),
                &plan.source_snapshot().canonical_label(),
            ),
        );
        if plan.provenance.digest() != expected_provenance_digest.as_ref() {
            return Err(BridgeRouteError::new(
                BridgeRouteErrorKind::InvalidLoweringContract,
                "Bridge lowering provenance digest did not match the canonical lowering provenance basis.",
            ));
        }
        let expected_summary_digest = digest_string(
            "lowering-summary",
            &lowering_summary_digest_basis(
                plan.route_identity(),
                plan.invalidation_targets().targets(),
                plan.subscription_slices().slices(),
                plan.planned_read_count,
            ),
        );
        if plan.lowering_summary.digest() != expected_summary_digest.as_ref() {
            return Err(BridgeRouteError::new(
                BridgeRouteErrorKind::InvalidLoweringContract,
                "Bridge lowering summary digest did not match the canonical lowering summary basis.",
            ));
        }

        Ok(Self { plan: plan.clone() })
    }

    pub(crate) fn plan(&self) -> &BridgeLoweringPlan {
        &self.plan
    }

    pub(crate) fn provenance(&self) -> &BridgeLoweringProvenance {
        self.plan.provenance()
    }

    pub(crate) fn summary(&self) -> &BridgeLoweringSummary {
        self.plan.lowering_summary()
    }
}
