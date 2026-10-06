use super::boundary_validation::BoundaryValidatedCommitExecution;

mod phase;

use phase::{run_authoritative_mutation_phase, MutationPhaseInput};

pub(super) struct MutatedCommitExecution {
    validated: BoundaryValidatedCommitExecution,
    version_id: crate::identity::data::VersionId,
    effect: crate::authority::mutation::MutationEffect,
    created_entities: crate::transactions::data::CommitCreatedEntityBindings,
    created_relations: crate::transactions::data::CommitCreatedRelationBindings,
    record_allocations: crate::runtime::PendingRecordAllocations,
}

impl MutatedCommitExecution {
    pub(super) fn validated_mut(&mut self) -> &mut BoundaryValidatedCommitExecution {
        &mut self.validated
    }

    pub(super) fn version_id(&self) -> crate::identity::data::VersionId {
        self.version_id
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        BoundaryValidatedCommitExecution,
        crate::identity::data::VersionId,
        crate::authority::mutation::MutationEffect,
        crate::transactions::data::CommitCreatedEntityBindings,
        crate::transactions::data::CommitCreatedRelationBindings,
        crate::runtime::PendingRecordAllocations,
    ) {
        (
            self.validated,
            self.version_id,
            self.effect,
            self.created_entities,
            self.created_relations,
            self.record_allocations,
        )
    }
}

pub(super) fn mutate_commit_execution(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    mut validated: BoundaryValidatedCommitExecution,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
) -> Result<MutatedCommitExecution, crate::transactions::data::TransactionCommitError> {
    let selected_branch_state = validated.prepared_mut().selected_branch_state().clone();
    let proposed_version_id = validated.prepared_mut().proposed_version_id();
    let proposal_identity = validated.prepared_mut().proposal_identity().clone();
    let prevalidated_mutation_sensitive = validated
        .prepared_mut()
        .admitted_mut()
        .take_prevalidated_mutation_sensitive();
    let (admitted, working_state) = validated.prepared_mut().mutation_parts();
    let (transaction_id, validation_input, merged_plan, _, commit_log, phase_timing) =
        admitted.phase_view().into_parts();
    let mutation = run_authoritative_mutation_phase(
        runtime,
        MutationPhaseInput {
            commit_log,
            phase_timing,
            transaction_id,
            working_state,
            merged_plan,
            schema_authority: validation_input.schema_authority(),
            selected_branch_state: &selected_branch_state,
            proposed_version_id,
            proposal_identity: &proposal_identity,
            prevalidated_mutation_sensitive,
            lease,
        },
    )?;
    let (
        version_id,
        effect,
        invariant_results,
        created_entities,
        created_relations,
        record_allocations,
    ) = mutation.into_parts();
    validated.push_invariant(invariant_results);
    Ok(MutatedCommitExecution {
        validated,
        version_id,
        effect,
        created_entities,
        created_relations,
        record_allocations,
    })
}
