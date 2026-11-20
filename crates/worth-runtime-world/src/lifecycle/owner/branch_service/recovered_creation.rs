//! Decide one cold-recovered creation by the fate its owners recorded: a
//! performed original continues to its exact forks, an effect-free one releases.
//! The decision is durable before any live installation and is never repeated.
use super::*;
use crate::durability::{
    WorldArtifactInput, WorldCommittedPublication, WorldCreatedBranchOwnerEvidence as Owners,
    WorldCreationIntentInput, WorldDurableArtifact, WorldDurablePublicationStop as Stop,
    WorldRecord, WorldRelationalForkEvidence, WorldSignalEvidence,
};
use crate::recovery::{
    ClaimedRecoveredCreation, RecoveredCreationFate, RecoveredCreationForks,
    RecoveredWorldCreationRecord, RuntimeWorldRecoveryDenial as Denial,
};

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn resolve_recovered_creation(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mut claim: ClaimedRecoveredCreation,
    ) -> Result<RuntimeWorldBranchCreationOutcome, Denial> {
        let _operation = self
            .reserve_recovery_operation_if_open_and_bootstrapped()
            .map_err(|()| crate::lifecycle::RuntimeWorldOwnerUnavailable::new())?;
        let record = claim.record.clone();
        if claim.custody().decision.is_none() {
            let decision = self.prepare_recovered_decision(execution, &record, claim.custody())?;
            claim.custody().decision = Some(decision);
        }
        if claim.custody().committed.is_none() {
            let committed = publish_decision(execution, &record, claim.custody())?;
            claim.custody().committed = Some(committed);
        }
        let observation = match &claim.custody().fate {
            RecoveredCreationFate::Release => None,
            RecoveredCreationFate::Continue(_) => {
                Some(self.install_recovered_creation(execution, &record, claim.custody())?)
            }
        };
        let custody = claim.finish();
        let committed = custody.committed.expect("durable original decision");
        Ok(match observation {
            Some(observation) => RuntimeWorldBranchCreationOutcome::Performed(observation),
            None => RuntimeWorldBranchCreationOutcome::ReleasedOriginal(
                crate::lifecycle::RuntimeWorldBranchCreationRelease::from_recovered(
                    record.intent.association.clone(),
                    committed,
                ),
            ),
        })
    }

    /// Encode the one admissible decision. A continuation records exactly the
    /// owner-admitted forks, described by their owners as the live selection does.
    fn prepare_recovered_decision(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        record: &RecoveredWorldCreationRecord,
        custody: &crate::recovery::RecoveredCreationCustody,
    ) -> Result<WorldDurableArtifact, Denial> {
        let intent = &record.intent;
        let input = match &custody.fate {
            RecoveredCreationFate::Release => WorldArtifactInput::CreationReleased {
                association: &intent.association,
                custody: intent.custody,
            },
            RecoveredCreationFate::Continue(forks) => WorldArtifactInput::BranchCreated {
                intent: WorldCreationIntentInput::from_record(intent),
                components: self.recovered_creation_components(execution, record, forks)?,
                owners: owner_evidence(execution, forks)?,
                publishing_incarnation: record.binding.binding().incarnation(),
            },
        };
        WorldDurableArtifact::from_input(input, execution)
            .map_err(|_| Denial::OutputCapacityExhausted)
    }

    fn recovered_creation_components(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        record: &RecoveredWorldCreationRecord,
        forks: &RecoveredCreationForks,
    ) -> Result<crate::durability::RecordedWorldInitialComponents, Denial> {
        let source = record.source.basis();
        let native_basis = forks
            .native
            .as_ref()
            .map_or_else(|| source.relational_basis(), |fork| fork.target_basis());
        let relational = self
            .state
            .relational_services()
            .basis_port()
            .export_recorded_retained_component(execution, native_basis)
            .map_err(|_| Denial::OriginalSourceUnavailable)?;
        let point = forks.signal.as_ref().map_or_else(
            || record.source_signal.clone(),
            |actual| actual.source_point().clone(),
        );
        let signal = WorldSignalEvidence::from_point(point, execution)
            .map_err(|_| Denial::OutputCapacityExhausted)?;
        // A creation keeps its source's correspondence, as the live successor does.
        let bridge = self
            .state
            .bridge
            .as_ref()
            .expect("installed correspondence")
            .export_recorded_runtime_world_baseline_in(execution, source.correspondence_basis())
            .map_err(|_| Denial::OriginalSourceUnavailable)?;
        Ok(crate::durability::RecordedWorldInitialComponents {
            relational,
            signal,
            bridge,
        })
    }

    /// The durable selection installs through the same cold installer reopen uses.
    fn install_recovered_creation(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        record: &RecoveredWorldCreationRecord,
        custody: &mut crate::recovery::RecoveredCreationCustody,
    ) -> Result<ProductBranchObservation, Denial> {
        let Some(WorldRecord::BranchCreated(created)) =
            custody.decision.as_ref().map(WorldDurableArtifact::record)
        else {
            unreachable!("a continuing original selects its branch")
        };
        let RecoveredCreationFate::Continue(forks) = &custody.fate else {
            unreachable!("a continuing original holds its forks")
        };
        // Installation after the durable selection is ordinary readmission:
        // a refusal here leaves the selection for the next reopen to install.
        let (observation, origin) = self
            .install_readmitted_created_branch(
                created,
                &record.source,
                &record.source_signal,
                forks.native.as_ref(),
                forks.signal.as_deref(),
                &mut custody.destination,
                execution,
            )
            .map_err(|_| Denial::ReopenRequired)?;
        record.binding.retain_checkpoint_occurrence(origin);
        Ok(observation)
    }
}

fn owner_evidence(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    forks: &RecoveredCreationForks,
) -> Result<Owners, Denial> {
    let native = forks
        .native
        .as_ref()
        .map(|fork| {
            let original = fork.original();
            original
                .recorded_artifact_digest()
                .map(|digest| WorldRelationalForkEvidence {
                    sequence: original.sequence(),
                    digest,
                })
                .ok_or(Denial::OriginalSourceMismatch)
        })
        .transpose()?;
    let signal = forks
        .signal
        .as_ref()
        .map(|actual| WorldSignalEvidence::from_point(actual.source_point().clone(), execution))
        .transpose()
        .map_err(|_| Denial::OutputCapacityExhausted)?;
    Ok(match (native, signal) {
        (Some(relational), None) => Owners::Relational(relational),
        (None, Some(signal)) => Owners::Signal(signal),
        (Some(relational), Some(signal)) => Owners::RelationalAndSignal { relational, signal },
        (None, None) => unreachable!("an effect-free original releases"),
    })
}

/// Pending or refused writes leave the original for a retry; an unresolved
/// write leaves it for the next reopen, which reads what actually landed.
fn publish_decision(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    record: &RecoveredWorldCreationRecord,
    custody: &crate::recovery::RecoveredCreationCustody,
) -> Result<WorldCommittedPublication, Denial> {
    let decision = custody.decision.as_ref().expect("prepared decision");
    record
        .binding
        .publish(decision, execution)
        .map_err(|stop| match stop {
            Stop::Pending | Stop::Busy | Stop::Refused(_) => Denial::Busy,
            Stop::Unresolved | Stop::EvidenceRequired => Denial::ReopenRequired,
        })
}
