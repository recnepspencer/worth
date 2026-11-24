//! Law-based property suites for Store physical format binary codecs:
//! WAL v1 frames, inline record pages, extent chunks, and durable root selectors.
use proptest::prelude::*;
use worth_store_physical_format::{
    append_inline_records_owned, decode_extent_chunk, decode_inline_record, encode_extent_chunk,
    encode_inline_page, inspect_inline_page, restamp_inline_page_generation,
    store_namespace::{
        ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord,
        StoreNamespaceVersion,
    },
    wal_frame::{
        decode_bounded_wal_frame_v1, decode_wal_frame_v1_header, encode_wal_frame_v1,
        wal_frame_v1_declared_identity_digest, WalFrameV1EncodeRequest, WAL_FRAME_V1_FOOTER_BYTES,
        WAL_FRAME_V1_HEADER_BYTES,
    },
    DurableRootSelector, ExtentChunkCoordinate, InlineRecordAppend, PersistedRecordIdentity,
    PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageId,
    PhysicalRecordFormatDeclaration, PhysicalRecordSlot, PhysicalSegmentId, RootSelectorIdentity,
    RootSelectorRole, DURABLE_FRAME_HEADER_BYTES, DURABLE_INLINE_PAGE_PREFIX_BYTES,
    DURABLE_INLINE_SLOT_BYTES, ROOT_SELECTOR_BYTES,
};

fn canonical_format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder()
        .admit()
        .expect("canonical format admits")
}

fn nonzero_u8_16() -> impl Strategy<Value = [u8; 16]> {
    prop::array::uniform16(any::<u8>()).prop_filter("nonzero 16-byte id", |b| *b != [0; 16])
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn wal_frame_v1_roundtrips_and_rejects_any_bit_flip_or_truncation(
        segment_id in 1_u64..=u64::MAX,
        generation in 1_u64..=u64::MAX,
        lsn_start in 0_u64..(u64::MAX - 10_000),
        lsn_delta in 1_u64..=10_000,
        declared_identity in prop::collection::vec(any::<u8>(), 0..=48),
        payload in prop::collection::vec(any::<u8>(), 1..=512),
        flip_pos in any::<usize>(),
        flip_bit in 0_u8..8,
        trunc_pick in any::<usize>(),
        trailing in any::<u8>(),
    ) {
        let lsn_end = lsn_start + lsn_delta;
        let req = WalFrameV1EncodeRequest::new(
            segment_id,
            generation,
            lsn_start,
            lsn_end,
            &declared_identity,
            &payload,
        )
        .expect("valid WAL encode request");
        let encoded = encode_wal_frame_v1(req);
        prop_assert_eq!(
            encoded.len(),
            WAL_FRAME_V1_HEADER_BYTES + payload.len() + WAL_FRAME_V1_FOOTER_BYTES
        );

        let decoded = decode_bounded_wal_frame_v1(&encoded).expect("valid WAL frame decodes");
        let hdr = decoded.header();
        prop_assert_eq!(hdr.segment_id(), segment_id);
        prop_assert_eq!(hdr.generation(), generation);
        prop_assert_eq!(hdr.lsn_start(), lsn_start);
        prop_assert_eq!(hdr.lsn_end(), lsn_end);
        prop_assert_eq!(hdr.payload_bytes(), payload.len() as u64);
        prop_assert_eq!(
            hdr.identity_digest(),
            wal_frame_v1_declared_identity_digest(&declared_identity)
        );
        prop_assert_eq!(decoded.payload(), payload.as_slice());

        let raw_hdr: &[u8; WAL_FRAME_V1_HEADER_BYTES] = encoded[..WAL_FRAME_V1_HEADER_BYTES]
            .try_into()
            .unwrap();
        prop_assert_eq!(decode_wal_frame_v1_header(raw_hdr).expect("header decodes"), hdr);

        // Any single-bit flip anywhere in header, payload, or footer must be rejected.
        let mut corrupted = encoded.clone();
        let idx = flip_pos % corrupted.len();
        corrupted[idx] ^= 1_u8 << flip_bit;
        prop_assert!(
            decode_bounded_wal_frame_v1(&corrupted).is_err(),
            "single-bit flip at offset {idx} bit {flip_bit} must fail decode"
        );

        // Any strict truncation must be rejected.
        let trunc_len = trunc_pick % encoded.len();
        prop_assert!(decode_bounded_wal_frame_v1(&encoded[..trunc_len]).is_err());

        // Trailing bytes must be rejected.
        let mut extended = encoded;
        extended.push(trailing);
        prop_assert!(decode_bounded_wal_frame_v1(&extended).is_err());
    }

    #[test]
    fn inline_page_append_restamp_and_corruption_laws(
        seg_raw in 1_u64..=10_000,
        page_raw in 1_u64..=10_000,
        gen_raw in 1_u64..=1_000_000,
        restamp_delta in 1_u64..=1_000,
        epoch in nonzero_u8_16(),
        payloads in prop::collection::vec(prop::collection::vec(any::<u8>(), 1..=96), 1..=8),
        split_at in 0_usize..=8,
        flip_pos in any::<usize>(),
        flip_bit in 0_u8..8,
        trunc_pick in any::<usize>(),
    ) {
        let format = canonical_format();
        let auth = PhysicalGenerationAuthority::for_canonical_physical_format();
        let seg = PhysicalSegmentId::from_raw(seg_raw).unwrap();
        let page_id = PhysicalPageId::from_raw(page_raw).unwrap();
        let initial_cell = auth
            .page_cell(seg, page_id)
            .with_page_generation(PhysicalGeneration::from_raw(gen_raw).unwrap());

        let records_meta: Vec<_> = payloads
            .iter()
            .enumerate()
            .map(|(i, bytes)| {
                let rec = PersistedRecordIdentity::new(epoch, (i as u64) + 1).unwrap();
                let slot = PhysicalRecordSlot::from_raw((i as u16) + 1).unwrap();
                let slot_cell = auth
                    .slot_cell(seg, page_id, slot)
                    .with_slot_generation(PhysicalGeneration::from_raw((i as u64) + 10).unwrap());
                (rec, slot_cell, bytes.as_slice())
            })
            .collect();

        let split = (split_at % records_meta.len()).max(1);
        let first_batch: Vec<_> = records_meta[..split]
            .iter()
            .map(|(rec, slot, bytes)| InlineRecordAppend::new(*rec, *slot, bytes))
            .collect();
        let mut page = encode_inline_page(format, initial_cell, &first_batch)
            .expect("initial inline page encodes");
        let mut active_cell = initial_cell;

        if split < records_meta.len() {
            let next_cell = auth
                .page_cell(seg, page_id)
                .with_page_generation(PhysicalGeneration::from_raw(gen_raw + 1).unwrap());
            let second_batch: Vec<_> = records_meta[split..]
                .iter()
                .map(|(rec, slot, bytes)| InlineRecordAppend::new(*rec, *slot, bytes))
                .collect();
            let (updated, appended) =
                append_inline_records_owned(format, next_cell, Some(page), &second_batch)
                    .expect("incremental append succeeds");
            prop_assert_eq!(appended.len(), records_meta.len() - split);
            page = updated;
            active_cell = next_cell;
        }

        let geom = inspect_inline_page(format, &page).expect("sealed page inspects");
        let payload_sum: usize = payloads.iter().map(Vec::len).sum();
        let expected_free = page.len()
            - DURABLE_FRAME_HEADER_BYTES
            - DURABLE_INLINE_PAGE_PREFIX_BYTES
            - records_meta.len() * DURABLE_INLINE_SLOT_BYTES
            - payload_sum;
        prop_assert_eq!(geom.slot_count(), records_meta.len() as u16);
        prop_assert_eq!(geom.free_bytes(), expected_free as u32);
        prop_assert_eq!(geom.page_cell(), active_cell);

        for (rec, slot_cell, expected_bytes) in &records_meta {
            let (range, decoded_fmt) =
                decode_inline_record(&page, *rec, active_cell, *slot_cell)
                    .expect("appended record decodes");
            prop_assert_eq!(decoded_fmt, format);
            prop_assert_eq!(&page[range.range()], *expected_bytes);
        }

        // Restamp to higher generation preserves records and rejects stale generation.
        let new_gen = active_cell.generation().get() + restamp_delta;
        prop_assert!(
            restamp_inline_page_generation(format, &page, active_cell.generation().get()).is_err()
        );
        let restamped = restamp_inline_page_generation(format, &page, new_gen)
            .expect("monotonic restamp succeeds");
        let restamped_cell = auth
            .page_cell(seg, page_id)
            .with_page_generation(PhysicalGeneration::from_raw(new_gen).unwrap());
        let (rec0, slot0, bytes0) = records_meta[0];
        prop_assert!(decode_inline_record(&restamped, rec0, active_cell, slot0).is_err());
        let (r0, _) = decode_inline_record(&restamped, rec0, restamped_cell, slot0)
            .expect("restamped record decodes");
        prop_assert_eq!(&restamped[r0.range()], bytes0);

        // Single-bit flip or truncation anywhere invalidates the page frame.
        let mut corrupted = restamped.clone();
        let idx = flip_pos % corrupted.len();
        corrupted[idx] ^= 1_u8 << flip_bit;
        prop_assert!(inspect_inline_page(format, &corrupted).is_err());

        let trunc_len = trunc_pick % restamped.len();
        prop_assert!(inspect_inline_page(format, &restamped[..trunc_len]).is_err());
    }

    #[test]
    fn extent_chunk_roundtrips_and_rejects_corruption_or_wrong_coordinate(
        epoch in nonzero_u8_16(),
        ordinal in 1_u64..=100_000,
        extent_raw in 1_u64..=100_000,
        gen_raw in 1_u64..=100_000,
        chunk_ordinal in 1_u32..=10_000,
        logical_offset in 0_u64..=10_000,
        tail_extra in 0_u64..=10_000,
        chunk in prop::collection::vec(any::<u8>(), 1..=256),
        flip_pos in any::<usize>(),
        flip_bit in 0_u8..8,
        trunc_pick in any::<usize>(),
    ) {
        let format = canonical_format();
        let auth = PhysicalGenerationAuthority::for_canonical_physical_format();
        let record = PersistedRecordIdentity::new(epoch, ordinal).unwrap();
        let extent_cell = auth
            .record_extent_cell(PhysicalExtentId::from_raw(extent_raw).unwrap())
            .with_extent_generation(PhysicalGeneration::from_raw(gen_raw).unwrap());
        let logical_bytes = logical_offset + (chunk.len() as u64) + tail_extra;
        let coord = ExtentChunkCoordinate::new(
            record,
            extent_cell,
            logical_bytes,
            logical_offset,
            chunk_ordinal,
        )
        .expect("valid extent chunk coordinate");

        let encoded = encode_extent_chunk(format, coord, &chunk).expect("extent chunk encodes");
        let (decoded, decoded_fmt) =
            decode_extent_chunk(&encoded, coord).expect("extent chunk decodes");
        prop_assert_eq!(decoded, chunk.as_slice());
        prop_assert_eq!(decoded_fmt, format);

        // Wrong generation coordinate must be denied.
        let wrong_gen_cell = auth
            .record_extent_cell(PhysicalExtentId::from_raw(extent_raw).unwrap())
            .with_extent_generation(PhysicalGeneration::from_raw(gen_raw + 1).unwrap());
        let wrong_coord = ExtentChunkCoordinate::new(
            record,
            wrong_gen_cell,
            logical_bytes,
            logical_offset,
            chunk_ordinal,
        )
        .unwrap();
        prop_assert!(decode_extent_chunk(&encoded, wrong_coord).is_err());

        // Single-bit flip anywhere must fail.
        let mut corrupted = encoded.clone();
        let idx = flip_pos % corrupted.len();
        corrupted[idx] ^= 1_u8 << flip_bit;
        prop_assert!(decode_extent_chunk(&corrupted, coord).is_err());

        // Truncation must fail.
        let trunc_len = trunc_pick % encoded.len();
        prop_assert!(decode_extent_chunk(&encoded[..trunc_len], coord).is_err());
    }

    #[test]
    fn durable_root_selector_roundtrips_and_rejects_corruption(
        store_bytes in nonzero_u8_16(),
        selector_id in 1_u64..=u64::MAX,
        is_current in any::<bool>(),
        root_gen in 1_u64..=u64::MAX,
        linked in prop::option::of((1_u64..=u64::MAX, 1_u64..=u64::MAX)),
        flip_pos in any::<usize>(),
        flip_bit in 0_u8..8,
        trunc_pick in any::<usize>(),
    ) {
        let format = canonical_format();
        let store: StableStoreIdentity = StoreNamespaceIdentityRecord::new(
            StoreNamespaceVersion::CURRENT,
            ProposedStoreIdentity::from_nonzero_bytes(store_bytes).unwrap(),
        )
        .published_identity();
        let role = if is_current {
            RootSelectorRole::Current
        } else {
            RootSelectorRole::Previous
        };
        let (linked_sel, linked_gen) = match linked {
            Some((id, g)) => (RootSelectorIdentity::new(id), Some(g)),
            None => (None, None),
        };
        let selector = DurableRootSelector::new(
            store,
            format,
            RootSelectorIdentity::new(selector_id).unwrap(),
            role,
            root_gen,
            linked_sel,
            linked_gen,
        )
        .expect("valid selector");
        let encoded = selector.encode();
        prop_assert_eq!(encoded.len(), ROOT_SELECTOR_BYTES);
        prop_assert_eq!(DurableRootSelector::decode(&encoded), Ok(selector));

        let mut corrupted = encoded;
        let idx = flip_pos % corrupted.len();
        corrupted[idx] ^= 1_u8 << flip_bit;
        prop_assert!(DurableRootSelector::decode(&corrupted).is_err());

        let trunc_len = trunc_pick % encoded.len();
        prop_assert!(DurableRootSelector::decode(&encoded[..trunc_len]).is_err());
    }
}
