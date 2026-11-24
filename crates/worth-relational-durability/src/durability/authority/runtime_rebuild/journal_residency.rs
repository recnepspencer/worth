//! Release replayed envelopes only against their verified journal placement.
use crate::durability::canonical_commit_artifact::journal_source_read::readmit_journal_commit;
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use contracts::durability::data::recovery_errors::{DurabilityError, RecoveryFailureClass};
use contracts::durability::data::recovery_stop::RelationalRecoveryStop;
use contracts::storage::data::storage_unavailable::{
    RelationalStorageSubject, RelationalStorageUnavailable, RelationalStorageUnavailableCause,
};
use worth_relational_branch::internal_port::runtime::state::subsystems::history::HistorySubsystem;
use worth_relational_contracts::internal_port as contracts;

pub fn install(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    history: &HistorySubsystem,
    plan: &crate::durability::data::RecoveryPlan,
) -> Result<(), RelationalRecoveryStop> {
    for readmitted in &plan.tail_log {
        if !plan.should_restore_authoritative_envelope(readmitted.commit_id()) {
            continue;
        }
        contracts::execution::request_custody::check_request(execution)?;
        let Some((digest, source)) = readmitted.journal_placement() else {
            continue;
        };
        let artifact = history
            .commit_artifact(readmitted.commit_id())
            .ok_or_else(|| corrupt("recovered journal entry has no catalog artifact"))?;
        let cell = artifact.envelope_residency();
        let positioned = cell
            .positioned_if_resident()
            .ok_or_else(|| corrupt("recovered journal entry has no sealed positioned body"))?;
        let observed = RelationalCanonicalCommitArtifact::positioned_payload_digest(&positioned)
            .map_err(|error| corrupt(format!("recovered journal payload digest: {error}")))?;
        if positioned.position() != readmitted.position() || digest != observed {
            return Err(RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Envelope(readmitted.position()),
                cause: RelationalStorageUnavailableCause::IntegrityMismatch,
            }
            .into());
        }
        let entry = worth_relational_branch::internal_port::history::commit::retained_index::RetainedCanonicalIndex::from_publication(
            &positioned,
            artifact.roots(),
            digest,
            execution,
            readmit_journal_commit,
        )
        .map_err(|cause| {
            RelationalRecoveryStop::from(
                contracts::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial::from(cause),
            )
        })?;
        contracts::execution::request_custody::check_request(execution)?;
        cell.bind_performed(&positioned, Some((entry, source)), None);
        cell.settle();
    }
    Ok(())
}
fn corrupt(detail: impl Into<String>) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::CorruptCanonicalArtifact, detail)
}
