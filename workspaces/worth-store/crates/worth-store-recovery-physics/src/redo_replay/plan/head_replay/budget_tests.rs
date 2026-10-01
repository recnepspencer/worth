use super::*;
use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobGenerationPublicationV1, BlobReclaimSourceBasisV1, PersistedRecordIdentity,
    ReleaseCustodyHeadBlockV1, ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1,
    ReleaseCustodyHeadMutationV1, ReleaseCustodyHeadTransitionLimitsV1,
    ReleaseCustodyHeadTransitionV1, ReleasedGenerationReclaimBasisV1,
};

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([1; 16], ordinal).unwrap()
}

fn fixture() -> (
    PersistedReleaseCustodyHeadEffectV1,
    DurablePhysicalRootManifest,
    PhysicalRecordFormatDeclaration,
    Vec<u8>,
) {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let source_entry = ReleaseCustodyHeadEntryV1::new(
        ReleaseCustodyHeadKeyV1::new([1; 16], 1).unwrap(),
        record(20),
        [1; 32],
        record(21),
        [2; 32],
        record(22),
        [3; 32],
        [4; 32],
        None,
        10,
        1,
        false,
    )
    .unwrap();
    let source_block =
        ReleaseCustodyHeadBlockV1::leaf(6, 10, 1, vec![source_entry], format).unwrap();
    let source_ref = source_block.reference(format);
    let source_frame = source_block.encode(format);
    let source_path = vec![ReleaseCustodyHeadPathNodeV1::new(
        source_ref,
        source_frame.clone(),
    )];
    let publication = BlobGenerationPublicationV1::new(
        [7; 16],
        [2; 16],
        [3; 16],
        3,
        record(3),
        [4; 32],
        8,
        [5; 32],
        64 * 1024,
        [6; 32],
    )
    .unwrap();
    let basis = ReleasedGenerationReclaimBasisV1::new(
        publication,
        record(4),
        Sha256::digest(publication.encode()).into(),
        [6; 32],
    )
    .unwrap();
    let next = ReleaseCustodyHeadEntryV1::new(
        ReleaseCustodyHeadKeyV1::new(basis.object(), basis.generation()).unwrap(),
        record(9),
        [5; 32],
        record(7),
        [8; 32],
        record(8),
        [9; 32],
        BlobReclaimSourceBasisV1::ReleasedGeneration(basis).digest(publication.store()),
        None,
        11,
        1,
        false,
    )
    .unwrap();
    let limits = ReleaseCustodyHeadTransitionLimitsV1::new(1, 3, 64 * 1024).unwrap();
    let planned = ReleaseCustodyHeadTransitionV1::plan(
        Some(source_ref),
        2,
        &source_path,
        ReleaseCustodyHeadMutationV1::Upsert {
            expected_prior: None,
            next,
        },
        12,
        6,
        format,
        limits,
    )
    .unwrap();
    let effect = PersistedReleaseCustodyHeadEffectV1::new_upsert(
        6,
        11,
        basis,
        source_path,
        planned,
        format,
        limits,
    )
    .unwrap();
    let source = DurablePhysicalRootManifest::builder(11, 6, 4, 1)
        .release_custody_head_root(Some(source_ref))
        .next_release_custody_head_block(2)
        .admit()
        .unwrap();
    (effect, source, format, source_frame)
}

fn group() -> PhysicalRedoGroupBinding {
    PhysicalRedoGroupBinding::new([1; 32], [2; 32], 1, 1, [3; 32]).unwrap()
}

#[test]
fn insufficient_extra_heap_denies_before_selected_path_read() {
    let (effect, source, format, _) = fixture();
    let mut reads = 0;
    let denial = VerifiedSelectedReleaseHeadReplayV14::admit_effect(
        Some(&effect),
        &source,
        format,
        64 * 1024,
        0,
        &mut |_, _| {
            reads += 1;
            Ok::<Vec<u8>, ()>(Vec::new())
        },
        None,
        [4; 32],
        group(),
        RecoveryOperationFate::Indeterminate,
        [5; 32],
    );
    assert_eq!(denial, Err(SelectedReleaseHeadReplayDenial::BoundExceeded));
    assert_eq!(reads, 0);
}

#[test]
fn sufficient_heap_accepts_exact_path_and_wrong_source_still_denies() {
    let (effect, source, format, source_frame) = fixture();
    let budget = effect
        .verification_additional_peak_bytes(format)
        .unwrap()
        .max(effect.owned_heap_bytes().unwrap())
        .max(u64::from(format.page_size().bytes()));
    let mut reads = 0;
    let replay = VerifiedSelectedReleaseHeadReplayV14::admit_effect(
        Some(&effect),
        &source,
        format,
        64 * 1024,
        budget,
        &mut |_, _| {
            reads += 1;
            Ok::<Vec<u8>, ()>(source_frame.clone())
        },
        None,
        [4; 32],
        group(),
        RecoveryOperationFate::Indeterminate,
        [5; 32],
    )
    .unwrap();
    assert_eq!(reads, 1);
    assert_eq!(replay.effect(), &effect);

    let wrong_source = DurablePhysicalRootManifest::builder(11, 6, 4, 1)
        .release_custody_head_root(source.release_custody_head_root())
        .next_release_custody_head_block(3)
        .admit()
        .unwrap();
    let mut wrong_reads = 0;
    let denial = VerifiedSelectedReleaseHeadReplayV14::admit_effect(
        Some(&effect),
        &wrong_source,
        format,
        64 * 1024,
        budget,
        &mut |_, _| {
            wrong_reads += 1;
            Ok::<Vec<u8>, ()>(source_frame.clone())
        },
        None,
        [4; 32],
        group(),
        RecoveryOperationFate::Indeterminate,
        [5; 32],
    );
    assert_eq!(denial, Err(SelectedReleaseHeadReplayDenial::SourceRoot));
    assert_eq!(wrong_reads, 0);
}
