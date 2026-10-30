use {
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadLimits,
};

use {
    worth_store_testkit_store::durable_publication,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::read_record,
    worth_store_testkit_store::physical_record_journeys::scenario_configuration::dense_configuration,
    worth_store_testkit_store::physical_record_journeys::success,
    worth_store_testkit_store::stream_fixture::PatternSource,
};

#[test]
fn batch_packing_matches_an_independent_page_oracle() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, placement, access) = dense_configuration(4);
    let serving = success(initialize_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )
    }));
    let payloads = [b"abc".as_slice(), b"12345".as_slice(), b"".as_slice()];
    let completed = worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material(
            "page-packing-independent-oracle",
            0,
        ),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter(payloads).unwrap(),
    );
    let published = &completed.settled_members()[0];
    assert_eq!(
        published.observation().records(),
        3,
        "C5_PREDICATE:batch-atomicity: every admitted batch member must reach one publication"
    );
    assert_eq!(
        published.observation().transfer_count(),
        published.data_effect_count() as u64
    );
    assert_eq!(
        published.observation().explicit_copy_count(),
        3,
        "C5_PREDICATE:batch-atomicity: every admitted batch member must perform its required copy"
    );
    assert_eq!(published.observation().copied_bytes(), 8);

    let page = std::fs::read(
        root.join("families/records/segments/segment-0000000000000001-0000000000000001.pages"),
    )
    .unwrap();
    assert_eq!(page.len(), 16_384);
    assert_eq!(&page[..8], b"WRC5FRM\0");
    assert_eq!(page[8], 3);
    assert_eq!(u64::from_le_bytes(page[28..36].try_into().unwrap()), 1);
    assert_eq!(page[9], 2);
    let member_lsn_range = published
        .wal_barrier_settlement()
        .member_basis()
        .lsn_range();
    assert_eq!(
        u64::from_le_bytes(page[36..44].try_into().unwrap()),
        member_lsn_range
            .end_exclusive()
            .get()
            .checked_sub(1)
            .expect("a nonempty WAL member has a greatest redo LSN")
    );
    assert_eq!(
        u32::from_le_bytes(page[44..48].try_into().unwrap()),
        worth_store_testkit_store::durable_frame_oracle::independent_crc32c(&[
            &page[..44],
            &page[worth_store_testkit_store::durable_frame_oracle::HEADER_BYTES..],
        ])
    );

    let frame_payload = worth_store_testkit_store::durable_frame_oracle::payload(&page);
    assert_eq!(
        u64::from_le_bytes(frame_payload[..8].try_into().unwrap()),
        1
    );
    assert_eq!(
        u64::from_le_bytes(frame_payload[8..16].try_into().unwrap()),
        1
    );
    assert_eq!(
        u16::from_le_bytes(frame_payload[16..18].try_into().unwrap()),
        3
    );
    assert_eq!(&frame_payload[18..24], &[0; 6]);
    let expected_offsets = [
        frame_payload.len() - 3,
        frame_payload.len() - 8,
        frame_payload.len() - 8,
    ];
    for (index, expected_payload) in payloads.iter().enumerate() {
        let base = 24 + index * 40;
        let record = published.record_id(index).unwrap();
        assert_eq!(&frame_payload[base..base + 16], &record.allocation_epoch());
        assert_eq!(
            u64::from_le_bytes(frame_payload[base + 16..base + 24].try_into().unwrap()),
            record.ordinal()
        );
        let offset =
            u32::from_le_bytes(frame_payload[base + 24..base + 28].try_into().unwrap()) as usize;
        let length =
            u32::from_le_bytes(frame_payload[base + 28..base + 32].try_into().unwrap()) as usize;
        assert_eq!(
            (offset, length),
            (expected_offsets[index], expected_payload.len())
        );
        assert_eq!(
            u64::from_le_bytes(frame_payload[base + 32..base + 40].try_into().unwrap()),
            1
        );
        assert_eq!(&frame_payload[offset..offset + length], *expected_payload);
    }
    let directory_end = 24 + payloads.len() * 40;
    let expected_unoccupied_bytes =
        16_384 - worth_store_testkit_store::durable_frame_oracle::HEADER_BYTES - 8 - directory_end;
    assert_eq!(
        expected_offsets[2] - directory_end,
        expected_unoccupied_bytes
    );
    serving.close();
}

#[test]
fn inline_source_and_delivery_copies_are_counted_at_the_actual_copy_seams() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, placement, access) = dense_configuration(4);
    let serving = success(initialize_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )
    }));
    let published = worth_store_testkit_store::durable_publication::root::publish_single(
        &serving,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material(
            "page-packing-copy-seams",
            0,
        ),
        worth_store::physical_runtime::RecordAppendBatch::builder()
            .push_source(
                worth_store_testkit_store::stream_fixture::PatternSource::fragmented(13, 4),
            )
            .build()
            .unwrap(),
    );
    let member = &published.settled_members()[0];
    assert_eq!(member.observation().explicit_copy_count(), 5);
    assert_eq!(member.observation().copied_bytes(), 26);

    let session = serving
        .records()
        .expect("read protection admission")
        .open(
            member.record_id(0).unwrap(),
            worth_store::physical_runtime::RecordReadLimits::new(
                worth_store::physical_runtime::RecordByteLimit::new(13).unwrap(),
            ),
        )
        .unwrap();
    let (_, observation) = read_record(session, 13);
    assert_eq!(observation.bytes_completed(), 13);
    assert_eq!(observation.explicit_copy_count(), 1);
    assert_eq!(observation.copied_bytes(), 13);
    serving.close();
}
