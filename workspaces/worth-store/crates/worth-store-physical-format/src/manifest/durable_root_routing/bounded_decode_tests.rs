use crate::record_framing::{decode_durable_frame, encode_durable_frame};
use crate::{
    CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement, DurableFrameKind,
    PersistedRecordIdentity, PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority,
    PhysicalRecordFormatDeclaration,
};

use super::{
    BoundedRootRoutingBlockDecodeDenial, PhysicalRootRoutingBlock, RootRoutingBlockDecodeLimits,
    RootRoutingBlockDenial, ROUTING_BLOCK_PREFIX_BYTES, ROUTING_LEAF_ENTRY_BYTES,
};

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

fn placement(ordinal: u64) -> CurrentPhysicalRecordPlacement {
    let record = PersistedRecordIdentity::new([9; 16], ordinal).unwrap();
    let extent = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(ordinal).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(1).unwrap());
    CurrentPhysicalRecordPlacement::Extent(
        DurableExtentRecordPlacement::new(
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
