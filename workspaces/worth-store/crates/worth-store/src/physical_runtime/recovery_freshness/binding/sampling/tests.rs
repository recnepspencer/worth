//! Mechanical native-owner evidence, not an admission of serialized WAL fate.

use super::*;
use crate::physical_runtime::{
    durability::{
        PhysicalBindingDecodingContext, PhysicalMutationDurabilityRequest,
        PhysicalMutationFingerprintInput, PhysicalMutationOperationFamily,
        PhysicalMutationPayloadDigest, PhysicalMutationRequestScope, PhysicalMutationSecurityBasis,
    },
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy, FilesystemAccessPosture,
    FilesystemMediaAdmission, PhysicalDurabilityPolicyIdentity, PhysicalIdempotencyPolicy,
    PhysicalMutationIdentity, PhysicalMutationRequestFingerprint, PhysicalOperationIdentity,
    PhysicalRecoveryCoordinationCapacity, PhysicalRecoveryFreshnessPort, PhysicalRuntimeAdmission,
    PhysicalStore, PhysicalWorkGeneration, PhysicalWorkIdentity, QualifiedRecoveryFilesystemMedia,
};
use std::num::NonZeroU64;
use worth_proof::TransitionOutcome;
use worth_store_physical_format::*;
use worth_store_physical_integrity::*;

mod native_owner;

const ORIGINAL: u64 = 16 * 1024;

fn with_owner(
    run: impl FnOnce(
        &mut PhysicalRecoveryCoordination,
        &AdmittedRecoveryFilesystemMedia,
        super::super::StoreRecoveryOperationEvidence,
    ),
) {
    let directory = tempfile::tempdir().unwrap();
    let runtime =
        PhysicalStore::admit(PhysicalRuntimeAdmission::new(directory.path()).unwrap()).unwrap();
    let TransitionOutcome::Success(media) = runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    else {
        panic!("actual namespace must initialize");
    };
    let store = media.store_identity();
    let mutation = PhysicalMutationIdentity::from_reserved_operation(
        PhysicalWorkIdentity::from_instance_owner(
            store,
            media.runtime_identity(),
            PhysicalWorkGeneration::from_lifecycle(
                media.observer().snapshot().unwrap().generation(),
            ),
            PhysicalOperationIdentity::from_owner_sequence(NonZeroU64::new(1).unwrap()),
        ),
    );
    let policy = PhysicalDurabilityPolicyIdentity::from_recovery_binding([7; 32]);
    let fingerprint =
        PhysicalMutationRequestFingerprint::derive(PhysicalMutationFingerprintInput {
            store,
            durability_policy: policy,
            scope: PhysicalMutationRequestScope::record_append([3; 32]),
            payload: PhysicalMutationPayloadDigest::from_validated_payload([4; 32]),
            durability_request: PhysicalMutationDurabilityRequest::PlatformDurable,
            operation_family: PhysicalMutationOperationFamily::RecordAppend,
            security_bases: &[PhysicalMutationSecurityBasis::from_admitted_security(
                [5; 32],
            )],
        })
        .unwrap();
    // These inline comparison observations issue no lease, submission or WAL authority.
    let evidence = super::super::StoreRecoveryOperationEvidence {
        idempotency_identity: [1; 32],
        mutation,
        request_fingerprint: fingerprint,
        lease_issuance_generation: 0,
        lease_expiry_generation: 4,
        freshness: super::super::StoreRecoveryBindingFreshness::Retained,
        fate: super::super::StoreRecoveryOperationFate::Indeterminate,
        attempt_binding_identity: None,
    };
    media.close();
    let qualified = QualifiedRecoveryFilesystemMedia::qualify_existing(directory.path()).unwrap();
    let freshness = PhysicalRecoveryFreshnessPort::admit(&qualified).unwrap();
    let media = qualified.admit_persisted_store().unwrap();
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let mut coordination = freshness
        .register_session()
        .unwrap()
        .admit_coordination(
            &media,
            PhysicalRecoveryCoordinationCapacity::admit(1, 4096, 1, 4096)
                .unwrap()
                .with_recovery_allocation_bytes(ORIGINAL)
                .unwrap(),
            AdmittedPhysicalRecordResidencyPolicy::canonical(format),
            None,
        )
        .unwrap();
    run(&mut coordination, &media, evidence);
}

fn with_empty_checkpoint<R>(
    store: store_namespace::StableStoreIdentity,
    run: impl FnOnce(ValidatedCheckpointStreamAssembly<'_, '_>) -> R,
) -> R {
    let identity = PhysicalCheckpointIdentity::new(store, NonZeroU64::new(1).unwrap());
    let source = PhysicalCheckpointSource::secured_concurrent(
        identity,
        CheckpointWalSourceRange::new(0, 2).unwrap(),
        CheckpointRootBasis::new(1, 1),
        0,
        [7; 32],
        1,
    )
    .unwrap();
    let (encoder, header_bytes) = CheckpointStreamEncoder::begin(source);
    let (encoder, compaction_bytes) =
        encoder.begin_binding_compaction(CheckpointBindingCompactionHeader::new(1, 2).unwrap());
    let (_, footer_bytes) = encoder.finish();
    let compaction_at = header_bytes.len();
    let footer_at = compaction_at + compaction_bytes.len();
    let mut bytes = header_bytes;
    bytes.extend_from_slice(&compaction_bytes);
    bytes.extend_from_slice(&footer_bytes);
    let input = |start, end| UntrustedPhysicalArtifact::from_bounded_bytes(&bytes[start..end]);
    let range = |start: usize, end: usize| {
        PhysicalByteRange::new(start as u64, (end - start) as u64).unwrap()
    };
    let CheckpointStreamHeaderIntegrityValidation::Intact(header) =
        validate_checkpoint_stream_header(
            input(0, compaction_at),
            PhysicalArtifactScope::checkpoint_stream_header(
                CheckpointStreamHeaderScopeIdentity::known(identity),
                range(0, compaction_at),
            ),
        )
        .0
    else {
        panic!("canonical header must admit");
    };
    let CheckpointBindingCompactionIntegrityValidation::Intact(compaction) =
        validate_checkpoint_binding_compaction(
            input(compaction_at, footer_at),
            PhysicalArtifactScope::checkpoint_binding_compaction(
                identity,
                range(compaction_at, footer_at),
            ),
        )
        .0
    else {
        panic!("canonical compaction must admit");
    };
    let CheckpointFooterIntegrityValidation::Intact(footer) = validate_checkpoint_footer(
        input(footer_at, bytes.len()),
        PhysicalArtifactScope::checkpoint_footer(identity, range(footer_at, bytes.len())),
        CheckpointFooterValidationBasis::from_record_references(&header, &[], &compaction, &[]),
    )
    .0
    else {
        panic!("exact C9 aggregate must admit");
    };
    run(
        VerifiedCheckpointStream::validate_records_with_certificates(
            input(0, bytes.len()),
            &header,
            &[],
            &compaction,
            &[],
            &[],
            &footer,
        )
        .unwrap(),
    )
}
