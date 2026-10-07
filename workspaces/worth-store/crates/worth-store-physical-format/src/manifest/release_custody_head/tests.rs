use super::*;
use crate::record_framing::encode_durable_frame_schema;
use crate::{DurableFrameKind, PersistedRecordIdentity, PhysicalRecordFormatDeclaration};
use crate::{DurablePhysicalRootManifest, ReleasedDropPredecessorV1};
use sha2::{Digest, Sha256};

fn format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder().admit().unwrap()
}

fn entry(object: u8, generation: u64) -> ReleaseCustodyHeadEntryV1 {
    let record = |ordinal| PersistedRecordIdentity::new([object; 16], ordinal).unwrap();
    ReleaseCustodyHeadEntryV1::new(
        ReleaseCustodyHeadKeyV1::new([object; 16], generation).unwrap(),
        record(1),
        [1; 32],
        record(2),
        [2; 32],
        record(3),
        [3; 32],
        [4; 32],
        None,
        7,
        1,
        false,
    )
    .unwrap()
}

#[test]
fn head_block_codec_binds_exact_target_and_rejects_noncanonical_entries() {
    let block =
        ReleaseCustodyHeadBlockV1::leaf(9, 8, 1, vec![entry(1, 1), entry(2, 1)], format()).unwrap();
    let reference = block.reference(format());
    let bytes = block.encode(format());
    assert_eq!(
        ReleaseCustodyHeadBlockV1::decode(&bytes, reference, 9),
        Ok((block, format()))
    );
    assert_eq!(
        ReleaseCustodyHeadBlockV1::decode(&bytes, reference, 10),
        Err(ReleaseCustodyHeadDenial::Identity)
    );
    assert!(
        ReleaseCustodyHeadBlockV1::leaf(9, 8, 2, vec![entry(1, 1), entry(1, 1)], format()).is_err()
    );
    let mut payload = bytes[48..].to_vec();
    payload[0] = 2;
    let unknown = encode_durable_frame_schema(
        DurableFrameKind::ReleaseCustodyHeadBlock,
        format(),
        1,
        &payload,
        2,
    );
    assert_eq!(
        ReleaseCustodyHeadBlockV1::decode(&unknown, reference, 9),
        Err(ReleaseCustodyHeadDenial::UnsupportedVersion)
    );
    let same = PersistedRecordIdentity::new([1; 16], 1).unwrap();
    assert_eq!(
        ReleaseCustodyHeadEntryV1::new(
            entry(1, 1).key(),
            same,
            [1; 32],
            same,
            [2; 32],
            same,
            [3; 32],
            [4; 32],
            None,
            7,
            1,
            false
        ),
        Err(ReleaseCustodyHeadDenial::Malformed)
    );
}

#[test]
fn roster_commitment_rejects_duplicate_and_out_of_order_keys() {
    let block = ReleaseCustodyHeadBlockV1::leaf(9, 8, 1, vec![entry(1, 1)], format()).unwrap();
    let mut roster = ReleaseCustodyHeadRosterDigestV1::new(Some(block.reference(format())), 2);
    roster.push(entry(1, 1)).unwrap();
    assert_eq!(
        roster.push(entry(1, 1)),
        Err(ReleaseCustodyHeadDenial::CanonicalOrder)
    );
    assert_eq!(roster.finish().0, 1);
}

fn limits() -> ReleaseCustodyHeadTransitionLimitsV1 {
    ReleaseCustodyHeadTransitionLimitsV1::new(8, 16, 16 * 16_384).unwrap()
}

#[test]
fn head_root_schema_preserves_legacy_and_roundtrips_head_frontier() {
    let old = DurablePhysicalRootManifest::builder(8, 9, 2, 43)
        .admit()
        .unwrap();
    assert_eq!(old.encode(format())[9], 2);
    let decoded = DurablePhysicalRootManifest::decode(&old.encode(format()), 200)
        .unwrap()
        .0;
    assert_eq!(decoded.release_custody_head_root(), None);
    assert_eq!(decoded.next_release_custody_head_block(), 1);

    let block = ReleaseCustodyHeadBlockV1::leaf(9, 8, 1, vec![entry(1, 1)], format()).unwrap();
    let headed = DurablePhysicalRootManifest::builder(8, 9, 2, 43)
        .release_custody_head_root(Some(block.reference(format())))
        .next_release_custody_head_block(2)
        .admit()
        .unwrap()
        .with_maintenance_protocol();
    let encoded = headed.encode(format());
    assert_eq!(encoded.len(), 728);
    assert_eq!(encoded[9], 10);
    assert_eq!(
        DurablePhysicalRootManifest::decode(&encoded, 200)
            .unwrap()
            .0,
        headed
    );
    let unknown = encode_durable_frame_schema(
        DurableFrameKind::RootManifest,
        format(),
        8,
        &encoded[48..],
        11,
    );
    assert!(matches!(
        DurablePhysicalRootManifest::decode(&unknown, 200),
        Err(crate::RootManifestDenial::Frame(
            crate::DurableFrameDenial::UnsupportedSchema(11)
        ))
    ));
    let mut reserved = encoded[48..].to_vec();
    reserved[562] = 1;
    let malformed =
        encode_durable_frame_schema(DurableFrameKind::RootManifest, format(), 8, &reserved, 10);
    assert_eq!(
        DurablePhysicalRootManifest::decode(&malformed, 200),
        Err(crate::RootManifestDenial::ReservedFieldNonZero)
    );
}

#[test]
fn cow_transition_changes_only_one_key_with_exact_wal_node_writes() {
    let source =
        ReleaseCustodyHeadBlockV1::leaf(9, 8, 1, vec![entry(1, 1), entry(3, 1)], format()).unwrap();
    let source_ref = source.reference(format());
    let path = [ReleaseCustodyHeadPathNodeV1::new(
        source_ref,
        source.encode(format()),
    )];
    let mutation = ReleaseCustodyHeadMutationV1::Upsert {
        expected_prior: None,
        next: entry(2, 1),
    };
    let planned = ReleaseCustodyHeadTransitionV1::plan(
        Some(source_ref),
        2,
        &path,
        mutation,
        9,
        9,
        format(),
        limits(),
    )
    .unwrap();
    assert_eq!(planned.writes().len(), 1);
    assert_eq!(planned.writes()[0].reference().block(), 2);
    assert_eq!(planned.result_next_block(), 3);
    ReleaseCustodyHeadTransitionV1::verify_exact(
        Some(source_ref),
        2,
        &path,
        mutation,
        9,
        9,
        format(),
        limits(),
        planned.result_root(),
        planned.result_next_block(),
        planned.writes(),
    )
    .unwrap();
    let mut forged = planned.writes().to_vec();
    forged[0] = ReleaseCustodyHeadNodeWriteV1::new(forged[0].reference(), source.encode(format()));
    assert_eq!(
        ReleaseCustodyHeadTransitionV1::verify_exact(
            Some(source_ref),
            2,
            &path,
            mutation,
            9,
            9,
            format(),
            limits(),
            planned.result_root(),
            planned.result_next_block(),
            &forged
        ),
        Err(ReleaseCustodyHeadDenial::Mutation)
    );

    let prior = entry(1, 1);
    let successor = ReleaseCustodyHeadEntryV1::new(
        prior.key(),
        PersistedRecordIdentity::new([1; 16], 4).unwrap(),
        [5; 32],
        prior.manifest_record(),
        prior.manifest_frame_sha256(),
        prior.reservation_record(),
        prior.reservation_frame_sha256(),
        prior.source_basis_digest(),
        Some(
            ReleasedDropPredecessorV1::new(
                prior.descriptor_record(),
                prior.descriptor_frame_sha256(),
            )
            .unwrap(),
        ),
        8,
        2,
        false,
    )
    .unwrap();
    let advance = ReleaseCustodyHeadMutationV1::Upsert {
        expected_prior: Some(prior),
        next: successor,
    };
    assert!(ReleaseCustodyHeadTransitionV1::plan(
        Some(source_ref),
        2,
        &path,
        advance,
        9,
        9,
        format(),
        limits()
    )
    .is_ok());
}

#[test]
fn first_release_allocates_one_head_leaf_from_empty_tree() {
    let mutation = ReleaseCustodyHeadMutationV1::Upsert {
        expected_prior: None,
        next: entry(1, 1),
    };
    let planned =
        ReleaseCustodyHeadTransitionV1::plan(None, 1, &[], mutation, 8, 9, format(), limits())
            .unwrap();
    assert_eq!(planned.source_root(), None);
    assert_eq!(planned.result_root().unwrap().block(), 1);
    assert_eq!(planned.result_next_block(), 2);
    assert_eq!(planned.writes().len(), 1);
    assert!(ReleaseCustodyHeadTransitionV1::plan(
        None,
        1,
        &[],
        ReleaseCustodyHeadMutationV1::RetireTerminal {
            expected_prior: entry(1, 1),
        },
        8,
        9,
        format(),
        limits()
    )
    .is_err());
}

#[test]
fn cow_split_has_contiguous_new_head_blocks_and_no_allocator_choice() {
    let capacity = (format().page_size().bytes() as usize - 48 - 40)
        / ReleaseCustodyHeadEntryV1::ENCODED_BYTES;
    let source = ReleaseCustodyHeadBlockV1::leaf(
        9,
        8,
        1,
        (1..=capacity as u8)
            .map(|object| entry(object, 1))
            .collect(),
        format(),
    )
    .unwrap();
    let source_ref = source.reference(format());
    let path = [ReleaseCustodyHeadPathNodeV1::new(
        source_ref,
        source.encode(format()),
    )];
    let mutation = ReleaseCustodyHeadMutationV1::Upsert {
        expected_prior: None,
        next: entry((capacity + 1) as u8, 1),
    };
    let planned = ReleaseCustodyHeadTransitionV1::plan(
        Some(source_ref),
        2,
        &path,
        mutation,
        9,
        9,
        format(),
        limits(),
    )
    .unwrap();
    assert_eq!(planned.writes().len(), 3);
    assert_eq!(
        planned
            .writes()
            .iter()
            .map(|write| write.reference().block())
            .collect::<Vec<_>>(),
        vec![2, 3, 4]
    );
    assert_eq!(planned.result_root().unwrap().level(), 1);
    assert_eq!(planned.result_next_block(), 5);
}

fn reference_with_frame_sha(
    original: ReleaseCustodyHeadBlockReferenceV1,
    frame: &[u8],
) -> ReleaseCustodyHeadBlockReferenceV1 {
    ReleaseCustodyHeadBlockReferenceV1::new(
        original.generation(),
        original.block(),
        original.level(),
        original.first(),
        original.last(),
        Sha256::digest(frame).into(),
    )
    .unwrap()
}

#[test]
fn borrowed_head_view_validates_late_leaf_entry_before_exposing_items() {
    let block =
        ReleaseCustodyHeadBlockV1::leaf(9, 8, 1, vec![entry(1, 1), entry(2, 1)], format()).unwrap();
    let original = block.reference(format());
    let encoded = block.encode(format());
    let (view, decoded_format) =
        ReleaseCustodyHeadBlockViewV1::decode(&encoded, original, 9).unwrap();
    assert_eq!(decoded_format, format());
    assert_eq!(
        view.entries().unwrap().collect::<Vec<_>>().as_slice(),
        block.entries().unwrap()
    );

    let mut payload = encoded[48..].to_vec();
    payload[40 + ReleaseCustodyHeadEntryV1::ENCODED_BYTES + 305] = 1;
    let malformed = encode_durable_frame_schema(
        DurableFrameKind::ReleaseCustodyHeadBlock,
        format(),
        1,
        &payload,
        2,
    );
    let reference = reference_with_frame_sha(original, &malformed);
    assert!(matches!(
        ReleaseCustodyHeadBlockViewV1::decode(&malformed, reference, 9),
        Err(ReleaseCustodyHeadDenial::Malformed)
    ));
    assert_eq!(
        ReleaseCustodyHeadBlockV1::decode(&malformed, reference, 9),
        Err(ReleaseCustodyHeadDenial::Malformed)
    );
}

#[path = "tests/borrowed_branch.rs"]
mod borrowed_branch;
