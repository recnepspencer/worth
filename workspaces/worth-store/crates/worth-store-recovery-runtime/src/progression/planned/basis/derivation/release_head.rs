//! A pending head-bearing publication may only consume the C8 media-joined
//! C9 transition. Routed controls and a raw persisted effect are insufficient.

use worth_store_recovery_physics::VerifiedSelectedReleaseHeadReplayV14;

use super::{pending::PendingProjectionBasis, ExecutionBasisDenial};

pub(super) fn require_exact_pending_replay(
    pending: &PendingProjectionBasis<'_>,
    replay: Option<&VerifiedSelectedReleaseHeadReplayV14>,
) -> Result<(), ExecutionBasisDenial> {
    let mut effects = pending.projections.iter().filter_map(|projection| {
        (match projection.materialization().operation() {
            worth_store_physical_format::PersistedPhysicalRecoveryOperation::RecordsDropped {
                head_effect,
                ..
            } => head_effect.as_ref(),
            _ => None,
        })
        .map(|effect| (projection.operation(), effect))
    });
    match (effects.next(), replay) {
        (None, None) => Ok(()),
        (Some((operation, effect)), Some(replay))
            if effects.next().is_none()
                && replay.operation() == operation
                && replay.effect() == effect
                && effect.result_root().generation() == pending.staging_generation =>
        {
            Ok(())
        }
        _ => Err(ExecutionBasisDenial::Invalid),
    }
}
