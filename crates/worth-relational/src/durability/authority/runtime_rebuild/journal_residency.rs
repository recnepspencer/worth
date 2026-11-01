//! Release replayed envelopes only against their verified journal placement.
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact;
use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
use crate::runtime::RelationalRuntime;
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable, RelationalStorageUnavailableCause,
};

pub(super) fn install(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    plan: &crate::durability::data::RecoveryPlan,
) -> Result<(), RelationalRecoveryStop> {
    for readmitted in &plan.tail_log {
        if !plan.should_restore_authoritative_envelope(readmitted.commit_id()) {
            continue;
        }
        crate::execution::check_request(execution)?;
        let Some((digest, source)) = readmitted.journal_placement() else {
            continue;
        };
        let artifact = runtime
            .history
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
        let entry = crate::history::RetainedCanonicalIndex::from_publication(
            &positioned,
            artifact.roots(),
            digest,
            execution,
        )
        .map_err(|cause| {
            RelationalRecoveryStop::from(
                crate::durability::data::RelationalNativeCheckpointCaptureDenial::from(cause),
            )
        })?;
        crate::execution::check_request(execution)?;
        cell.bind_performed(&positioned, Some((entry, source)), None);
        cell.settle();
    }
    Ok(())
}
fn corrupt(detail: impl Into<String>) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::CorruptCanonicalArtifact, detail)
}
