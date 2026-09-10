mod retained_charge;

use serde::Serialize;
use worth_proof::TransitionOutcome;

use super::core::{BranchMergeStrategy, MergeBoundaryWitness, MergeBoundaryWitnessKind};
use super::semantics::SelectedMergeSemanticsBundle;
use super::strategy_identity::{
    unique_causality_carry_policies, unique_retained_artifact_carry_policies,
    unique_runtime_artifact_carry_policies, SignalDeliveryStrategyIdentity,
    SignalInvalidationStrategyIdentity, SignalMergeStrategyIdentity,
};
use super::strategy_witness_denial::{
    canonical_digest, SignalMergeStrategyWitnessDenial, SignalMergeStrategyWitnessDenialKind,
};
use super::{SignalAspectPolicyInventoryEntry, SourceNodeAdoptionCarryPolicy};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SignalMergeStrategyWitness {
    merge_strategy: SignalMergeStrategyIdentity,
    invalidation_strategy: SignalInvalidationStrategyIdentity,
    delivery_strategy: SignalDeliveryStrategyIdentity,
    merge_strategy_digest: String,
    invalidation_strategy_digest: String,
    delivery_strategy_digest: String,
    witness_digest: String,
}

impl SignalMergeStrategyWitness {
    pub(crate) fn try_from_identities(
        merge_strategy: Option<SignalMergeStrategyIdentity>,
        invalidation_strategy: Option<SignalInvalidationStrategyIdentity>,
        delivery_strategy: Option<SignalDeliveryStrategyIdentity>,
    ) -> TransitionOutcome<Self, SignalMergeStrategyWitnessDenial> {
        let Some(merge_strategy) = merge_strategy else {
            return TransitionOutcome::Denied(SignalMergeStrategyWitnessDenial::new(
                SignalMergeStrategyWitnessDenialKind::MissingMergeStrategyIdentity,
                "strategy witness requires merge strategy identity",
            ));
        };
        let Some(invalidation_strategy) = invalidation_strategy else {
            return TransitionOutcome::Denied(SignalMergeStrategyWitnessDenial::new(
                SignalMergeStrategyWitnessDenialKind::MissingInvalidationStrategyIdentity,
                "strategy witness requires invalidation strategy identity",
            ));
        };
        let Some(delivery_strategy) = delivery_strategy else {
            return TransitionOutcome::Denied(SignalMergeStrategyWitnessDenial::new(
                SignalMergeStrategyWitnessDenialKind::MissingDeliveryStrategyIdentity,
                "strategy witness requires delivery strategy identity",
            ));
        };

        let merge_strategy_digest = canonical_digest(&merge_strategy);
        let invalidation_strategy_digest = canonical_digest(&invalidation_strategy);
        let delivery_strategy_digest = canonical_digest(&delivery_strategy);
        let witness_digest = canonical_digest(&(
            &merge_strategy_digest,
            &invalidation_strategy_digest,
            &delivery_strategy_digest,
        ));

        TransitionOutcome::success(Self {
            merge_strategy,
            invalidation_strategy,
            delivery_strategy,
            merge_strategy_digest,
            invalidation_strategy_digest,
            delivery_strategy_digest,
            witness_digest,
        })
    }

    pub(crate) fn from_admitted_plan_components(
        selected_semantics: &SelectedMergeSemanticsBundle,
        merge_strategy: BranchMergeStrategy,
        lowered_strategy_bundle_digest: &str,
        boundary_witness: &MergeBoundaryWitness,
        aspect_policy_inventory: Vec<SignalAspectPolicyInventoryEntry>,
        adoption_policy: &[SourceNodeAdoptionCarryPolicy],
    ) -> Self {
        let merge_strategy = SignalMergeStrategyIdentity::new(
            merge_strategy,
            selected_semantics.strategy_name.clone(),
            selected_semantics.strategy_digest.clone(),
            selected_semantics.strategy_basis,
            selected_semantics.merge_base_name.clone(),
            selected_semantics.merge_base_digest.clone(),
            selected_semantics.merge_base_basis,
            lowered_strategy_bundle_digest.to_owned(),
        )
        .expect("admitted merge strategy identity should be complete");
        let invalidation_strategy = SignalInvalidationStrategyIdentity::new(
            boundary_witness_kind(boundary_witness),
            selected_semantics.conflict_isolation_name.clone(),
            selected_semantics.conflict_isolation_digest.clone(),
            selected_semantics.conflict_isolation_basis,
            selected_semantics.identity_matcher_name.clone(),
            selected_semantics.identity_matcher_digest.clone(),
            selected_semantics.identity_matcher_basis,
        )
        .expect("admitted invalidation strategy identity should be complete");
        let delivery_strategy = SignalDeliveryStrategyIdentity::new(
            selected_semantics.conflict_policy_name.clone(),
            selected_semantics.conflict_policy_digest.clone(),
            selected_semantics.conflict_policy_basis,
            selected_semantics.source_only_policy_name.clone(),
            selected_semantics.source_only_policy_digest.clone(),
            selected_semantics.source_only_policy_basis,
            selected_semantics.deletion_policy_name.clone(),
            selected_semantics.deletion_policy_digest.clone(),
            selected_semantics.deletion_policy_basis,
            aspect_policy_inventory,
            unique_runtime_artifact_carry_policies(adoption_policy),
            unique_retained_artifact_carry_policies(adoption_policy),
            unique_causality_carry_policies(adoption_policy),
        )
        .expect("admitted delivery strategy identity should be complete");

        match Self::try_from_identities(
            Some(merge_strategy),
            Some(invalidation_strategy),
            Some(delivery_strategy),
        ) {
            TransitionOutcome::Success(witness) => witness,
            outcome => unreachable!("admitted plan strategy witness should not deny: {outcome:?}"),
        }
    }

    pub fn merge_strategy(&self) -> &SignalMergeStrategyIdentity {
        &self.merge_strategy
    }

    pub fn invalidation_strategy(&self) -> &SignalInvalidationStrategyIdentity {
        &self.invalidation_strategy
    }

    pub fn delivery_strategy(&self) -> &SignalDeliveryStrategyIdentity {
        &self.delivery_strategy
    }

    pub fn merge_strategy_digest(&self) -> &str {
        self.merge_strategy_digest.as_str()
    }

    pub fn invalidation_strategy_digest(&self) -> &str {
        self.invalidation_strategy_digest.as_str()
    }

    pub fn delivery_strategy_digest(&self) -> &str {
        self.delivery_strategy_digest.as_str()
    }

    pub fn witness_digest(&self) -> &str {
        self.witness_digest.as_str()
    }
}

fn boundary_witness_kind(boundary_witness: &MergeBoundaryWitness) -> MergeBoundaryWitnessKind {
    boundary_witness.kind
}
