use crate::entry::{
    AdmittedPlatformAuthority, PhysicalRecoveryBlock, PhysicalRecoveryBlockCause,
    PhysicalRecoveryBlockEvidence, PhysicalRecoveryOutcome,
};
use crate::orchestration::RecoveryCoordination;

pub(crate) fn block_unsupported_scope(
    authority: AdmittedPlatformAuthority,
    coordination: RecoveryCoordination,
    cause: PhysicalRecoveryBlockCause,
    evidence: PhysicalRecoveryBlockEvidence,
) -> PhysicalRecoveryOutcome {
    assert!(
        coordination.shutdown_is_quiescent(),
        "source discovery must be quiescent before a persisted-source block terminates"
    );
    let recovery_effects = authority.media.recovery_effect_count();
    let store = authority.media.store_identity();
    let session_identity = authority.session.identity();
    let AdmittedPlatformAuthority { media, session, .. } = authority;
    drop(media);
    session.block();
    PhysicalRecoveryOutcome::Blocked(PhysicalRecoveryBlock::new(
        cause,
        store,
        session_identity,
        evidence,
        recovery_effects,
    ))
}
