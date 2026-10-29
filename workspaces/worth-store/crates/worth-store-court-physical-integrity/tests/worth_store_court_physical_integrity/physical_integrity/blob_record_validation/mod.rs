#[allow(unused_imports)]
use crate::blob_record_validation::*;

#[test]
fn intact_chunk_is_bound_to_exact_record_scope_and_input_incarnation() {
    let bytes = chunk_bytes();
    let scope = chunk_scope(&bytes, record(1));
    assert_eq!(
        scope.blob_record_identity(),
        Some((
            record(1),
            worth_store_physical_format::BlobRecordKind::Chunk
        ))
    );
    let input =
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (outcome, counters) = validate_blob_record(input, scope);
    let worth_store_physical_integrity::BlobRecordIntegrityValidation::Intact(validated) = outcome
    else {
        panic!("valid chunk inner payload must validate");
    };
    assert_eq!(
        validated.record().kind(),
        worth_store_physical_format::BlobRecordKind::Chunk
    );
    assert!(validated.matches_input(input));
    let copied = bytes.clone();
    assert!(!validated.matches_input(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&copied)
    ));
    let evidence = validated.into_validation_record();
    assert!(evidence.matches_scope(scope));
    assert!(!evidence.matches_scope(chunk_scope(&bytes, record(2))));
    assert_eq!(counters.inspected_frames(), 1);
    assert_eq!(counters.intact_frames(), 1);
}

#[test]
fn inner_chunk_digest_failure_is_damage_even_with_resealed_envelope() {
    let mut bytes = chunk_bytes();
    let last = bytes.len() - 1;
    bytes[last] ^= 1;
    let mut digest = sha2::Sha256::new();
    digest.update(&bytes[..16]);
    digest.update(&bytes[48..]);
    bytes[16..48].copy_from_slice(&digest.finalize());
    let scope = chunk_scope(&bytes, record(1));
    let (outcome, _) = validate_blob_record(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes),
        scope,
    );
    let worth_store_physical_integrity::BlobRecordIntegrityValidation::Rejected(
        worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(damage),
    ) = outcome
    else {
        panic!("inner digest failure must be damage");
    };
    assert_eq!(
        damage.cause(),
        worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch
    );
    assert_eq!(damage.scope(), scope);
}

#[test]
fn wrong_kind_store_unsupported_version_and_truncation_remain_distinct() {
    let bytes = chunk_bytes();
    let wrong_kind = worth_store_physical_integrity::PhysicalArtifactScope::blob_tree_node(
        store(3),
        record(1),
        range(&bytes),
    );
    let (outcome, _) = validate_blob_record(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes),
        wrong_kind,
    );
    let worth_store_physical_integrity::BlobRecordIntegrityValidation::Rejected(
        worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(damage),
    ) = outcome
    else {
        panic!("kind mismatch must be damage");
    };
    assert_eq!(
        damage.cause(),
        worth_store_physical_integrity::PhysicalDamageCause::RecordKindMismatch
    );

    let wrong_store = worth_store_physical_integrity::PhysicalArtifactScope::blob_chunk_frame(
        store(9),
        record(1),
        range(&bytes),
    );
    let (outcome, _) = validate_blob_record(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes),
        wrong_store,
    );
    let worth_store_physical_integrity::BlobRecordIntegrityValidation::Rejected(
        worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(damage),
    ) = outcome
    else {
        panic!("store mismatch must be damage");
    };
    assert_eq!(
        damage.cause(),
        worth_store_physical_integrity::PhysicalDamageCause::StoreIdentityMismatch
    );

    let mut future = bytes.clone();
    future[9] = 2;
    let (outcome, _) = validate_blob_record(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&future),
        chunk_scope(&future, record(1)),
    );
    let worth_store_physical_integrity::BlobRecordIntegrityValidation::Rejected(
        worth_store_physical_integrity::PhysicalIntegrityRejection::Unsupported(version),
    ) = outcome
    else {
        panic!("future version must be unsupported");
    };
    assert_eq!(
        version.axis(),
        worth_store_physical_integrity::PhysicalIntegrityVersionAxis::BlobRecord
    );
    assert_eq!(version.observed(), 2);

    let (outcome, _) = validate_blob_record(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
            &bytes[..bytes.len() - 1],
        ),
        chunk_scope(&bytes, record(1)),
    );
    let worth_store_physical_integrity::BlobRecordIntegrityValidation::Rejected(
        worth_store_physical_integrity::PhysicalIntegrityRejection::Damaged(damage),
    ) = outcome
    else {
        panic!("short input must be damaged");
    };
    assert_eq!(
        damage.cause(),
        worth_store_physical_integrity::PhysicalDamageCause::Truncated
    );
}

#[test]
fn tree_and_publication_validate_through_scrub_dispatch() {
    let entry =
        worth_store_physical_format::BlobTreeEntryV1::new([8; 32], record(1), 64 << 10).unwrap();
    let node = worth_store_physical_format::BlobTreeNodeV1::new(
        worth_store_physical_format::BlobTreeOccurrenceV1::new(
            [3; 16],
            [4; 16],
            worth_store_physical_format::BlobTreeNodeKind::Leaf,
            0,
            0,
        )
        .unwrap(),
        vec![entry],
    )
    .unwrap();
    let tree_bytes = node.encode();
    let tree_scope = worth_store_physical_integrity::PhysicalArtifactScope::blob_tree_node(
        store(3),
        record(2),
        range(&tree_bytes),
    );
    let (inspection, counters) = inspect_physical_integrity_window(
        worth_store_physical_integrity::PhysicalIntegrityScrubWindow::new(
            0,
            tree_scope,
            worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
                &tree_bytes,
            ),
        ),
    );
    assert_eq!(inspection.outcome().scope(), tree_scope);
    assert_eq!(counters.intact_frames(), 1);

    let publication = worth_store_physical_format::BlobGenerationPublicationV1::new(
        [3; 16],
        [4; 16],
        [6; 16],
        1,
        record(2),
        node.frame_digest(),
        64 << 10,
        [9; 32],
        64 << 10,
        [10; 32],
    )
    .unwrap();
    let publication_bytes = publication.encode();
    let publication_scope =
        worth_store_physical_integrity::PhysicalArtifactScope::blob_generation_publication(
            store(3),
            record(3),
            range(&publication_bytes),
        );
    let (inspection, counters) = inspect_physical_integrity_window(
        worth_store_physical_integrity::PhysicalIntegrityScrubWindow::new(
            1,
            publication_scope,
            worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
                &publication_bytes,
            ),
        ),
    );
    assert_eq!(inspection.outcome().scope(), publication_scope);
    assert_eq!(counters.intact_frames(), 1);
}
