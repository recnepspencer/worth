use crate::capabilities::RuntimeConfigSource;
use crate::durability::data::{
    DurableCheckpoint, RecoveryAuthorityContinuityMismatch, RecoveryAuthorityParity, RecoveryPlan,
    RecoveryVerificationMode, RecoveryVerificationOutcome,
};
use crate::replay::data::ReplayVerificationLayer;
use crate::runtime::durability_access::{
    authority_continuity_for_envelopes, descriptor_semantics_version_for_envelopes,
};
use crate::runtime::RelationalRuntime;
use worth_relational_durability::internal_port::durability::data::checkpoint_images::DurableCheckpointOwnerPort as _;
use worth_relational_durability::internal_port::durability::data::recovery_plan::RecoveryPlanOwnerPort as _;

pub(crate) fn native_checkpoint_recovery_plan(
    runtime: &RelationalRuntime,
    checkpoint: DurableCheckpoint,
    verification_mode: RecoveryVerificationMode,
) -> RecoveryPlan {
    let descriptor_semantics_version =
        descriptor_semantics_version_for_envelopes(checkpoint.envelopes(), &[]);
    let mut continuity = authority_continuity_for_envelopes(runtime, checkpoint.envelopes(), &[]);
    bind_source_runtime_identity(runtime, &checkpoint, &mut continuity);
    RecoveryPlan::new(
        runtime.runtime_config().clone(),
        Some(checkpoint),
        Vec::new(),
        crate::durability::data::RecoveryBranchTransitions::Contiguous(Vec::new()),
        continuity,
        verification_mode,
        descriptor_semantics_version,
        Vec::new(),
        runtime.durability.cold_source(),
    )
}

pub(super) fn bind_source_runtime_identity(
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
