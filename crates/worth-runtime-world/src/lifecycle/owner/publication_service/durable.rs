use super::RuntimeWorldOwnerRoot;
use crate::durability::WorldRelationalEvidence;
use crate::publication::{
    OwnerExecutionOutcome, RuntimeWorldCancellationToken, WorldDurablePublicationDenial as Denial,
    WorldDurablePublicationOutcome as Outcome,
};
use worth_relational::facade::branch::RecordedRelationalComponentTarget;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(crate) fn finish_durable_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        outcome: OwnerExecutionOutcome,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> Outcome {
        let settlement = match outcome {
            OwnerExecutionOutcome::ProductUnpublished(value) => {
                return Outcome::ProductUnpublished(value)
            }
            OwnerExecutionOutcome::Settled(value) => value,
            OwnerExecutionOutcome::NoEffect(_) => {
                unreachable!("an accepted durable open remains retained")
            }
        };
        let successor = settlement
            .successor_basis()
            .expect("actual settled successor retained before consuming Native performed")
            .clone();
        let ready = match settlement.ready(successor) {
            Ok(ready) => ready,
            Err(custody) => return Outcome::ProductUnpublished(custody),
        };
        let Some((_, original_basis, original_result)) = ready.original_relational_parts() else {
            return ready.durable_recovery(Denial::OriginalNativeEvidenceRequired);
        };
        let recorded = match self
            .state
            .relational
            .basis_port()
            .export_recorded_retained_component(execution, &original_basis)
        {
            Ok(recorded) => recorded,
            Err(cause) => return ready.durable_recovery(Denial::Native(cause)),
        };
        let RecordedRelationalComponentTarget::Canonical(actual) = recorded.target() else {
            return ready.durable_recovery(Denial::OriginalNativeEvidenceRequired);
        };
        if actual.receipt() != &original_result.outcome().commit
            || actual.position() != original_result.patch_position()
        {
            return ready.durable_recovery(Denial::OriginalNativeEvidenceRequired);
        }
        let evidence = WorldRelationalEvidence {
            commit: actual.receipt().commit_id.0,
            position: actual.position().0,
            digest: *actual.artifact_digest(),
        };
        let Some(cell) = self
            .state
            .branches
            .branch_cell(ready.expected_head().branch_identity())
        else {
            return ready.durable_recovery(Denial::MovementAdmission);
        };
        ready.begin_durable(
            cell,
            evidence,
            execution,
            cancellation.clone(),
            &self.state.clock,
        )
    }
}
