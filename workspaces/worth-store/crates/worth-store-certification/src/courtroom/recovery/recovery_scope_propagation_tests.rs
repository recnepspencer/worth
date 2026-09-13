use super::security_scope::{platform_recovery_scope, recovery_entry_identity};
use worth_proof::TransitionOutcome;
use worth_store_contracts::StableDigest;
use worth_store_security::{
    RecoveryCheckpointRecordSecurityMetadataEnvelope, RecoveryRootSecurityMetadataEnvelope,
    RecoverySecurityScopePropagation, RecoverySecurityScopePropagationDenial,
    RecoverySecurityScopePropagationInput, RecoveryWalRecordSecurityMetadataEnvelope,
    StoreKeyVersionPosture, StoreLegacySecurityPosture, StoreSecurityScopePropagationDenialKind,
};
use worth_store_wal::{
    CheckpointRecordSecurityMetadataEnvelope, StoreCheckpointRecordIdentity,
    StoreWalRecordIdentity, WalRecordSecurityMetadataEnvelope, WalSecurityMetadataCarrier,
};

#[test]
fn recovery_scope_propagation_uses_wal_checkpoint_carrier_identities() {
    let entry_identity = recovery_entry_identity("wal-carrier");
    let propagation = recovery_scope_from_wal_carriers(
        &entry_identity,
        StoreKeyVersionPosture::Current,
        StoreKeyVersionPosture::Current,
        StoreKeyVersionPosture::Current,
    )
    .expect("matching WAL-carried scope should admit");

    assert_eq!(propagation.entry_identity(), &entry_identity);
    assert_eq!(propagation.wal_record_identity().sequence(), 42);
    assert_eq!(
        propagation.checkpoint_record_identity().checkpoint_epoch(),
        7
    );
    assert_eq!(
        propagation
            .counters()
            .wal_checkpoint_store_counters()
            .preserved(),
        1
    );
    assert_eq!(propagation.counters().root_store_counters().preserved(), 1);
}

#[test]
fn recovery_scope_denies_stale_wal_scope_before_replay_publication() {
    let entry_identity = recovery_entry_identity("stale-wal");
    let denial = recovery_scope_from_wal_carriers(
        &entry_identity,
        StoreKeyVersionPosture::Stale,
        StoreKeyVersionPosture::Current,
        StoreKeyVersionPosture::Current,
    )
    .unwrap_err();

    assert_eq!(
        denial.store_denial().kind(),
        StoreSecurityScopePropagationDenialKind::StalePropagatedSecurityScope
    );
    assert_eq!(denial.store_denial().counters().stale(), 1);
}

#[test]
fn recovery_scope_denies_unsupported_scope_before_replay_publication() {
    let entry_identity = recovery_entry_identity("unsupported-scope");
    let denial = recovery_scope_from_wal_carriers(
        &entry_identity,
        StoreKeyVersionPosture::Unsupported,
        StoreKeyVersionPosture::Current,
        StoreKeyVersionPosture::Current,
    )
    .unwrap_err();

    assert_eq!(
        denial.store_denial().kind(),
        StoreSecurityScopePropagationDenialKind::UnsupportedPropagatedSecurityScope
    );
    assert_eq!(denial.store_denial().counters().unsupported(), 1);
}

#[test]
fn recovery_scope_missing_root_denies_before_replay_publication() {
    let entry_identity = recovery_entry_identity("missing-root");
    let admitted = platform_recovery_scope("missing-root");
    let wal = RecoveryWalRecordSecurityMetadataEnvelope::from_wal_record_envelope(&wal_record(
        &admitted,
        StoreKeyVersionPosture::Current,
    ));
    let checkpoint =
        RecoveryCheckpointRecordSecurityMetadataEnvelope::from_checkpoint_record_envelope(
            &checkpoint_record(&admitted, StoreKeyVersionPosture::Current),
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
        other => panic!("missing root must deny: {other:?}"),
    }
}

fn recovery_scope_from_wal_carriers(
    entry_identity: &StableDigest,
    wal_key_version: StoreKeyVersionPosture,
    checkpoint_key_version: StoreKeyVersionPosture,
    root_key_version: StoreKeyVersionPosture,
) -> Result<RecoverySecurityScopePropagation, RecoverySecurityScopePropagationDenial> {
    let admitted = platform_recovery_scope(entry_identity.as_str());
    let wal = RecoveryWalRecordSecurityMetadataEnvelope::from_wal_record_envelope(&wal_record(
        &admitted,
        wal_key_version,
    ));
    let checkpoint =
        RecoveryCheckpointRecordSecurityMetadataEnvelope::from_checkpoint_record_envelope(
            &checkpoint_record(&admitted, checkpoint_key_version),
        );
    let root = RecoveryRootSecurityMetadataEnvelope::from_recovery_entry(
        entry_identity,
        &admitted,
        root_key_version,
        StoreLegacySecurityPosture::NativeScoped,
    );

    match RecoverySecurityScopePropagation::admit(RecoverySecurityScopePropagationInput::new(
        &wal,
        &checkpoint,
        &root,
        entry_identity,
    )) {
        TransitionOutcome::Success(scope) => Ok(scope),
        TransitionOutcome::Denied(denial) => Err(denial),
        TransitionOutcome::Deferred(deferred) => match deferred {},
        TransitionOutcome::Stale(stale) => match stale {},
        TransitionOutcome::RebindRequired(rebind) => match rebind {},
        TransitionOutcome::Failed(failed) => match failed {},
    }
}

fn wal_record(
    admitted: &worth_store_security::StoreAdmittedSecurityScope,
    key_version: StoreKeyVersionPosture,
) -> WalRecordSecurityMetadataEnvelope {
    WalRecordSecurityMetadataEnvelope::wal_record(
        StoreWalRecordIdentity::new(42),
        WalSecurityMetadataCarrier::for_wal_record(
            admitted.witnesses(),
            key_version,
            StoreLegacySecurityPosture::NativeScoped,
        ),
    )
}

fn checkpoint_record(
    admitted: &worth_store_security::StoreAdmittedSecurityScope,
    key_version: StoreKeyVersionPosture,
) -> CheckpointRecordSecurityMetadataEnvelope {
    CheckpointRecordSecurityMetadataEnvelope::checkpoint_record(
        StoreCheckpointRecordIdentity::new(7),
        WalSecurityMetadataCarrier::for_checkpoint_record(
            admitted.witnesses(),
            key_version,
            StoreLegacySecurityPosture::NativeScoped,
        ),
    )
}
