use super::*;

/// The exact record shape every proof in this module expects when a settled
/// Relational effect is retained and the Signal owner never moved.
pub(super) fn assert_retains_only_the_relational_effect(
    record: &crate::recovery::ProductUnpublishedOwnerEffects,
    cause: ProductUnpublishedCause,
) {
    assert_eq!(record.cause(), cause);
    assert_eq!(record.owner_effect_count(), 1);
    assert_eq!(
        record.progress().relational_posture(),
        RelationalAttemptProgressPosture::Settled
    );
    assert_eq!(
        record.progress().signal_posture(),
        SignalAttemptProgressPosture::Untouched
    );
}
