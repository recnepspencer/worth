use super::InvariantPreparationControl;
use crate::transactions::data::MergedCommitPlan;
use crate::validation::data::InvariantPlanContract;
use std::collections::BTreeMap;

use super::relation_integrity_scopes::prepare_relation_integrity_scopes;
use super::InvariantExecutionRequest;
use crate::validation::engine::{InvariantObservation, InvariantRequestProfile};

impl<'state> InvariantExecutionRequest<'state> {
    #[cfg(test)]
    pub(crate) fn from_profile_with_contract<'runtime>(
        profile: InvariantRequestProfile,
        runtime: &'runtime crate::runtime::RelationalRuntime,
        observation: InvariantObservation<'state>,
        version_id: crate::identity::data::VersionId,
        merged_plan: Option<&'state MergedCommitPlan>,
        plan_contract: Option<InvariantPlanContract>,
    ) -> Self
    where
        'runtime: 'state,
    {
        let view = crate::validation::engine::InvariantRuntimeView::from_runtime(runtime);
        Self::from_profile_with_contract_at_current_version(
            profile,
            &view,
            observation,
            version_id,
            view.current_version_id(),
            merged_plan,
            plan_contract,
            &InvariantPreparationControl::new(
                crate::mvcc::RelationalOperationControl::uninterrupted(),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                crate::mvcc::RelationalInterruptionBoundary::ProposalValidation,
            ),
        )
        .expect("uninterrupted System scope preparation")
    }

    pub(crate) fn from_profile_with_contract_at_current_version<'runtime>(
        profile: InvariantRequestProfile,
        runtime: &crate::validation::engine::InvariantRuntimeView<'runtime>,
        observation: InvariantObservation<'state>,
        version_id: crate::identity::data::VersionId,
        current_version_id: crate::identity::data::VersionId,
        merged_plan: Option<&'state MergedCommitPlan>,
        plan_contract: Option<InvariantPlanContract>,
        control: &InvariantPreparationControl<'_, '_>,
    ) -> Result<Self, crate::transactions::data::TransactionCommitError> {
        control.check()?;
        debug_assert!(
            profile.supports_observation(observation.kind()),
            "invariant profile {:?} does not support {:?} observation",
            profile,
            observation.kind(),
        );
        let runtime_policy = crate::validation::engine::policy::RelationalInvariantRuntime::resolve(
            profile,
            crate::validation::engine::policy::derive_invariant_context(runtime),
        );
        let consumed_groups = profile.consumed_groups();
        let applicable_groups = plan_contract
            .map(|contract| contract.selected_groups().intersection(consumed_groups))
            .unwrap_or(consumed_groups);
        let relation_scope_requirements =
            relation_scope_requirements_for(runtime, profile, control)?;
        let relation_integrity_scopes = prepare_relation_integrity_scopes(
            merged_plan,
            observation.committed_partition_access(),
            version_id,
            relation_scope_requirements,
            &runtime.performance_access(),
            control,
        )?;
        let proposal_identity = observation.proposal_identity().cloned();
        Ok(Self {
            observation,
            version_id,
            current_version_id,
            checkpoint: profile.execution_point(),
            runtime_policy,
            consumed_groups,
            applicable_groups,
            plan_contract,
            merged_plan,
            relation_integrity_scopes,
            proposal_identity,
        })
    }
}

fn relation_scope_requirements_for(
    runtime: &crate::validation::engine::InvariantRuntimeView,
    profile: InvariantRequestProfile,
    control: &InvariantPreparationControl<'_, '_>,
) -> Result<
    BTreeMap<crate::identity::data::KindId, super::RelationScopeRequirement>,
    crate::transactions::data::TransactionCommitError,
> {
    runtime
        .config
        .schema
        .invariant_catalog
        .registrations_for_execution_point(profile.execution_point())
        .chain(
            runtime
                .schema_contract_runtime
                .relation_integrity_registrations
                .iter()
                .filter(|registration| registration.execution_point == profile.execution_point()),
        )
        .try_fold(BTreeMap::new(), |mut requirements, registration| {
            control.check()?;
            let Some((kind_id, requirement)) =
                super::relation_scope_requirement(&registration.rule)
            else {
                return Ok(requirements);
            };
            let entry = requirements
                .entry(kind_id)
                .or_insert_with(super::RelationScopeRequirement::default);
            entry.requires_global_evaluation |= requirement.requires_global_evaluation;
            entry.requires_visible_successors |= requirement.requires_visible_successors;
            entry
                .minimum_candidate_kinds
                .extend(requirement.minimum_candidate_kinds);
            entry.scan_existing_source_on_create |= requirement.scan_existing_source_on_create;
            entry.scan_existing_target_on_create |= requirement.scan_existing_target_on_create;
            entry.scan_existing_pair_on_create |= requirement.scan_existing_pair_on_create;
            entry.scan_full_on_create |= requirement.scan_full_on_create;
            Ok(requirements)
        })
}
