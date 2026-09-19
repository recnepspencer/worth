use crate::capabilities::{RuntimeConfigSource, RuntimeIdentitySource};
use crate::durability::access::{
    authority_continuity_for_envelopes, descriptor_semantics_version_for_envelopes,
};
use crate::durability::data::{
    DurableCheckpoint, RecoveryAuthorityContinuityMismatch, RecoveryAuthorityParity,
    RecoveryCursor, RecoveryIntegrityReport, RecoveryPlan, RecoveryVerificationMode,
    RecoveryVerificationOutcome,
};
use crate::replay::data::ReplayVerificationLayer;
use crate::runtime::RelationalRuntime;

pub(crate) fn native_checkpoint_recovery_plan(
    runtime: &RelationalRuntime,
    checkpoint: DurableCheckpoint,
    verification_mode: RecoveryVerificationMode,
) -> RecoveryPlan {
    let descriptor_semantics_version =
        descriptor_semantics_version_for_envelopes(checkpoint.envelopes.as_slice(), &[]);
    let mut continuity =
        authority_continuity_for_envelopes(runtime, checkpoint.envelopes.as_slice(), &[]);
    bind_source_runtime_identity(runtime, &checkpoint, &mut continuity);
    RecoveryPlan::new(
        runtime.runtime_config().clone(),
        None,
        None,
        Some(checkpoint),
        Vec::new(),
        RecoveryCursor {
            checkpoint_id: None,
            segment_ids: Vec::new(),
        },
        RecoveryIntegrityReport {
            selected_checkpoint_id: None,
            skipped_corrupt_checkpoints: Vec::new(),
            verified_segment_ids: Vec::new(),
            corrupt_segment_id: None,
        },
        continuity,
        verification_mode,
        descriptor_semantics_version,
        Vec::new(),
    )
    .with_commit_strategy_executors(runtime.commit_strategy_executor_registry().clone())
}

fn bind_source_runtime_identity(
    runtime: &RelationalRuntime,
    checkpoint: &DurableCheckpoint,
    continuity: &mut crate::durability::data::RecoveryAuthorityContinuityCheck,
) {
    if checkpoint.runtime_name == runtime.runtime_name() {
        return;
    }
    continuity.runtime_name_parity = RecoveryAuthorityParity::drift();
    continuity.first_mismatch.get_or_insert_with(|| {
        RecoveryAuthorityContinuityMismatch::RuntimeName {
            expected: runtime.runtime_name().to_owned(),
            found: checkpoint.runtime_name.clone(),
        }
    });
    continuity.verification_outcome = RecoveryVerificationOutcome::Rejected {
        layer: ReplayVerificationLayer::DigestParity,
        detail: "native checkpoint source runtime name differs".to_owned(),
    };
}
