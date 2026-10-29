
    #[allow(unused_imports)] use crate::phase_eight_process::history::parent_oracle::canonical_membership_frame::routing_schema_tests::*;
use {crate::phase_eight_process::history::parent_oracle::canonical_membership_placement::parse_placement, crate::phase_eight_process::history::parent_oracle::canonical_membership_placement::Placement, crate::phase_eight_process::history::parent_oracle::canonical_membership_placement::RecordIdentity};
    use {crate::phase_eight_process::history::parent_oracle::canonical_membership_frame::crc32c, crate::phase_eight_process::history::parent_oracle::canonical_membership_frame::decode_frame, crate::phase_eight_process::history::parent_oracle::canonical_membership_frame::FRAME_BYTES};
    use {worth_store_physical_format::CurrentPhysicalRecordPlacement, worth_store_physical_format::DurableExtentRecordPlacement, worth_store_physical_format::ExtentArenaId, worth_store_physical_format::ExtentArenaRange, worth_store_physical_format::PersistedRecordIdentity, worth_store_physical_format::PhysicalExtentId, worth_store_physical_format::PhysicalGeneration, worth_store_physical_format::PhysicalGenerationAuthority, worth_store_physical_format::PhysicalRecordFormatDeclaration, worth_store_physical_format::PhysicalRootRoutingBlock, worth_store_physical_format::SelectedRecordContentClass, worth_store_physical_format::SelectedRecordRouteMetadata};

    #[test]
    fn selected_routing_leaf_preserves_exact_extent_placement() {
        let bytes = selected_extent_leaf();
        let frame = decode_frame(&bytes).expect("a format-encoded selected leaf is framed");
        assert_eq!((frame.kind, frame.schema, frame.identity), (8, 3, 7));
        let placement = parse_placement(&frame.payload[40..128], frame.schema).unwrap();
        assert!(matches!(
            placement,
            crate::phase_eight_process::history::parent_oracle::canonical_membership_placement::Placement::Extent {
                record: RecordIdentity {
                    allocation_epoch,
                    ordinal: 5,
                },
                arena: 3,
                extent: 11,
                generation: 2,
                arena_offset: 4096,
                arena_length: 8192,
                payload_bytes: 23,
            } if allocation_epoch == [9; 16]
        ));
    }

    #[test]
    fn selected_route_metadata_is_schema_bound_and_integrity_checked() {
        let bytes = selected_extent_leaf();
        let mut corrupt = bytes.clone();
        corrupt[ENTRY_OFFSET + 25] ^= 1;
        assert!(decode_frame(&corrupt).is_none());

        let mut malformed = bytes.clone();
        malformed[ENTRY_OFFSET + 30] = 1;
        reseal(&mut malformed);
        let frame = decode_frame(&malformed).expect("malformed metadata still has valid framing");
        assert!(parse_placement(&frame.payload[40..128], frame.schema).is_err());

        let mut legacy = bytes.clone();
        legacy[9] = 2;
        reseal(&mut legacy);
        let frame = decode_frame(&legacy).expect("schema two still has valid framing");
        assert!(parse_placement(&frame.payload[40..128], frame.schema).is_err());

        let mut wrong_kind = bytes;
        wrong_kind[8] = 3;
        reseal(&mut wrong_kind);
        assert!(decode_frame(&wrong_kind).is_none());
    }
