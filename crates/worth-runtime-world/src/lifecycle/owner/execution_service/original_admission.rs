//! The original capture is paid before the attempt's first owner effect.
use crate::history::MemoryOriginalSuccessor;
use crate::publication::{
    NoEffectCause, RelationalComponentPlanPosture, ReservedCompositePublicationAttempt,
};
use worth_relational::facade::branch::{
    RelationalBranchBasisDenial, RelationalRecordedComponentRefusal as Refusal,
};

use super::RuntimeWorldOwnerRoot;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Admit the memory original this attempt's publication will seal: its
    /// retained memory, copy work, before component, successor quote and
    /// registry slot. A request that cannot pay it stops here with no effect,
    /// and the seal after the owner effects charges nothing.
    pub(super) fn admit_memory_original(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        attempt: &ReservedCompositePublicationAttempt,
    ) -> Result<(), NoEffectCause> {
        if attempt.has_durable_open() {
            return Ok(());
        }
        let Some(envelope) = attempt.original_publication_envelope() else {
            return Ok(());
        };
        let relational = attempt.plan().relational();
        let successor = match relational.posture() {
            RelationalComponentPlanPosture::RetainExact => return Ok(()),
            RelationalComponentPlanPosture::PublishPrepared => {
                let Some(candidate) = relational.prepared_candidate() else {
                    return Ok(());
                };
                MemoryOriginalSuccessor::Candidate(candidate)
            }
            RelationalComponentPlanPosture::AdoptSettled => {
                let Some(adoption) = relational.settled_adoption() else {
                    return Ok(());
                };
                // Adopting the already selected Native original creates no
                // second World origin for the same Native effect.
                let native = adoption.commit_identity().commit_id().0;
                if attempt
                    .expected_head()
                    .basis()
                    .relational_basis()
                    .reference()
                    .target()
                    .as_basis()
                    .is_some_and(|prior| prior.selected_commit_id() == native)
                {
                    return Ok(());
                }
                MemoryOriginalSuccessor::Issued(adoption.successor_basis())
            }
        };
        envelope
            .admit_memory_original(
                execution,
                &self.state.history,
                &self.state.relational_services().basis_port(),
                attempt.expected_head(),
                successor,
            )
            .map_err(no_effect)
    }
}

fn no_effect(refusal: Refusal) -> NoEffectCause {
    match refusal {
        Refusal::Request(cause) => cause.as_request_denial().map_or(
            NoEffectCause::PreEffectFailure,
            NoEffectCause::ExecutionRequest,
        ),
        Refusal::Basis(RelationalBranchBasisDenial::OwnerUnavailable) => {
            NoEffectCause::OwnerUnavailable
        }
        Refusal::Basis(
            RelationalBranchBasisDenial::ForeignRuntime { .. }
            | RelationalBranchBasisDenial::UnknownBranch(_)
            | RelationalBranchBasisDenial::ArchivedBranch(_)
            | RelationalBranchBasisDenial::DeletingBranch(_),
        ) => NoEffectCause::OwnerDeniedBeforeEffect,
        _ => NoEffectCause::PreEffectFailure,
    }
}
