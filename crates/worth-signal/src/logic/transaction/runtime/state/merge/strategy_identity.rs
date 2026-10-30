mod retained_charge;

use serde::{Deserialize, Serialize};

use super::adoption::{
    CausalityCarryPolicy, RetainedArtifactCarryPolicy, RuntimeArtifactCarryPolicy,
    SourceNodeAdoptionCarryPolicy,
};
use super::aspect_policy_registry::{AspectMergePolicyName, AspectMergePolicySelectionBasis};
use super::conflict_isolation_registry::{
    ConflictIsolationPolicyName, ConflictIsolationSelectionBasis,
};
use super::conflict_policy_registry::{ConflictPolicyName, ConflictPolicySelectionBasis};
use super::core::{BranchMergeStrategy, MergeBoundaryWitnessKind};
use super::deletion_policy_registry::{DeletionPolicyName, DeletionPolicySelectionBasis};
use super::identity_matcher_registry::{IdentityMatcherName, IdentityMatcherSelectionBasis};
use super::merge_base_registry::{MergeBaseSelectionBasis, MergeBaseStrategyName};
use super::source_only_policy_registry::{SourceOnlyPolicyName, SourceOnlyPolicySelectionBasis};
use super::strategy_registry::{MergeStrategyName, MergeStrategySelectionBasis};
use super::strategy_witness_denial::{
    ensure_non_empty_digest, SignalMergeStrategyWitnessDenial, SignalMergeStrategyWitnessDenialKind,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalAspectPolicyInventoryEntry {
    policy_name: AspectMergePolicyName,
    policy_digest: String,
    policy_basis: AspectMergePolicySelectionBasis,
}

impl SignalAspectPolicyInventoryEntry {
    pub(crate) fn new(
        policy_name: AspectMergePolicyName,
        policy_digest: String,
        policy_basis: AspectMergePolicySelectionBasis,
    ) -> Self {
        Self {
            policy_name,
            policy_digest,
            policy_basis,
        }
    }

    pub fn policy_name(&self) -> &AspectMergePolicyName {
        &self.policy_name
    }

    pub fn policy_digest(&self) -> &str {
        self.policy_digest.as_str()
    }

    pub fn policy_basis(&self) -> AspectMergePolicySelectionBasis {
        self.policy_basis
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SignalMergeStrategyIdentity {
    merge_strategy: BranchMergeStrategy,
    selected_strategy_name: MergeStrategyName,
    selected_strategy_digest: String,
    selected_strategy_basis: MergeStrategySelectionBasis,
    merge_base_name: MergeBaseStrategyName,
    merge_base_digest: String,
    merge_base_basis: MergeBaseSelectionBasis,
    lowered_strategy_bundle_digest: String,
}

impl SignalMergeStrategyIdentity {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        merge_strategy: BranchMergeStrategy,
        selected_strategy_name: MergeStrategyName,
        selected_strategy_digest: String,
        selected_strategy_basis: MergeStrategySelectionBasis,
        merge_base_name: MergeBaseStrategyName,
        merge_base_digest: String,
        merge_base_basis: MergeBaseSelectionBasis,
        lowered_strategy_bundle_digest: String,
    ) -> Result<Self, SignalMergeStrategyWitnessDenial> {
        ensure_non_empty_digest(
            "selected strategy",
            &selected_strategy_digest,
            SignalMergeStrategyWitnessDenialKind::EmptyDigestField,
        )?;
        ensure_non_empty_digest(
            "merge base",
            &merge_base_digest,
            SignalMergeStrategyWitnessDenialKind::EmptyDigestField,
        )?;
        ensure_non_empty_digest(
            "lowered strategy bundle",
            &lowered_strategy_bundle_digest,
            SignalMergeStrategyWitnessDenialKind::EmptyDigestField,
        )?;
        Ok(Self {
            merge_strategy,
            selected_strategy_name,
            selected_strategy_digest,
            selected_strategy_basis,
            merge_base_name,
            merge_base_digest,
            merge_base_basis,
            lowered_strategy_bundle_digest,
        })
    }

    pub fn merge_strategy(&self) -> BranchMergeStrategy {
        self.merge_strategy
    }

    pub fn selected_strategy_name(&self) -> &MergeStrategyName {
        &self.selected_strategy_name
    }

    pub fn selected_strategy_digest(&self) -> &str {
        self.selected_strategy_digest.as_str()
    }

    pub fn selected_strategy_basis(&self) -> MergeStrategySelectionBasis {
        self.selected_strategy_basis
    }

    pub fn merge_base_name(&self) -> &MergeBaseStrategyName {
        &self.merge_base_name
    }

    pub fn merge_base_digest(&self) -> &str {
        self.merge_base_digest.as_str()
    }

    pub fn merge_base_basis(&self) -> MergeBaseSelectionBasis {
        self.merge_base_basis
    }

    pub fn lowered_strategy_bundle_digest(&self) -> &str {
        self.lowered_strategy_bundle_digest.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SignalInvalidationStrategyIdentity {
    boundary_witness_kind: MergeBoundaryWitnessKind,
    conflict_isolation_name: ConflictIsolationPolicyName,
    conflict_isolation_digest: String,
    conflict_isolation_basis: ConflictIsolationSelectionBasis,
    identity_matcher_name: IdentityMatcherName,
    identity_matcher_digest: String,
    identity_matcher_basis: IdentityMatcherSelectionBasis,
}

impl SignalInvalidationStrategyIdentity {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        boundary_witness_kind: MergeBoundaryWitnessKind,
        conflict_isolation_name: ConflictIsolationPolicyName,
        conflict_isolation_digest: String,
        conflict_isolation_basis: ConflictIsolationSelectionBasis,
        identity_matcher_name: IdentityMatcherName,
        identity_matcher_digest: String,
        identity_matcher_basis: IdentityMatcherSelectionBasis,
    ) -> Result<Self, SignalMergeStrategyWitnessDenial> {
        ensure_non_empty_digest(
            "conflict isolation",
            &conflict_isolation_digest,
            SignalMergeStrategyWitnessDenialKind::EmptyDigestField,
        )?;
        ensure_non_empty_digest(
            "identity matcher",
            &identity_matcher_digest,
            SignalMergeStrategyWitnessDenialKind::EmptyDigestField,
        )?;
        Ok(Self {
            boundary_witness_kind,
            conflict_isolation_name,
            conflict_isolation_digest,
            conflict_isolation_basis,
            identity_matcher_name,
            identity_matcher_digest,
            identity_matcher_basis,
        })
    }

    pub fn boundary_witness_kind(&self) -> MergeBoundaryWitnessKind {
        self.boundary_witness_kind
    }

    pub fn conflict_isolation_name(&self) -> &ConflictIsolationPolicyName {
        &self.conflict_isolation_name
    }

    pub fn conflict_isolation_digest(&self) -> &str {
        self.conflict_isolation_digest.as_str()
    }

    pub fn conflict_isolation_basis(&self) -> ConflictIsolationSelectionBasis {
        self.conflict_isolation_basis
    }

    pub fn identity_matcher_name(&self) -> &IdentityMatcherName {
        &self.identity_matcher_name
    }

    pub fn identity_matcher_digest(&self) -> &str {
        self.identity_matcher_digest.as_str()
    }

    pub fn identity_matcher_basis(&self) -> IdentityMatcherSelectionBasis {
        self.identity_matcher_basis
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SignalDeliveryStrategyIdentity {
    conflict_policy_name: ConflictPolicyName,
    conflict_policy_digest: String,
    conflict_policy_basis: ConflictPolicySelectionBasis,
    source_only_policy_name: SourceOnlyPolicyName,
    source_only_policy_digest: String,
    source_only_policy_basis: SourceOnlyPolicySelectionBasis,
    deletion_policy_name: DeletionPolicyName,
    deletion_policy_digest: String,
    deletion_policy_basis: DeletionPolicySelectionBasis,
    aspect_policy_inventory: Vec<SignalAspectPolicyInventoryEntry>,
    runtime_artifact_carry_policies: Vec<RuntimeArtifactCarryPolicy>,
    retained_artifact_carry_policies: Vec<RetainedArtifactCarryPolicy>,
    causality_carry_policies: Vec<CausalityCarryPolicy>,
}

impl SignalDeliveryStrategyIdentity {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        conflict_policy_name: ConflictPolicyName,
        conflict_policy_digest: String,
        conflict_policy_basis: ConflictPolicySelectionBasis,
        source_only_policy_name: SourceOnlyPolicyName,
        source_only_policy_digest: String,
        source_only_policy_basis: SourceOnlyPolicySelectionBasis,
        deletion_policy_name: DeletionPolicyName,
        deletion_policy_digest: String,
        deletion_policy_basis: DeletionPolicySelectionBasis,
        aspect_policy_inventory: Vec<SignalAspectPolicyInventoryEntry>,
        runtime_artifact_carry_policies: Vec<RuntimeArtifactCarryPolicy>,
        retained_artifact_carry_policies: Vec<RetainedArtifactCarryPolicy>,
        causality_carry_policies: Vec<CausalityCarryPolicy>,
    ) -> Result<Self, SignalMergeStrategyWitnessDenial> {
        ensure_non_empty_digest(
            "conflict policy",
            &conflict_policy_digest,
            SignalMergeStrategyWitnessDenialKind::EmptyDigestField,
        )?;
        ensure_non_empty_digest(
            "source-only policy",
            &source_only_policy_digest,
            SignalMergeStrategyWitnessDenialKind::EmptyDigestField,
        )?;
        ensure_non_empty_digest(
            "deletion policy",
            &deletion_policy_digest,
            SignalMergeStrategyWitnessDenialKind::EmptyDigestField,
        )?;
        Ok(Self {
            conflict_policy_name,
            conflict_policy_digest,
            conflict_policy_basis,
            source_only_policy_name,
            source_only_policy_digest,
            source_only_policy_basis,
            deletion_policy_name,
            deletion_policy_digest,
            deletion_policy_basis,
            aspect_policy_inventory,
            runtime_artifact_carry_policies,
            retained_artifact_carry_policies,
            causality_carry_policies,
        })
    }

    pub fn conflict_policy_name(&self) -> &ConflictPolicyName {
        &self.conflict_policy_name
    }

    pub fn conflict_policy_digest(&self) -> &str {
        self.conflict_policy_digest.as_str()
    }

    pub fn conflict_policy_basis(&self) -> ConflictPolicySelectionBasis {
        self.conflict_policy_basis
    }

    pub fn source_only_policy_name(&self) -> &SourceOnlyPolicyName {
        &self.source_only_policy_name
    }

    pub fn source_only_policy_digest(&self) -> &str {
        self.source_only_policy_digest.as_str()
    }

    pub fn source_only_policy_basis(&self) -> SourceOnlyPolicySelectionBasis {
        self.source_only_policy_basis
    }

    pub fn deletion_policy_name(&self) -> &DeletionPolicyName {
        &self.deletion_policy_name
    }

    pub fn deletion_policy_digest(&self) -> &str {
        self.deletion_policy_digest.as_str()
    }

    pub fn deletion_policy_basis(&self) -> DeletionPolicySelectionBasis {
        self.deletion_policy_basis
    }

    pub fn aspect_policy_inventory(&self) -> &[SignalAspectPolicyInventoryEntry] {
        &self.aspect_policy_inventory
    }

    pub fn runtime_artifact_carry_policies(&self) -> &[RuntimeArtifactCarryPolicy] {
        &self.runtime_artifact_carry_policies
    }

    pub fn retained_artifact_carry_policies(&self) -> &[RetainedArtifactCarryPolicy] {
        &self.retained_artifact_carry_policies
    }

    pub fn causality_carry_policies(&self) -> &[CausalityCarryPolicy] {
        &self.causality_carry_policies
    }
}

pub(crate) fn aspect_policy_inventory(
    aspect_policy_plan: &super::plan::LoweredAspectMergePolicyPlan,
) -> Vec<SignalAspectPolicyInventoryEntry> {
    let mut inventory = aspect_policy_plan
        .records
        .iter()
        .map(|record| {
            SignalAspectPolicyInventoryEntry::new(
                record.selected_policy_name.clone(),
                record.selected_policy_digest.clone(),
                record.selected_policy_basis,
            )
        })
        .collect::<Vec<_>>();
    inventory.sort_by(|left, right| {
        left.policy_name()
            .as_str()
            .cmp(right.policy_name().as_str())
            .then_with(|| left.policy_digest().cmp(right.policy_digest()))
    });
    inventory.dedup_by(|left, right| {
        left.policy_name() == right.policy_name()
            && left.policy_digest() == right.policy_digest()
            && left.policy_basis() == right.policy_basis()
    });
    inventory
}

pub(crate) fn unique_runtime_artifact_carry_policies(
    adoption_policy: &[SourceNodeAdoptionCarryPolicy],
) -> Vec<RuntimeArtifactCarryPolicy> {
    let mut inventory = adoption_policy
        .iter()
        .map(|policy| policy.runtime_artifact)
        .collect::<Vec<_>>();
    inventory.sort();
    inventory.dedup();
    inventory
}

pub(crate) fn unique_retained_artifact_carry_policies(
    adoption_policy: &[SourceNodeAdoptionCarryPolicy],
) -> Vec<RetainedArtifactCarryPolicy> {
    let mut inventory = adoption_policy
        .iter()
        .map(|policy| policy.retained_artifact)
        .collect::<Vec<_>>();
    inventory.sort();
    inventory.dedup();
    inventory
}

pub(crate) fn unique_causality_carry_policies(
    adoption_policy: &[SourceNodeAdoptionCarryPolicy],
) -> Vec<CausalityCarryPolicy> {
    let mut inventory = adoption_policy
        .iter()
        .map(|policy| policy.causality)
        .collect::<Vec<_>>();
    inventory.sort();
    inventory.dedup();
    inventory
}
