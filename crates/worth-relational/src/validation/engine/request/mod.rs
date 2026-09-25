mod admission;
mod construction;
mod filtering;
mod relation_integrity_scopes;
mod scope_types;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;

use crate::transactions::data::MergedCommitPlan;
use crate::validation::data::{
    InvariantCostClass, InvariantExecutionPoint, InvariantGroupSet, InvariantPlanContract,
    InvariantRegistration, InvariantViolation,
};
use crate::{
    identity::data::KindId,
    validation::data::{CustomInvariantRegistration, InvariantRule},
};

use super::observation::InvariantObservation;
use super::policy::RelationalInvariantRuntime;
pub(crate) use admission::InvariantRegistrationAdmission;
pub(crate) use scope_types::{
    PlannedRelationEdge, PreparedRelationEndpointKey, PreparedRelationIntegrityScope,
    PreparedRelationIntegrityScopes, PreparedRelationPairKey, PreparedVisibleRelationEdge,
};

#[derive(Debug, Clone, Default)]
struct RelationScopeRequirement {
    requires_global_evaluation: bool,
    requires_visible_successors: bool,
    minimum_candidate_kinds: std::collections::BTreeSet<KindId>,
    scan_existing_source_on_create: bool,
    scan_existing_target_on_create: bool,
    scan_existing_pair_on_create: bool,
    scan_full_on_create: bool,
}

pub(crate) struct InvariantExecutionRequest<'runtime> {
    observation: InvariantObservation<'runtime>,
    version_id: crate::identity::data::VersionId,
    current_version_id: crate::identity::data::VersionId,
    checkpoint: InvariantExecutionPoint,
    runtime_policy: RelationalInvariantRuntime,
    consumed_groups: InvariantGroupSet,
    applicable_groups: InvariantGroupSet,
    plan_contract: Option<InvariantPlanContract>,
    merged_plan: Option<&'runtime MergedCommitPlan>,
    relation_integrity_scopes: Option<PreparedRelationIntegrityScopes>,
    preparation_violation: Option<InvariantViolation>,
    proposal_identity: Option<crate::mvcc::RelationalMutationProposalIdentity>,
}

impl<'runtime> InvariantExecutionRequest<'runtime> {
    pub(crate) fn observation(&self) -> &InvariantObservation<'runtime> {
        &self.observation
    }

    pub(crate) fn version_id(&self) -> crate::identity::data::VersionId {
        self.version_id
    }

    pub(crate) fn execution_point(&self) -> InvariantExecutionPoint {
        self.checkpoint
    }

    pub(crate) fn current_version_id(&self) -> crate::identity::data::VersionId {
        self.current_version_id
    }

    pub(crate) fn merged_plan(&self) -> Option<&'runtime MergedCommitPlan> {
        self.merged_plan
    }

    pub(crate) fn consumed_groups(&self) -> InvariantGroupSet {
        self.consumed_groups
    }

    pub(crate) fn applicable_groups(&self) -> InvariantGroupSet {
        self.applicable_groups
    }

    pub(crate) fn plan_contract(&self) -> Option<InvariantPlanContract> {
        self.plan_contract
    }

    pub(crate) fn max_cost(&self) -> InvariantCostClass {
        self.runtime_policy.max_cost_at(self.checkpoint)
    }

    pub(crate) fn relation_integrity_scopes(&self) -> Option<&PreparedRelationIntegrityScopes> {
        self.relation_integrity_scopes.as_ref()
    }

    pub(crate) fn preparation_violation(&self) -> Option<&InvariantViolation> {
        self.preparation_violation.as_ref()
    }

    pub(crate) fn proposal_identity(
        &self,
    ) -> Option<&crate::mvcc::RelationalMutationProposalIdentity> {
        self.proposal_identity.as_ref()
    }

    pub(crate) fn should_execute_anything(&self) -> bool {
        self.merged_plan.is_none() || !self.applicable_groups.is_empty()
    }

    pub(crate) fn includes_registration(&self, registration: &InvariantRegistration) -> bool {
        let rule_groups = registration.groups();
        self.runtime_policy.should_run(rule_groups, self.checkpoint)
            && (self.applicable_groups.is_empty() || self.applicable_groups.intersects(rule_groups))
            && self
                .plan_contract
                .is_none_or(|contract| contract.applies_to_rule(&registration.rule))
            && filtering::rule_matches_plan_scope(self, &registration.rule)
            && self.registration_admission(registration).is_admitted()
    }

    pub(crate) fn includes_custom_registration(
        &self,
        registration: &CustomInvariantRegistration,
    ) -> bool {
        let rule_groups = registration.groups();
        registration.execution_point() == self.checkpoint
            && self.runtime_policy.should_run(rule_groups, self.checkpoint)
            && (self.applicable_groups.is_empty() || self.applicable_groups.intersects(rule_groups))
            && admission::for_failure_effect(
                &self.runtime_policy,
                self.checkpoint,
                registration.failure_effect(),
                registration.cost_class(),
            )
            .is_admitted()
    }

    pub(crate) fn registration_admission(
        &self,
        registration: &InvariantRegistration,
    ) -> InvariantRegistrationAdmission {
        admission::for_failure_effect(
            &self.runtime_policy,
            self.checkpoint,
            registration.failure_effect,
            registration.cost(),
        )
    }

    #[cfg(test)]
    pub(crate) fn with_applicable_groups(mut self, applicable_groups: InvariantGroupSet) -> Self {
        self.applicable_groups = applicable_groups;
        self
    }
}

pub(crate) fn relation_kind_scope(rule: &InvariantRule) -> Option<KindId> {
    match rule {
        InvariantRule::EndpointKindContract(contract) => Some(contract.relation_kind_id),
        InvariantRule::CardinalityMaximumContract(contract) => Some(contract.relation_kind_id),
        InvariantRule::CardinalityMinimumContract(contract) => Some(contract.relation_kind_id),
        InvariantRule::UniquenessContract(contract) => Some(contract.relation_kind_id),
        InvariantRule::SymmetryContract(contract) => Some(contract.relation_kind_id),
        InvariantRule::EndpointDeletionIntegrityContract(contract) => {
            Some(contract.relation_kind_id)
        }
        InvariantRule::AcyclicityContract(contract) => Some(contract.relation_kind_id),
        InvariantRule::PartitionIsolationContract(contract) => Some(contract.relation_kind_id),
        InvariantRule::ConnectivityMinimumContract(contract) => Some(contract.relation_kind_id),
        _ => None,
    }
}

fn relation_scope_requirement(rule: &InvariantRule) -> Option<(KindId, RelationScopeRequirement)> {
    let relation_kind_id = relation_kind_scope(rule)?;
    let requirement = match rule {
        InvariantRule::CardinalityMinimumContract(contract) => RelationScopeRequirement {
            minimum_candidate_kinds: contract
                .candidate_source_kinds
                .iter()
                .chain(&contract.candidate_target_kinds)
                .copied()
                .collect(),
            requires_visible_successors: false,
            requires_global_evaluation: false,
            scan_existing_source_on_create: contract.source_min.is_some_and(|minimum| minimum > 1),
            scan_existing_target_on_create: contract.target_min.is_some_and(|minimum| minimum > 1),
            scan_existing_pair_on_create: contract.pair_min.is_some(),
            scan_full_on_create: false,
        },
        InvariantRule::CardinalityMaximumContract(contract) => RelationScopeRequirement {
            scan_existing_source_on_create: contract.source_max.is_some(),
            scan_existing_target_on_create: contract.target_max.is_some(),
            scan_existing_pair_on_create: contract.pair_max.is_some(),
            ..RelationScopeRequirement::default()
        },
        InvariantRule::UniquenessContract(_) | InvariantRule::SymmetryContract(_) => {
            RelationScopeRequirement {
                scan_existing_pair_on_create: true,
                scan_full_on_create: true,
                ..RelationScopeRequirement::default()
            }
        }
        InvariantRule::AcyclicityContract(_) => RelationScopeRequirement {
            requires_global_evaluation: false,
            requires_visible_successors: true,
            scan_existing_source_on_create: true,
            scan_existing_target_on_create: true,
            scan_full_on_create: true,
            ..RelationScopeRequirement::default()
        },
        InvariantRule::ConnectivityMinimumContract(_) => RelationScopeRequirement {
            requires_global_evaluation: true,
            requires_visible_successors: true,
            scan_existing_source_on_create: true,
            scan_existing_target_on_create: true,
            scan_full_on_create: true,
            ..RelationScopeRequirement::default()
        },
        _ => RelationScopeRequirement::default(),
    };
    Some((relation_kind_id, requirement))
}
