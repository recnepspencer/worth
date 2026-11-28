use crate::input::envelope::{
    BridgeProducerMetadata, TruthBranchIdentity, TruthCommitIdentity, TruthPatchIdentity,
};
use crate::routing::canonicalization::{digest_string, invalidation_digest_basis};
use crate::routing::counters::BridgeRoutingCounters;
use crate::routing::planning::BridgeRouteIdentity;
use crate::routing::proof::BridgeRouteContractProof;
use crate::snapshot::{BridgeSnapshotToken, TruthSnapshotIdentity};

use super::{
    BridgeInvalidationIdentity, BridgeSubscriptionSliceIdentity, CanonicalInvalidationTargets,
    CanonicalSubscriptionSlices, ValidatedBridgeLoweringPlan,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeInvalidationArtifact {
    lowering_plan: ValidatedBridgeLoweringPlan,
    invalidation_identity: BridgeInvalidationIdentity,
    snapshot_token: BridgeSnapshotToken,
    counters: BridgeRoutingCounters,
}

impl BridgeInvalidationArtifact {
    pub(crate) fn new(
        lowering_plan: ValidatedBridgeLoweringPlan,
        invalidation_identity: BridgeInvalidationIdentity,
        snapshot_token: BridgeSnapshotToken,
        counters: BridgeRoutingCounters,
    ) -> Self {
        Self {
            lowering_plan,
            invalidation_identity,
            snapshot_token,
            counters,
        }
    }

    pub fn route_identity(&self) -> &BridgeRouteIdentity {
        self.lowering_plan.plan().route_identity()
    }

    pub fn invalidation_identity(&self) -> &BridgeInvalidationIdentity {
        &self.invalidation_identity
    }

    pub fn source_commit(&self) -> &TruthCommitIdentity {
        self.lowering_plan.plan().source_commit()
    }

    pub fn source_branch(&self) -> &TruthBranchIdentity {
        self.lowering_plan.plan().source_branch()
    }

    pub fn source_patch(&self) -> &TruthPatchIdentity {
        self.lowering_plan.plan().source_patch()
    }

    pub fn source_snapshot(&self) -> &TruthSnapshotIdentity {
        self.lowering_plan.plan().source_snapshot()
    }

    pub fn invalidation_targets(&self) -> &CanonicalInvalidationTargets {
        self.lowering_plan.plan().invalidation_targets()
    }

    pub fn subscription_slices(&self) -> &CanonicalSubscriptionSlices {
        self.lowering_plan.plan().subscription_slices()
    }

    pub fn subscription_slice_identity(&self) -> &BridgeSubscriptionSliceIdentity {
        self.lowering_plan.plan().subscription_slice_identity()
    }

    pub fn snapshot_token(&self) -> &BridgeSnapshotToken {
        &self.snapshot_token
    }

    pub fn counters(&self) -> &BridgeRoutingCounters {
        &self.counters
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeSignalInvalidationDelivery {
    route_identity: BridgeRouteIdentity,
    invalidation_identity: BridgeInvalidationIdentity,
    source_snapshot: TruthSnapshotIdentity,
    contract_proof: BridgeRouteContractProof,
    invalidation_targets: CanonicalInvalidationTargets,
    subscription_slices: CanonicalSubscriptionSlices,
    subscription_slice_identity: BridgeSubscriptionSliceIdentity,
}

impl BridgeSignalInvalidationDelivery {
    pub(crate) fn new(
        artifact: &BridgeInvalidationArtifact,
        contract_proof: &BridgeRouteContractProof,
    ) -> Self {
        Self {
            route_identity: artifact.route_identity().clone(),
            invalidation_identity: artifact.invalidation_identity().clone(),
            source_snapshot: artifact.source_snapshot().clone(),
            contract_proof: contract_proof.clone(),
            invalidation_targets: artifact.invalidation_targets().clone(),
            subscription_slices: artifact.subscription_slices().clone(),
            subscription_slice_identity: artifact.subscription_slice_identity().clone(),
        }
    }

    pub fn route_identity(&self) -> &BridgeRouteIdentity {
        &self.route_identity
    }

    pub fn invalidation_identity(&self) -> &BridgeInvalidationIdentity {
        &self.invalidation_identity
    }

    pub fn source_snapshot(&self) -> &TruthSnapshotIdentity {
        &self.source_snapshot
    }

    pub fn producer_metadata(&self) -> &BridgeProducerMetadata {
        self.contract_proof.producer_metadata()
    }

    pub fn mapping_context_digest(&self) -> &str {
        self.contract_proof.mapping_context_digest()
    }

    pub fn planning_provenance_digest(&self) -> &str {
        self.contract_proof.planning_provenance_digest()
    }

    pub fn route_planning_policy_digest(&self) -> Option<&str> {
        self.contract_proof.route_planning_policy_digest()
    }

    pub fn planning_summary_digest(&self) -> &str {
        self.contract_proof.planning_summary_digest()
    }

    pub fn lowering_provenance_digest(&self) -> &str {
        self.contract_proof.lowering_provenance_digest()
    }

    pub fn lowering_summary_digest(&self) -> &str {
        self.contract_proof.lowering_summary_digest()
    }

    pub fn contract_proof(&self) -> &BridgeRouteContractProof {
        &self.contract_proof
    }

    pub fn invalidation_targets(&self) -> &CanonicalInvalidationTargets {
        &self.invalidation_targets
    }

    pub fn subscription_slices(&self) -> &CanonicalSubscriptionSlices {
        &self.subscription_slices
    }

    pub fn subscription_slice_identity(&self) -> &BridgeSubscriptionSliceIdentity {
        &self.subscription_slice_identity
    }
}

pub(crate) fn lower_validated_route(
    validated: ValidatedBridgeLoweringPlan,
    counters: BridgeRoutingCounters,
) -> BridgeInvalidationArtifact {
    let invalidation_basis = invalidation_digest_basis(
        validated.plan().route_identity(),
        &validated.plan().source_commit().canonical_label(),
        &validated.plan().source_patch().canonical_label(),
        &validated.plan().source_snapshot().canonical_label(),
        validated.plan().invalidation_targets().targets(),
    );
    let snapshot_basis = format!(
        "snapshot-token|route={}|commit={}|patch={}|snapshot={}",
        validated.plan().route_identity().as_str(),
        validated.plan().source_commit().canonical_label(),
        validated.plan().source_patch().canonical_label(),
        validated.plan().source_snapshot().canonical_label()
    );
    let counters = counters
        .with_digest_computations(2)
        .with_digest_input_bytes(invalidation_basis.len() + snapshot_basis.len());
    let plan = validated.plan();
    let invalidation_identity = BridgeInvalidationIdentity::admit_bridge_owned(digest_string(
        "invalidation",
        &invalidation_basis,
    ));
    let snapshot_token = BridgeSnapshotToken::issued(
        plan.source_snapshot().clone(),
        digest_string("snapshot-token", &snapshot_basis),
    );

    BridgeInvalidationArtifact::new(validated, invalidation_identity, snapshot_token, counters)
}
