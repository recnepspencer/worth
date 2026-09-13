use worth_proof::TransitionOutcome;
use worth_store_aspect_native::StorePhysicalBoundaryWitness;
use worth_store_contracts::{
    StableDigest, StorePhysicalAuthorityWitness, ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
};

use crate::{
    admitted_store_internal_security_scope_for_io_qos_test,
    RecoveryCheckpointRecordSecurityMetadataEnvelope,
    RecoveryCheckpointRecordSecurityMetadataIdentity,
    RecoveryCheckpointRecordSecurityMetadataSource, RecoveryRootSecurityMetadataAdmission,
    RecoveryRootSecurityMetadataEnvelope, RecoverySecurityScopePropagation,
    RecoverySecurityScopePropagationDenial, RecoveryWalRecordSecurityMetadataEnvelope,
    RecoveryWalRecordSecurityMetadataIdentity, RecoveryWalRecordSecurityMetadataSource,
    StoreKeyVersionPosture, StoreLegacySecurityPosture, StoreSecurityMetadata,
    StoreSecurityScopePropagationDenialKind,
};

#[test]
fn recovery_root_metadata_admission_preserves_physical_scope_facts() {
    let admitted = admitted_store_internal_security_scope_for_io_qos_test();
    let metadata = StoreSecurityMetadata::from_current_security_scope(
        admitted.witnesses(),
        StoreKeyVersionPosture::Current,
        StoreLegacySecurityPosture::ReadmissionRequired,
    );
    let admission = RecoveryRootSecurityMetadataAdmission::from_physical_metadata(metadata);

    let raw = admission.to_raw_security_scope_declaration(physical_witness());

    assert_eq!(admission.metadata(), metadata);
    assert_eq!(raw.key_scope(), metadata.key_scope());
    assert_eq!(raw.tenant_scope(), metadata.tenant_scope());
    assert_eq!(raw.key_version_posture(), metadata.key_version_posture());
    assert_eq!(
        raw.authenticity_requirement(),
        Some(metadata.authenticity_requirement())
    );
    assert_eq!(raw.custody_posture(), Some(metadata.custody_posture()));
}

#[test]
fn recovery_metadata_identities_use_durable_source_facts() {
    let wal = TestWalSource {
        sequence: 42,
        metadata: test_metadata(),
    };
    let checkpoint = TestCheckpointSource {
        checkpoint_epoch: 7,
        metadata: test_metadata(),
    };

    assert_eq!(
        RecoveryWalRecordSecurityMetadataIdentity::from_store_wal_record(&wal).sequence(),
        42
    );
    assert_eq!(
        RecoveryCheckpointRecordSecurityMetadataIdentity::from_store_checkpoint_record(&checkpoint)
            .checkpoint_epoch(),
        7
    );

    let wal_envelope = RecoveryWalRecordSecurityMetadataEnvelope::from_wal_record_envelope(&wal);
    let checkpoint_envelope =
        RecoveryCheckpointRecordSecurityMetadataEnvelope::from_checkpoint_record_envelope(
            &checkpoint,
        );
    assert_eq!(wal_envelope.security_metadata(), wal.metadata);
    assert_eq!(checkpoint_envelope.security_metadata(), checkpoint.metadata);
}

#[test]
fn recovery_scope_propagation_preserves_counters_and_denies_missing_root() {
    let admitted = admitted_store_internal_security_scope_for_io_qos_test();
    let wal = RecoveryWalRecordSecurityMetadataEnvelope::from_admitted_scope(
        RecoveryWalRecordSecurityMetadataIdentity::new(42),
        &admitted,
        StoreKeyVersionPosture::Current,
        StoreLegacySecurityPosture::NativeScoped,
    );
    let checkpoint = RecoveryCheckpointRecordSecurityMetadataEnvelope::from_admitted_scope(
        RecoveryCheckpointRecordSecurityMetadataIdentity::new(7),
        &admitted,
        StoreKeyVersionPosture::Current,
        StoreLegacySecurityPosture::NativeScoped,
    );
    let entry_identity = StableDigest::new("recovery-entry").unwrap();
    let root = RecoveryRootSecurityMetadataEnvelope::from_recovery_entry(
        &entry_identity,
        &admitted,
        StoreKeyVersionPosture::Current,
        StoreLegacySecurityPosture::NativeScoped,
    );

    let outcome = RecoverySecurityScopePropagation::admit_required(
        Some(&wal),
        Some(&checkpoint),
        Some(&root),
        &entry_identity,
    );
    let propagation = match outcome {
        TransitionOutcome::Success(propagation) => propagation,
        other => panic!("matching recovery metadata should propagate: {other:?}"),
    };
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

    let missing_root = RecoverySecurityScopePropagation::admit_required(
        Some(&wal),
        Some(&checkpoint),
        None,
        &entry_identity,
    );
    assert_eq!(
        denial_kind(missing_root),
        StoreSecurityScopePropagationDenialKind::MissingPropagatedSecurityScope
    );
}

#[test]
fn recovery_scope_propagation_denies_root_identity_drift_before_replay() {
    let admitted = admitted_store_internal_security_scope_for_io_qos_test();
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
    let expected_identity = StableDigest::new("expected-entry").unwrap();
    let root_identity = StableDigest::new("different-entry").unwrap();
    let root = RecoveryRootSecurityMetadataEnvelope::from_recovery_entry(
        &root_identity,
        &admitted,
        StoreKeyVersionPosture::Current,
        StoreLegacySecurityPosture::NativeScoped,
    );

    let outcome = RecoverySecurityScopePropagation::admit_required(
        Some(&wal),
        Some(&checkpoint),
        Some(&root),
        &expected_identity,
    );
    assert_eq!(
        denial_kind(outcome),
        StoreSecurityScopePropagationDenialKind::ScopeDriftBeforeLogicalDecode
    );
}

fn denial_kind(
    outcome: TransitionOutcome<
        RecoverySecurityScopePropagation,
        RecoverySecurityScopePropagationDenial,
    >,
) -> StoreSecurityScopePropagationDenialKind {
    match outcome {
        TransitionOutcome::Denied(denial) => denial.store_denial().kind(),
        other => panic!("expected recovery security denial, got {other:?}"),
    }
}

fn physical_witness() -> StorePhysicalBoundaryWitness {
    StorePhysicalBoundaryWitness::from_physical_authority(
        StorePhysicalAuthorityWitness::for_aspect_native_boundary(
            ROADMAP_2_ASPECT_NATIVE_GATE_SCOPE,
        )
        .unwrap(),
    )
    .unwrap()
}

fn test_metadata() -> StoreSecurityMetadata {
    let admitted = admitted_store_internal_security_scope_for_io_qos_test();
    StoreSecurityMetadata::from_current_security_scope(
        admitted.witnesses(),
        StoreKeyVersionPosture::Current,
        StoreLegacySecurityPosture::NativeScoped,
    )
}

struct TestWalSource {
    sequence: u64,
    metadata: StoreSecurityMetadata,
}

impl RecoveryWalRecordSecurityMetadataSource for TestWalSource {
    fn recovery_sequence(&self) -> u64 {
        self.sequence
    }

    fn recovery_security_metadata(&self) -> StoreSecurityMetadata {
        self.metadata
    }
}

struct TestCheckpointSource {
    checkpoint_epoch: u64,
    metadata: StoreSecurityMetadata,
}

impl RecoveryCheckpointRecordSecurityMetadataSource for TestCheckpointSource {
    fn recovery_checkpoint_epoch(&self) -> u64 {
        self.checkpoint_epoch
    }

    fn recovery_security_metadata(&self) -> StoreSecurityMetadata {
        self.metadata
    }
}
