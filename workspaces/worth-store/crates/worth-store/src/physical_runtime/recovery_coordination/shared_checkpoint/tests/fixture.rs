//! Genuine registered Coordination and Integrity-admitted checkpoint records.

use super::*;
use worth_proof::TransitionOutcome;
use worth_store_physical_format::{
    CheckpointBindingCompactionHeader, CheckpointCertificateKind, CheckpointRootBasis,
    CheckpointStreamEncoder, CheckpointWalSourceRange, PhysicalCheckpointIdentity,
    PhysicalCheckpointSource, ReleaseCheckpointNoReleaseV1,
};
use worth_store_physical_integrity::*;

pub(super) fn coordination() -> (
    tempfile::TempDir,
    crate::physical_runtime::AdmittedRecoveryFilesystemMedia,
    PhysicalRecoveryCoordination,
) {
    use crate::physical_runtime::{
        AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy,
        FilesystemAccessPosture, FilesystemMediaAdmission, PhysicalRecoveryCoordinationCapacity,
        PhysicalRecoveryFreshnessPort, PhysicalRuntimeAdmission, PhysicalStore,
        QualifiedRecoveryFilesystemMedia,
    };
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let runtime =
        PhysicalStore::admit(PhysicalRuntimeAdmission::new(root.clone()).unwrap()).unwrap();
    let media = match runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    {
        TransitionOutcome::Success(media) => media,
        _ => panic!("genuine Store namespace initialization failed"),
    };
    let _ = media.close();
    let qualified = QualifiedRecoveryFilesystemMedia::qualify_existing(&root).unwrap();
    let freshness = PhysicalRecoveryFreshnessPort::admit(&qualified).unwrap();
    let media = qualified.admit_persisted_store().unwrap();
    let format = AdmittedPhysicalRecordFormat::admit(
        worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    let capacity = PhysicalRecoveryCoordinationCapacity::admit(2, 4096, 2, 4096)
        .unwrap()
        .with_recovery_allocation_bytes(16 << 10)
        .unwrap();
    let owner = freshness
        .register_session()
        .unwrap()
        .admit_coordination(
            &media,
            capacity,
            AdmittedPhysicalRecordResidencyPolicy::canonical(format),
            None,
        )
        .unwrap();
    (parent, media, owner)
}

pub(super) fn with_assembly<R>(
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
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
    let marker =
        ReleaseCheckpointNoReleaseV1::new(identity, 1, [9; 32], 0, [0; 32], [0; 32]).unwrap();
    let (encoder, header_bytes) = CheckpointStreamEncoder::begin_certified(source);
    let (mut encoder, compaction_bytes) =
        encoder.begin_binding_compaction(CheckpointBindingCompactionHeader::new(1, 2).unwrap());
    let certificate = encoder
        .encode_certificate_record(CheckpointCertificateKind::ReleasedDrop, &marker.encode())
        .unwrap();
    let (_, footer_bytes) = encoder.finish();
    let compaction_at = header_bytes.len();
    let certificate_at = compaction_at + compaction_bytes.len();
    let footer_at = certificate_at + certificate.len();
    let bytes = [header_bytes, compaction_bytes, certificate, footer_bytes].concat();
    let header_input = UntrustedPhysicalArtifact::from_bounded_bytes(&bytes[..compaction_at]);
    let CheckpointStreamHeaderIntegrityValidation::Intact(header) =
        validate_checkpoint_stream_header(
            header_input,
            PhysicalArtifactScope::checkpoint_stream_header(
                CheckpointStreamHeaderScopeIdentity::known(identity),
                range(0, compaction_at),
            ),
        )
        .0
    else {
        panic!("fixture checkpoint header rejected");
    };
    let CheckpointBindingCompactionIntegrityValidation::Intact(compaction) =
        validate_checkpoint_binding_compaction(
            UntrustedPhysicalArtifact::from_bounded_bytes(&bytes[compaction_at..certificate_at]),
            PhysicalArtifactScope::checkpoint_binding_compaction(
                identity,
                range(compaction_at, certificate_at - compaction_at),
            ),
        )
        .0
    else {
        panic!("fixture binding compaction rejected");
    };
    let certificates = [(
        range(certificate_at, footer_at - certificate_at),
        &bytes[certificate_at..footer_at],
    )];
    let CheckpointFooterIntegrityValidation::Intact(footer) = validate_checkpoint_footer(
        UntrustedPhysicalArtifact::from_bounded_bytes(&bytes[footer_at..]),
        PhysicalArtifactScope::checkpoint_footer(
            identity,
            range(footer_at, bytes.len() - footer_at),
        ),
        CheckpointFooterValidationBasis::from_record_references(&header, &[], &compaction, &[])
            .with_certificates(&certificates),
    )
    .0
    else {
        panic!("fixture checkpoint footer rejected");
    };
    let assembly = VerifiedCheckpointStream::validate_records_with_certificates(
        UntrustedPhysicalArtifact::from_bounded_bytes(&bytes),
        &header,
        &[],
        &compaction,
        &[],
        &certificates,
        &footer,
    )
    .unwrap();
    run(assembly)
}

fn range(offset: usize, bytes: usize) -> PhysicalByteRange {
    PhysicalByteRange::new(offset as u64, bytes as u64).unwrap()
}
