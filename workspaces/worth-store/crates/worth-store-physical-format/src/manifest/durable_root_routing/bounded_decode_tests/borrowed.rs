use super::super::{RootRoutingBlockPreflight, ROUTING_REFERENCE_BYTES};
use super::*;

fn limits() -> RootRoutingBlockDecodeLimits {
    RootRoutingBlockDecodeLimits {
        leaf_entries: 8,
        branch_children: 8,
    }
}

fn preflight<'a>(bytes: &'a [u8]) -> RootRoutingBlockPreflight<'a> {
    RootRoutingBlockPreflight::inspect_frame(bytes, 8, limits())
        .unwrap()
        .0
}

#[test]
fn borrowed_leaf_requires_exact_scratch_and_matches_owned_semantics() {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let block =
        PhysicalRootRoutingBlock::leaf(7, 1, 1, vec![placement(1), placement(2)], 8).unwrap();
    let encoded = block.encode(format);
    let preflight = preflight(&encoded);
    assert_eq!(preflight.coordinate_scratch_slots(), 2);
    assert!(matches!(
        preflight.validate(&mut Vec::new()),
        Err(
            BoundedRootRoutingBlockDecodeDenial::CoordinateScratchInsufficient {
                required: 2,
                provided: 0,
            }
        )
    ));
    let mut scratch = Vec::with_capacity(2);
    let view = preflight.validate(&mut scratch).unwrap();
    assert_eq!(
        view.entries().unwrap().collect::<Vec<_>>().as_slice(),
        block.entries().unwrap()
    );
    assert_eq!(view.reference(17), block.reference(17));
    assert_eq!(
        PhysicalRootRoutingBlock::decode_bounded(&encoded, 8, limits())
            .unwrap()
            .0,
        block
    );
}

#[test]
fn borrowed_branch_validates_late_reference_before_exposing_children() {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let left = PhysicalRootRoutingBlock::leaf(7, 1, 1, vec![placement(1)], 8).unwrap();
    let right = PhysicalRootRoutingBlock::leaf(7, 1, 2, vec![placement(2)], 8).unwrap();
    let branch = PhysicalRootRoutingBlock::branch(
        7,
        1,
        3,
        1,
        vec![left.reference(17), right.reference(19)],
        8,
    )
    .unwrap();
    let encoded = branch.encode(format);
    let view = preflight(&encoded).validate(&mut Vec::new()).unwrap();
    assert_eq!(
        view.children().unwrap().collect::<Vec<_>>().as_slice(),
        branch.children().unwrap()
    );

    let (_, frame) = decode_durable_frame(&encoded, DurableFrameKind::RootRoutingBlock).unwrap();
    let mut payload = frame.payload.to_vec();
    payload[ROUTING_BLOCK_PREFIX_BYTES + ROUTING_REFERENCE_BYTES + 18] = 1;
    let malformed = encode_durable_frame(DurableFrameKind::RootRoutingBlock, format, 3, &payload);
    assert!(matches!(
        preflight(&malformed).validate(&mut Vec::new()),
        Err(BoundedRootRoutingBlockDecodeDenial::Format(
            RootRoutingBlockDenial::InvalidReference
        ))
    ));
}

#[test]
fn borrowed_leaf_rejects_late_metadata_and_duplicate_coordinate() {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let classified = placement(2).with_route_metadata(
        SelectedRecordRouteMetadata::primary(SelectedRecordContentClass::Blob(
            BlobRecordKind::Chunk,
        ))
        .unwrap(),
    );
    let block = PhysicalRootRoutingBlock::leaf(7, 1, 1, vec![placement(1), classified], 8).unwrap();
    let encoded = block.encode(format);
    let (_, frame) = decode_durable_frame(&encoded, DurableFrameKind::RootRoutingBlock).unwrap();
    let mut payload = frame.payload.to_vec();
    payload[ROUTING_BLOCK_PREFIX_BYTES + ROUTING_LEAF_ENTRY_BYTES + 29] = 255;
    let malformed = encode_durable_frame_schema(
        DurableFrameKind::RootRoutingBlock,
        format,
        1,
        &payload,
        SELECTED_ROUTE_SCHEMA,
    );
    assert!(matches!(
        preflight(&malformed).validate(&mut Vec::with_capacity(2)),
        Err(BoundedRootRoutingBlockDecodeDenial::Format(
            RootRoutingBlockDenial::Placement(_)
        ))
    ));

    let ordinary =
        PhysicalRootRoutingBlock::leaf(7, 1, 1, vec![placement(1), placement(2)], 8).unwrap();
    let bytes = ordinary.encode(format);
    let (_, frame) = decode_durable_frame(&bytes, DurableFrameKind::RootRoutingBlock).unwrap();
    let mut payload = frame.payload.to_vec();
    let first = payload[ROUTING_BLOCK_PREFIX_BYTES + 32..ROUTING_BLOCK_PREFIX_BYTES + 80].to_vec();
    let second = ROUTING_BLOCK_PREFIX_BYTES + ROUTING_LEAF_ENTRY_BYTES;
    payload[second + 32..second + 80].copy_from_slice(&first);
    let duplicate = encode_durable_frame(DurableFrameKind::RootRoutingBlock, format, 1, &payload);
    assert!(matches!(
        preflight(&duplicate).validate(&mut Vec::with_capacity(2)),
        Err(BoundedRootRoutingBlockDecodeDenial::Format(
            RootRoutingBlockDenial::CanonicalOrder
        ))
    ));
}

#[test]
fn borrowed_view_preserves_accepted_schema_three_legacy_primary() {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let block = PhysicalRootRoutingBlock::leaf(7, 1, 1, vec![placement(1)], 8).unwrap();
    let encoded = block.encode(format);
    let (_, frame) = decode_durable_frame(&encoded, DurableFrameKind::RootRoutingBlock).unwrap();
    let selected_schema = encode_durable_frame_schema(
        DurableFrameKind::RootRoutingBlock,
        format,
        1,
        frame.payload,
        SELECTED_ROUTE_SCHEMA,
    );
    let view = preflight(&selected_schema)
        .validate(&mut Vec::with_capacity(1))
        .unwrap();
    assert_eq!(
        view.entries().unwrap().next().unwrap().content_class(),
        SelectedRecordContentClass::UnknownLegacy
    );
    assert_eq!(
        PhysicalRootRoutingBlock::decode(&selected_schema, 8)
            .unwrap()
            .0,
        block
    );
}
