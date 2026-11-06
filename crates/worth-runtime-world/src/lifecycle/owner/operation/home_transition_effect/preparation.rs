//! Admit every live resource before transferring the original Native capability.
use super::*;
use crate::recovery::{
    ContinuationCommon, OriginalHomeTransitionContinuation, PreparedOriginalContinuation,
    RecoveredWorldPublicationRecord,
};

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn prepare_original_home_continuation(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        plan: &HomeProgramTransitionPlanDescription,
        step: HomeProgramTransitionStepAssociation,
        association: [u8; 32],
        record: &RecoveredWorldPublicationRecord,
        native: &worth_relational::facade::durability::RelationalRecoveredPublicationCustody,
    ) -> Result<PreparedOriginalContinuation, Denial> {
        let current = self
            .admit_publication_source(&record.expected)
            .map_err(|cause| Denial::Source(cause.cause()))?;
        if record
            .expected
            .mismatch_against_snapshot(&current)
            .is_some()
        {
            return Err(Denial::Source(NoEffectCause::StaleExpectedProductHead));
        }
        let memory =
            OriginalHomeTransitionContinuation::reserve_backing(request).map_err(Denial::Medium)?;
        let prepared = self
            .prepare_readmitted_successor(request, &record.expected, record.open.open(), native)
            .map_err(Denial::Readmission)?;
        if prepared.after.selected_commit() != &record.commit || prepared.attempt != record.attempt
        {
            return Err(Denial::NativeEffectMismatch);
        }
        let operation = self
            .reserve_recovery_operation_if_open_and_bootstrapped()
            .map_err(|()| Denial::Source(NoEffectCause::OwnerUnavailable))?;
        let mut resources =
            reservation_steps::reserve_readmitted_resources(self, &record.expected, &record.commit)
                .map_err(Denial::Source)?;
        let publication_pins = resources
            .pins
            .try_bind_publication(prepared.after.basis())
            .map_err(Denial::retention)?;
        let history_pins = publication_pins
            .fork_history(prepared.after.basis())
            .map_err(Denial::transfer)?;
        let origin_pins = publication_pins
            .fork_history(prepared.after.basis())
            .map_err(Denial::transfer)?;
        let open =
            crate::durability::WorldDurableArtifact::from_readmitted_open(&record.open, request)
                .map_err(Denial::Medium)?;
        let mut occurrence = crate::durability::WorldOccurrenceCustody::prepare(&open, request)
            .map_err(Denial::Medium)?;
        occurrence.bind_origin(prepared.after.basis(), origin_pins);
        let actual = native.effect().subject();
        occurrence
            .finalize(
                crate::durability::WorldRelationalEvidence {
                    commit: actual.receipt().commit_id.0,
                    position: actual.position().0,
                    digest: *actual.artifact_digest(),
                },
                record.binding.binding().incarnation(),
            )
            .map_err(Denial::Medium)?;
        let cell = self
            .state
            .branches
            .branch_cell(record.expected.branch_identity())
            .ok_or(Denial::WorldHeadUnavailable)?;
        Ok(PreparedOriginalContinuation {
            step,
            plan: plan.commitment(),
            association,
            common: ContinuationCommon {
                control: record
                    .control
                    .lock()
                    .unwrap_or_else(|cause| cause.into_inner())
                    .clone(),
                before: record.expected.clone(),
                after: prepared.after,
                attempt: prepared.attempt,
                binding: record.binding.clone(),
                cell,
                occurrence,
                history: resources.history,
                capacity: resources.capacity,
                history_pins: Some(history_pins),
                publication_pins,
            },
            record_backing: prepared.record,
            operation,
            publication_capacity: resources.publication,
            memory,
        })
    }
}
