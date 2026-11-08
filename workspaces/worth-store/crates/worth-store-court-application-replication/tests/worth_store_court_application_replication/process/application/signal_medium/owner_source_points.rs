//! Original source membership, historical leases and currentness are separate claims.
use super::{medium_fixture as media, owner_fixture as fixture};
use std::num::NonZeroU64;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::*;
use worth_signal::facade::durability::*;

fn point(artifact: &SignalBranchAdvanceArtifact, position: u64) -> SignalRecordedSourcePoint {
    SignalRecordedSourcePoint::from_recorded(
        artifact.reference(),
        NonZeroU64::new(position).unwrap(),
        artifact.descriptor().clone(),
    )
    .unwrap()
}

#[test]
fn historical_originals_readmit_without_claiming_currentness_and_live_points_reopen_exactly() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    let reference = fixture::seed(medium.as_ref(), &execution);
    let owner = fixture::readmit(medium.clone(), &execution);
    let port = owner.source_port();
    let initial = owner
        .read_original_advance(
            SignalBranchAdvanceIdentity::from_recorded(media::owner(), 1),
            &execution,
        )
        .unwrap();
    let historical = point(&initial, 1);
    assert!(matches!(
        port.readmit_descriptor(historical.descriptor()),
        Err(SignalDurableSourceDenial::DescriptorMismatch)
    ));
    let admitted = owner.readmit_source_point(&historical, &execution).unwrap();
    assert_eq!(admitted.original().as_bytes(), initial.as_bytes());
    assert_eq!(admitted.source_point(), &historical);
    let branch = historical.descriptor().branch();
    let head = port.readmit_descriptor(&reference.heads[&branch]).unwrap();
    assert_ne!(admitted.basis().observation(), head.basis().observation());
    assert_eq!(
        admitted.basis().observation().branch_id(),
        head.basis().observation().branch_id()
    );
    let current = owner
        .readmit_source_point(head.source_point(), &execution)
        .unwrap();
    assert_eq!(
        current.basis().admission_identity(),
        head.basis().admission_identity()
    );
    // A new effect has a genuine original receipt; neither historical admission
    // nor its transport codec writes or executes a transaction.
    let calls = std::sync::atomic::AtomicUsize::new(0);
    let output = fixture::finish(
        owner
            .mutation_port()
            .begin_advance(execution, &head, &mut (), |_| {
                calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Ok(())
            })
            .unwrap(),
        &execution,
    );
    let committed = output.evidence().source_point();
    assert_eq!(output.branch().source_point(), &committed);
    assert_eq!(committed.reference(), output.artifact().reference());
    assert_eq!(committed.position(), output.evidence().position());
    let bytes = committed.encode_bytes(&execution).unwrap();
    assert_eq!(
        SignalRecordedSourcePoint::readmit_bytes(&execution, bytes.bytes()).unwrap(),
        committed
    );
    assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 1);
    drop((current, admitted, initial, head, output, port, owner));
    media::close(serving, medium, &execution);
    let (serving, medium) = media::open_with_identity(
        directory.path(),
        false,
        2,
        "22222222-2222-4222-8222-222222222222",
        &execution,
    );
    let owner = fixture::readmit(medium.clone(), &execution);
    let head = owner
        .source_port()
        .readmit_descriptor(committed.descriptor())
        .unwrap();
    assert_eq!(head.source_point(), &committed);
    let admitted = owner.readmit_source_point(&historical, &execution).unwrap();
    assert_eq!(admitted.original().reference(), historical.reference());
    assert_eq!(
        owner
            .readmit_source_point(&committed, &execution)
            .unwrap()
            .basis()
            .observation(),
        head.basis().observation()
    );
    assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 1);
    drop((admitted, head, owner));
    media::close(serving, medium, &execution);
}

#[test]
fn source_point_admission_refuses_forged_locations_digests_descriptors_and_foreign_owners() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    let reference = fixture::seed(medium.as_ref(), &execution);
    let owner = fixture::readmit(medium.clone(), &execution);
    let original = owner
        .read_original_advance(
            SignalBranchAdvanceIdentity::from_recorded(media::owner(), 1),
            &execution,
        )
        .unwrap();
    let source = point(&original, 1);
    let mut digest = source.reference().digest();
    digest[0] ^= 1;
    let forged = [
        SignalRecordedSourcePoint::from_recorded(
            source.reference(),
            NonZeroU64::new(2).unwrap(),
            source.descriptor().clone(),
        )
        .unwrap(),
        SignalRecordedSourcePoint::from_recorded(
            SignalBranchAdvanceReference::from_recorded(source.reference().identity(), digest),
            NonZeroU64::new(1).unwrap(),
            source.descriptor().clone(),
        )
        .unwrap(),
        SignalRecordedSourcePoint::from_recorded(
            source.reference(),
            NonZeroU64::new(1).unwrap(),
            reference.heads[&source.descriptor().branch()].clone(),
        )
        .unwrap(),
    ];
    for point in forged {
        assert!(matches!(
            owner.readmit_source_point(&point, &execution),
            Err(SignalSourcePointReadmissionDenial::Original(
                SignalOwnerReadmissionDenial::SourcePointMismatch(_)
            ))
        ));
    }
    let foreign = DurableSignalOwnerIdentity::from_recorded(
        HomeIdentity::from_observed_bytes([9; 16]).unwrap(),
        SignalOwnerSlotIdentity::from_recorded(3),
    );
    let branch = DurableSignalBranchIdentity::from_recorded(foreign, 0);
    let descriptor = SignalDurableBranchDescriptor::from_recorded(
        DurableSignalGeneration::from_recorded(
            DurableSignalCellIncarnation::from_recorded(branch, 1),
            0,
        ),
        None,
        DurableSignalAdmissionIdentity::from_recorded(foreign, 1),
        SignalDefinitionTransitionReference::from_recorded(
            SignalDefinitionTransitionIdentity::from_recorded(foreign, 1),
            [0; 32],
        ),
    )
    .unwrap();
    let point = SignalRecordedSourcePoint::from_recorded(
        SignalBranchAdvanceReference::from_recorded(
            SignalBranchAdvanceIdentity::from_recorded(foreign, 1),
            [0; 32],
        ),
        NonZeroU64::new(1).unwrap(),
        descriptor,
    )
    .unwrap();
    assert!(matches!(
        owner.readmit_source_point(&point, &execution),
        Err(SignalSourcePointReadmissionDenial::Original(
            SignalOwnerReadmissionDenial::ForeignOwner
        ))
    ));
    drop(owner);
    media::close(serving, medium, &execution);
}

#[test]
fn source_point_codec_refuses_unknown_versions_and_checksum_damage() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    fixture::seed(medium.as_ref(), &execution);
    let owner = fixture::readmit(medium.clone(), &execution);
    let original = owner
        .read_original_advance(
            SignalBranchAdvanceIdentity::from_recorded(media::owner(), 1),
            &execution,
        )
        .unwrap();
    let bytes = point(&original, 1).encode_bytes(&execution).unwrap();
    let mut damaged = bytes.bytes().to_vec();
    damaged[8..12].copy_from_slice(&999u32.to_le_bytes());
    assert!(matches!(
        SignalRecordedSourcePoint::readmit_bytes(&execution, &damaged),
        Err(SignalDurableFrameDenial::UnsupportedVersion(_))
    ));
    let mut damaged = bytes.bytes().to_vec();
    *damaged.last_mut().unwrap() ^= 1;
    assert!(matches!(
        SignalRecordedSourcePoint::readmit_bytes(&execution, &damaged),
        Err(SignalDurableFrameDenial::ChecksumMismatch)
    ));
    drop((original, bytes, owner));
    media::close(serving, medium, &execution);
}
