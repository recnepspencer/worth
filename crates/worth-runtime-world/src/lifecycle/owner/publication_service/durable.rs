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
        use crate::durability::WorldOpenComponentIntent as Intent;
        use crate::durability::{WorldOccurrenceOwnerEvidence as Owners, WorldSignalEvidence};
        let evidence = (|| {
            let signal = || {
                let output = ready
                    .signal_durable_output()
                    .ok_or(Denial::OriginalSignalEvidenceRequired)?;
                WorldSignalEvidence::from_point(output.branch().source_point().clone(), execution)
                    .map_err(Denial::Medium)
            };
            match ready
                .durable_component_intent()
                .ok_or(Denial::EvidenceRequired)?
            {
                Intent::RelationalOnly => Ok(Owners::RelationalOnly(
                    self.native_evidence(&ready, execution)?,
                )),
                Intent::SignalOnly => Ok(Owners::SignalOnly(signal()?)),
                Intent::RelationalAndSignal => Ok(Owners::RelationalAndSignal {
                    relational: self.native_evidence(&ready, execution)?,
                    signal: signal()?,
                }),
            }
        })();
        let evidence = match evidence {
            Ok(evidence) => evidence,
            Err(cause) => return ready.durable_recovery(cause),
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
            &self.state.operation_control,
        )
    }
    fn native_evidence(
        &self,
        ready: &crate::publication::CompositePublicationReady,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorldRelationalEvidence, Denial> {
        let Some((_, original_basis, original_result)) = ready.original_relational_parts() else {
            return Err(Denial::OriginalNativeEvidenceRequired);
        };
        let recorded = match self
            .state
            .relational_services()
            .basis_port()
            .export_recorded_retained_component(execution, &original_basis)
        {
            Ok(recorded) => recorded,
            Err(cause) => return Err(Denial::Native(cause)),
        };
        let RecordedRelationalComponentTarget::Canonical(actual) = recorded.target() else {
            return Err(Denial::OriginalNativeEvidenceRequired);
        };
        if actual.receipt() != &original_result.outcome().commit
            || actual.position() != original_result.patch_position()
        {
            return Err(Denial::OriginalNativeEvidenceRequired);
        }
        let evidence = WorldRelationalEvidence {
            commit: actual.receipt().commit_id.0,
            position: actual.position().0,
            digest: *actual.artifact_digest(),
        };
        Ok(evidence)
    }
}
