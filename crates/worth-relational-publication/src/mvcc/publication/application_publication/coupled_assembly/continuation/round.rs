//! Apply a continued round to a discarded image and judge its actual effects.
use worth_relational_contracts::internal_port as contracts;
use super::super::{assembly_failure::RelationalCoupledAssemblyFailure, writer_custody};
use super::application;
use crate::authority::commit::phases::prepare::{
    prepare_authoritative_working_state_scope_for_base, working_state_inputs::WorkingStatePreparation,
};
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority;
use worth_relational_branch::internal_port::history::retention::VisibilitySnapshotBasis;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::CanonicalRequiredOutputOrigin;
use contracts::config::data::{policies::MutationConfig, RelationalRuntimeConfig};
use contracts::execution::{
    request_work::RequestWorkBudget,
};
use contracts::identity::data::primitives::VersionId;
use contracts::transactions::data::outcomes::plan_artifacts::{
    AuthoritativeApplyPlan, MergedCommitPlan,
};
use worth_relational_mutation::internal_port::aggregate_identity::guard;
use worth_relational_mutation::internal_port::authority::mutation::{
    execution::apply_plan_with_publication_edges,
    pending_identities::PendingMutationIdentities, BranchLocalDeleteAllowance,
};
use worth_relational_mutation::internal_port::validation::engine::state_view::InvariantStateView;

use super::super::super::RelationalAuthoredCoupledOwners;

pub fn verify(
    execution: &ExecutionRequest<'_, '_>,
    // The pinned source: its read residency lends the base partitions.
    root: &VisibilitySnapshotBasis,
    plan: &MergedCommitPlan,
    owners: &RelationalAuthoredCoupledOwners,
    predecessor: Option<&CanonicalRequiredOutputOrigin>,
    config: &RelationalRuntimeConfig,
    working_state: WorkingStatePreparation<'_>,
    work: Option<&RequestWorkBudget>,
    schema: &RelationalBranchRootSchemaAuthority,
    version: VersionId,
    identities: &mut PendingMutationIdentities,
) -> Result<bool, RelationalCoupledAssemblyFailure> {
    let deferred_edges = writer_custody::require(
        execution,
        owners,
        writer_custody::Writer::Producer(predecessor),
        plan,
    )?;
    let (_, mut state, _) =
        prepare_authoritative_working_state_scope_for_base(working_state, root, plan, None);
    guard::ordinary_plan(
        execution,
        work,
        schema.registry(),
        &InvariantStateView::new(root, version),
        &plan.merged_intents,
        None,
    )?;
    let mut symbols = identities
        .symbols
        .discardable_copy()
        .map_err(|cause| RelationalCoupledAssemblyFailure::from_apply(cause.into()))?;
    let reserved = symbols.reserved_meaning_count();
    let config = MutationConfig {
        cascade_delete_policy: config.storage.cascade_delete_policy,
        adjacency_policy: config.storage.adjacency_policy.clone(),
        cross_context_policy: config.storage.cross_context_policy,
    };
    let apply = AuthoritativeApplyPlan {
        transaction_id: plan.transaction_id,
        version_id: version,
        merged_intents: plan.merged_intents.clone(),
    };
    let outcome = apply_plan_with_publication_edges(
        execution,
        &mut state,
        &apply,
        &config,
        schema.registry(),
        schema.aspect_plans(),
        &mut symbols,
        BranchLocalDeleteAllowance::default(),
        &mut identities.records,
        work.cloned(),
        Some(&deferred_edges),
    )
    .map_err(RelationalCoupledAssemblyFailure::from_apply)?;
    Ok(application::preserves_applied_records(
        execution,
        &outcome,
        reserved,
        symbols.reserved_meaning_count(),
    )?)
}
