//! Real registered namespace owners and allocation-free C9 assembly inputs.

use crate::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy,
    AdmittedRecoveryFilesystemMedia, FilesystemAccessPosture, FilesystemMediaAdmission,
    PhysicalRecoveryCoordination, PhysicalRecoveryCoordinationCapacity,
    PhysicalRecoveryFreshnessPort, PhysicalRuntimeAdmission, PhysicalStore,
    QualifiedRecoveryFilesystemMedia,
};
use std::{num::NonZeroU64, path::Path};
use worth_proof::TransitionOutcome;
use worth_store_physical_format::*;
use worth_store_physical_integrity::*;

pub(super) fn coordination() -> (
    tempfile::TempDir,
    AdmittedRecoveryFilesystemMedia,
    PhysicalRecoveryCoordination,
) {
    let root = tempfile::tempdir().unwrap();
    let runtime =
        PhysicalStore::admit(PhysicalRuntimeAdmission::new(root.path()).unwrap()).unwrap();
    let TransitionOutcome::Success(media) = runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    else {
        panic!("actual namespace initialization must admit");
    };
    media.close();
    let (media, owner) = reopen(root.path());
    (root, media, owner)
}

pub(super) fn reopen(
    root: &Path,
) -> (
    AdmittedRecoveryFilesystemMedia,
    PhysicalRecoveryCoordination,
) {
    let qualified = QualifiedRecoveryFilesystemMedia::qualify_existing(root).unwrap();
    let freshness = PhysicalRecoveryFreshnessPort::admit(&qualified).unwrap();
    let media = qualified.admit_persisted_store().unwrap();
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let owner = freshness
        .register_session()
        .unwrap()
        .admit_coordination(
            &media,
            PhysicalRecoveryCoordinationCapacity::admit(1, 4096, 1, 4096)
                .unwrap()
                .with_recovery_allocation_bytes(super::ORIGINAL)
                .unwrap(),
            AdmittedPhysicalRecordResidencyPolicy::canonical(format),
            None,
        )
        .unwrap();
    (media, owner)
}

pub(super) fn with_binding<R>(
    store: store_namespace::StableStoreIdentity,
    payload: Option<&[u8]>,
    run: impl FnOnce(
        ValidatedCheckpointStreamAssembly<'_, '_>,
        Option<&IntegrityValidatedCheckpointBinding<'_>>,
        Option<UntrustedPhysicalArtifact<'_>>,
    ) -> R,
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
    let payloads: Vec<_> = payload.into_iter().collect();
    with_bindings(source, &payloads, |assembly, bindings, records| {
        run(assembly, bindings.first(), records.first().copied())
    })
}

pub(super) fn with_bindings<R>(
    source: PhysicalCheckpointSource,
    payloads: &[&[u8]],
    run: impl FnOnce(
        ValidatedCheckpointStreamAssembly<'_, '_>,
        &[IntegrityValidatedCheckpointBinding<'_>],
        &[UntrustedPhysicalArtifact<'_>],
    ) -> R,
) -> R {
    let identity = source.identity();
    let (encoder, header_bytes) = CheckpointStreamEncoder::begin(source);
    let (mut encoder, compaction_bytes) =
        encoder.begin_binding_compaction(CheckpointBindingCompactionHeader::new(1, 2).unwrap());
    let binding_bytes: Vec<_> = payloads
        .iter()
        .map(|payload| encoder.encode_binding_record(payload).unwrap())
        .collect();
    let (_, footer_bytes) = encoder.finish();
    let compaction_at = header_bytes.len();
    let binding_at = compaction_at + compaction_bytes.len();
    let mut bytes = header_bytes;
    bytes.extend_from_slice(&compaction_bytes);
    let mut ranges = Vec::new();
    for binding in binding_bytes {
        let start = bytes.len();
        bytes.extend_from_slice(&binding);
        ranges.push((start, bytes.len()));
    }
    let footer_at = bytes.len();
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
        panic!("canonical C9 header must admit");
    };
    let CheckpointBindingCompactionIntegrityValidation::Intact(compaction) =
        validate_checkpoint_binding_compaction(
            input(compaction_at, binding_at),
            PhysicalArtifactScope::checkpoint_binding_compaction(
                identity,
                range(compaction_at, binding_at),
            ),
        )
        .0
    else {
        panic!("canonical C9 compaction must admit");
    };
    let bindings: Vec<_> = ranges
        .iter()
        .map(|&(start, end)| {
            let CheckpointBindingIntegrityValidation::Intact(binding) =
                validate_checkpoint_binding(
                    input(start, end),
                    PhysicalArtifactScope::checkpoint_binding(identity, range(start, end)),
                )
                .0
            else {
                panic!("issued-key history must have valid C9 framing");
            };
            binding
        })
        .collect();
    let references: Vec<_> = bindings.iter().collect();
    let CheckpointFooterIntegrityValidation::Intact(footer) = validate_checkpoint_footer(
        input(footer_at, bytes.len()),
        PhysicalArtifactScope::checkpoint_footer(identity, range(footer_at, bytes.len())),
        CheckpointFooterValidationBasis::from_record_references(
            &header,
            &[],
            &compaction,
            &references,
        ),
    )
    .0
    else {
        panic!("exact C9 aggregate must admit");
    };
    let assembly = VerifiedCheckpointStream::validate_records_with_certificates(
        input(0, bytes.len()),
        &header,
        &[],
        &compaction,
        &references,
        &[],
        &footer,
    )
    .unwrap();
    let exact: Vec<_> = ranges
        .iter()
        .map(|&(start, end)| input(start, end))
        .collect();
    run(assembly, &bindings, &exact)
}
