use worth_relational_contracts::internal_port::validation::data::catalog::{
    InvariantRegistration, InvariantRegistrationOwnerPort as _,
};
use worth_relational_contracts::internal_port::validation::data::execution::{
    InvariantExecutionPoint, InvariantExecutionPointOwnerPort as _,
};
pub mod admission;
pub mod construction;
pub mod filtering;
pub mod preparation_control;
pub mod relation_integrity_scopes;
pub mod relation_law_sources;
pub mod scope_admission;
pub mod scope_types;
#[cfg(test)]
pub mod test_support;
#[cfg(test)]
pub mod tests;

use crate::authority::commit::semantic_plan::meaning::RelationalSemanticPlan;
use crate::validation::custom_rule::CustomInvariantRegistration;
use worth_relational_contracts::internal_port::identity::data::primitives::KindId;
use worth_relational_contracts::internal_port::validation::data::contracts::InvariantPlanContract;
use worth_relational_contracts::internal_port::validation::data::groups::{
    InvariantCostClass, InvariantGroupSet,
};
use worth_relational_contracts::internal_port::validation::data::rules::InvariantRule;

use super::observation::InvariantObservation;
use super::policy::RelationalInvariantRuntime;
pub(crate) use admission::InvariantRegistrationAdmission;
pub use preparation_control::{InvariantPreparationControl, InvariantPreparationSource};
pub(crate) use scope_types::PlannedRelationEdge;
pub(crate) use scope_types::PreparedRelationEndpointKey;
pub(crate) use scope_types::PreparedRelationIntegrityScope;
pub use scope_types::PreparedRelationIntegrityScopes;
pub(crate) use scope_types::PreparedRelationPairKey;
pub(crate) use scope_types::PreparedVisibleRelationEdge;

#[derive(Debug, Clone, Default)]
struct RelationScopeRequirement {
    requires_affected_pairs: bool,
    requires_global_evaluation: bool,
    requires_visible_successors: bool,
    minimum_candidate_kinds: std::collections::BTreeSet<KindId>,
    scan_existing_source_on_create: bool,
    scan_existing_target_on_create: bool,
    scan_existing_pair_on_create: bool,
    scan_full_on_create: bool,
}

pub struct InvariantExecutionRequest<'runtime> {
    observation: InvariantObservation<'runtime>,
    applied_declaration_view: Option<InvariantObservation<'runtime>>,
    version_id: worth_relational_contracts::internal_port::identity::data::primitives::VersionId,
    current_version_id:
        worth_relational_contracts::internal_port::identity::data::primitives::VersionId,
    checkpoint: InvariantExecutionPoint,
    runtime_policy: RelationalInvariantRuntime,
    consumed_groups: InvariantGroupSet,
    applicable_groups: InvariantGroupSet,
    plan_contract: Option<InvariantPlanContract>,
    semantic_plan: Option<RelationalSemanticPlan<'runtime>>,
    relation_integrity_scopes: Option<PreparedRelationIntegrityScopes>,
    proposal_identity:
        Option<crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity>,
}

impl<'runtime> InvariantExecutionRequest<'runtime> {
    pub(crate) fn observation(&self) -> &InvariantObservation<'runtime> {
        &self.observation
    }

    pub(crate) fn observation_for(
        &self,
        point: InvariantExecutionPoint,
    ) -> &InvariantObservation<'runtime> {
        if matches!(
            point,
            InvariantExecutionPoint::MutationSensitive
                | InvariantExecutionPoint::SnapshotPublication
        ) {
            self.applied_declaration_view
                .as_ref()
                .unwrap_or(&self.observation)
        } else {
            &self.observation
        }
    }
    pub(crate) fn version_for(
        &self,
        point: InvariantExecutionPoint,
    ) -> worth_relational_contracts::internal_port::identity::data::primitives::VersionId {
        if self.checkpoint == InvariantExecutionPoint::ProposedState
            && point == InvariantExecutionPoint::CommitBoundary
        {
            self.current_version_id
        } else {
            self.version_id
        }
    }
    pub(crate) fn version_id(
        &self,
    ) -> worth_relational_contracts::internal_port::identity::data::primitives::VersionId {
        self.version_id
    }

    pub(crate) fn execution_point(&self) -> InvariantExecutionPoint {
        self.checkpoint
    }

    pub(crate) fn current_version_id(
        &self,
    ) -> worth_relational_contracts::internal_port::identity::data::primitives::VersionId {
        self.current_version_id
    }

    pub(crate) fn semantic_plan(&self) -> Option<RelationalSemanticPlan<'runtime>> {
        self.semantic_plan
    }

    pub(crate) fn consumed_groups(&self) -> InvariantGroupSet {
        self.consumed_groups
    }

    pub fn applicable_groups(&self) -> InvariantGroupSet {
        self.applicable_groups
    }

    pub(crate) fn plan_contract(&self) -> Option<InvariantPlanContract> {
        self.plan_contract
    }

    pub fn max_cost(&self) -> InvariantCostClass {
        self.runtime_policy.max_cost_at(self.checkpoint)
    }

    pub(crate) fn relation_integrity_scopes(&self) -> Option<&PreparedRelationIntegrityScopes> {
        self.relation_integrity_scopes.as_ref()
    }

    pub fn proposal_identity(
        &self,
    ) -> Option<&crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity>
    {
        self.proposal_identity.as_ref()
    }

    pub fn should_execute_anything(&self) -> bool {
        self.proposal_identity
            .as_ref()
            .is_some_and(|identity| identity.omitted_publication_appendix())
            || self.semantic_plan.is_none()
            || !self.applicable_groups.is_empty()
    }

    fn includes_family_groups(
        &self,
        point: InvariantExecutionPoint,
        groups: InvariantGroupSet,
    ) -> bool {
        if self.checkpoint != InvariantExecutionPoint::ProposedState {
            return true;
        }
        use super::InvariantRequestProfile as Profile;
        let consumed = match point {
            InvariantExecutionPoint::CommitBoundary => Profile::CommitBoundary.consumed_groups(),
            InvariantExecutionPoint::MutationSensitive => {
                Profile::MutationSensitive.consumed_groups()
            }
            InvariantExecutionPoint::SnapshotPublication => {
                Profile::SnapshotPublication.consumed_groups()
            }
            _ => return false,
        };
        let applicable = self.plan_contract.map_or(consumed, |plan| {
            plan.selected_groups().intersection(consumed)
        });
        consumed.intersects(groups) && applicable.intersects(groups)
    }

    pub(crate) fn includes_registration(&self, registration: &InvariantRegistration) -> bool {
        let rule_groups = registration.groups();
        self.includes_family_groups(registration.execution_point, rule_groups)
            && self.runtime_policy.should_run(rule_groups, self.checkpoint)
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
        // Omitting the publication appendix also removes its plan groups. The
        // real custom owner must still establish execution/non-applicability
        // for this exact proposal; Query cannot manufacture a missing receipt.
        let omitted_appendix = self
            .proposal_identity
            .as_ref()
            .is_some_and(|identity| identity.omitted_publication_appendix());
        self.checkpoint.includes(registration.execution_point())
            && (omitted_appendix
                || self.includes_family_groups(registration.execution_point(), rule_groups))
            && self.runtime_policy.should_run(rule_groups, self.checkpoint)
            && (omitted_appendix
                || self.applicable_groups.is_empty()
                || self.applicable_groups.intersects(rule_groups))
            && admission::for_failure_effect(
                &self.runtime_policy,
                registration.execution_point(),
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
            registration.execution_point,
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
            requires_affected_pairs: false,
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
        InvariantRule::SymmetryContract(contract) => RelationScopeRequirement {
            requires_affected_pairs: matches!(
                contract.mode,
                worth_relational_contracts::internal_port::schema::data::relation_integrity::declarations::SymmetryMode::PairedInverseRequired
                    | worth_relational_contracts::internal_port::schema::data::relation_integrity::declarations::SymmetryMode::PairedTwinRequired
            ),
            scan_existing_pair_on_create: true,
            scan_full_on_create: true,
            ..RelationScopeRequirement::default()
        },
        InvariantRule::UniquenessContract(_) => RelationScopeRequirement {
            scan_existing_pair_on_create: true,
            scan_full_on_create: true,
            ..RelationScopeRequirement::default()
        },
        InvariantRule::AcyclicityContract(_) => RelationScopeRequirement {
            requires_global_evaluation: false,
            requires_visible_successors: true,
            scan_existing_source_on_create: true,
            scan_existing_target_on_create: true,
            scan_full_on_create: true,
            ..RelationScopeRequirement::default()
        },
        InvariantRule::ConnectivityMinimumContract(contract) => RelationScopeRequirement {
            minimum_candidate_kinds: contract.source_kind_ids.iter().copied().collect(),
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
