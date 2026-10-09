use crate::record_framing::{
    decode_durable_frame, encode_durable_frame, encode_durable_frame_schema, SELECTED_ROUTE_SCHEMA,
};
use crate::{
    BlobRecordKind, CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement, DurableFrameKind,
    PersistedRecordIdentity, PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority,
    PhysicalRecordFormatDeclaration, SelectedRecordContentClass, SelectedRecordRouteMetadata,
};

use super::{
    BoundedRootRoutingBlockDecodeDenial, PhysicalRootRoutingBlock, RootRoutingBlockDecodeLimits,
    RootRoutingBlockDenial, RootRoutingCoordinateKey, ROUTING_BLOCK_PREFIX_BYTES,
    ROUTING_LEAF_ENTRY_BYTES,
};

#[path = "bounded_decode_tests/borrowed.rs"]
mod borrowed;

#[test]
fn leaf_cardinality_is_denied_before_crossing_entry_decoding() {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let block =
        PhysicalRootRoutingBlock::leaf(7, 1, 1, vec![placement(1), placement(2)], 2).unwrap();
    let bytes = block.encode(format);
    assert!(PhysicalRootRoutingBlock::decode_bounded(
        &bytes,
        2,
        RootRoutingBlockDecodeLimits {
            leaf_entries: 2,
            branch_children: 0,
        },
    )
    .is_ok());

    let (_, frame) = decode_durable_frame(&bytes, DurableFrameKind::RootRoutingBlock).unwrap();
    let mut payload = frame.payload.to_vec();
    payload[ROUTING_BLOCK_PREFIX_BYTES + ROUTING_LEAF_ENTRY_BYTES + 25] = 1;
    let damaged = encode_durable_frame(DurableFrameKind::RootRoutingBlock, format, 1, &payload);

    assert_eq!(
        PhysicalRootRoutingBlock::decode_bounded(
            &damaged,
            2,
            RootRoutingBlockDecodeLimits {
                leaf_entries: 1,
                branch_children: 0,
            },
        ),
        Err(BoundedRootRoutingBlockDecodeDenial::LeafEntries {
            observed: 2,
            admitted: 1,
        })
    );
    assert!(matches!(
        PhysicalRootRoutingBlock::decode_bounded(
            &damaged,
            2,
            RootRoutingBlockDecodeLimits {
                leaf_entries: 2,
                branch_children: 0,
            },
        ),
        Err(BoundedRootRoutingBlockDecodeDenial::Format(
            RootRoutingBlockDenial::Placement(_)
        ))
    ));
}

#[test]
fn selected_class_survives_schema_three_and_schema_two_remains_unknown() {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let legacy = placement(1);
    let classified = placement(2).with_route_metadata(
        SelectedRecordRouteMetadata::primary(SelectedRecordContentClass::Blob(
            BlobRecordKind::Chunk,
        ))
        .unwrap(),
    );
    let old = PhysicalRootRoutingBlock::leaf(7, 1, 1, vec![legacy], 2)
        .unwrap()
        .encode(format);
    assert_eq!(old[9], 2);
    let selected = PhysicalRootRoutingBlock::leaf(7, 2, 2, vec![classified], 2)
        .unwrap()
        .encode(format);
    assert_eq!(selected[9], SELECTED_ROUTE_SCHEMA);
    assert_eq!(old.len(), selected.len());
    let old_entry = PhysicalRootRoutingBlock::decode(&old, 2).unwrap().0;
    let selected_entry = PhysicalRootRoutingBlock::decode(&selected, 2).unwrap().0;
    assert_eq!(
        old_entry.entries().unwrap()[0].route_metadata(),
        SelectedRecordRouteMetadata::legacy_primary()
    );
    assert_eq!(
        selected_entry.entries().unwrap()[0].content_class(),
        SelectedRecordContentClass::Blob(BlobRecordKind::Chunk)
    );

    let (_, frame) = decode_durable_frame(&selected, DurableFrameKind::RootRoutingBlock).unwrap();
    let old_schema_with_class =
        encode_durable_frame(DurableFrameKind::RootRoutingBlock, format, 2, frame.payload);
    assert!(matches!(
        PhysicalRootRoutingBlock::decode(&old_schema_with_class, 2),
        Err(RootRoutingBlockDenial::Placement(_))
    ));
    let mut malformed = frame.payload.to_vec();
    malformed[ROUTING_BLOCK_PREFIX_BYTES + 29] = 255;
    let malformed = encode_durable_frame_schema(
        DurableFrameKind::RootRoutingBlock,
        format,
        2,
        &malformed,
        SELECTED_ROUTE_SCHEMA,
    );
    assert!(matches!(
        PhysicalRootRoutingBlock::decode(&malformed, 2),
        Err(RootRoutingBlockDenial::Placement(_))
    ));
}

#[test]
fn distinct_record_ids_cannot_share_one_physical_extent_coordinate() {
    assert_eq!(
        std::mem::size_of::<RootRoutingCoordinateKey>(),
        std::mem::size_of::<(u8, u64, u64, u64)>()
    );
    assert!(
        PhysicalRootRoutingBlock::leaf(7, 1, 1, vec![placement(1), placement_at(2, 1)], 2,)
            .is_none()
    );
    assert!(PhysicalRootRoutingBlock::leaf(7, 1, 1, vec![placement(1), placement(2)], 2).is_some());
}

#[test]
fn reserved_frame_backing_is_exact_and_decodes_canonically() {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let block = PhysicalRootRoutingBlock::leaf(7, 1, 1, vec![placement(1)], 2).unwrap();
    let required = block.encoded_frame_bytes().unwrap();
    assert!(block
        .encode_in_reserved(format, Vec::with_capacity(required - 1))
        .is_none());
    let bytes = block
        .encode_in_reserved(format, Vec::with_capacity(required))
        .unwrap();
    assert_eq!(bytes.len(), required);
    assert_eq!(bytes[8], DurableFrameKind::RootRoutingBlock as u8);
    assert_eq!(
        PhysicalRootRoutingBlock::decode(&bytes, 2),
        Ok((block, format))
    );
}

fn placement(ordinal: u64) -> CurrentPhysicalRecordPlacement {
    placement_at(ordinal, ordinal)
}

fn placement_at(ordinal: u64, extent_ordinal: u64) -> CurrentPhysicalRecordPlacement {
    let record = PersistedRecordIdentity::new([9; 16], ordinal).unwrap();
    let extent = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(extent_ordinal).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(1).unwrap());
    CurrentPhysicalRecordPlacement::Extent(
        DurableExtentRecordPlacement::legacy_unknown(
            record,
            extent,
            23,
            crate::ExtentArenaRange::new(
                crate::ExtentArenaId::new(1).unwrap(),
                (ordinal - 1) * 20480,
                20480,
            )
            .unwrap(),
        )
        .unwrap(),
    )
}
