use super::security_scope::{
    platform_recovery_scope, recovery_entry_identity, recovery_security_scope,
};
use worth_proof::TransitionOutcome;
use worth_store_security::{
    RecoveryCheckpointRecordSecurityMetadataEnvelope,
    RecoveryCheckpointRecordSecurityMetadataIdentity, RecoveryRootSecurityMetadataEnvelope,
    RecoverySecurityScopePropagation, RecoverySecurityScopePropagationInput,
    RecoveryWalRecordSecurityMetadataEnvelope, RecoveryWalRecordSecurityMetadataIdentity,
    StoreKeyVersionPosture, StoreLegacySecurityPosture, StoreSecurityScopePropagationDenialKind,
};

#[test]
fn replay_scope_propagation_requires_a_matching_durable_entry_identity() {
    let entry_identity = recovery_entry_identity("entry-required");
    let propagation = recovery_security_scope(&entry_identity);

    assert_eq!(propagation.entry_identity(), &entry_identity);
    assert_eq!(
        propagation
            .counters()
            .wal_checkpoint_store_counters()
            .preserved(),
        1
    );
    assert_eq!(propagation.counters().root_store_counters().preserved(), 1);
    assert_eq!(propagation.counters().wal_checkpoint_comparisons(), 1);
    assert_eq!(propagation.counters().root_scope_comparisons(), 1);
}

#[test]
fn replay_scope_propagation_denies_a_root_bound_to_another_identity() {
    let expected_identity = recovery_entry_identity("entry-replay");
    let root_identity = recovery_entry_identity("entry-root");
    let admitted = platform_recovery_scope("entry-root");
    let wal = RecoveryWalRecordSecurityMetadataEnvelope::from_admitted_scope(
        RecoveryWalRecordSecurityMetadataIdentity::new(1),
        &admitted,
        StoreKeyVersionPosture::Current,
        StoreLegacySecurityPosture::NativeScoped,
    );
    let checkpoint = RecoveryCheckpointRecordSecurityMetadataEnvelope::from_admitted_scope(
        RecoveryCheckpointRecordSecurityMetadataIdentity::new(1),
        &admitted,
        StoreKeyVersionPosture::Current,
        StoreLegacySecurityPosture::NativeScoped,
    );
    let root = RecoveryRootSecurityMetadataEnvelope::from_recovery_entry(
        &root_identity,
        &admitted,
        StoreKeyVersionPosture::Current,
        StoreLegacySecurityPosture::NativeScoped,
    );

    let outcome = RecoverySecurityScopePropagation::admit(
        RecoverySecurityScopePropagationInput::new(&wal, &checkpoint, &root, &expected_identity),
    );

    match outcome {
        TransitionOutcome::Denied(denial) => assert_eq!(
            denial.store_denial().kind(),
            StoreSecurityScopePropagationDenialKind::ScopeDriftBeforeLogicalDecode
        ),
        other => panic!("mismatched recovery root must deny: {other:?}"),
    }
}

#[test]
fn missing_root_denies_before_replay_publication() {
    let entry_identity = recovery_entry_identity("missing-root");
    let admitted = platform_recovery_scope("missing-root");
    let wal = RecoveryWalRecordSecurityMetadataEnvelope::from_admitted_scope(
        RecoveryWalRecordSecurityMetadataIdentity::new(1),
        &admitted,
        StoreKeyVersionPosture::Current,
        StoreLegacySecurityPosture::NativeScoped,
    );
    let checkpoint = RecoveryCheckpointRecordSecurityMetadataEnvelope::from_admitted_scope(
        RecoveryCheckpointRecordSecurityMetadataIdentity::new(1),
        &admitted,
        StoreKeyVersionPosture::Current,
        StoreLegacySecurityPosture::NativeScoped,
    );

    let outcome = RecoverySecurityScopePropagation::admit_required(
        Some(&wal),
        Some(&checkpoint),
        None,
        &entry_identity,
    );

    match outcome {
        TransitionOutcome::Denied(denial) => assert_eq!(
            denial.store_denial().kind(),
            StoreSecurityScopePropagationDenialKind::MissingPropagatedSecurityScope
        ),
        other => panic!("missing recovery root must deny: {other:?}"),
    }
}
