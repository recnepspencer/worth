pub(crate) use {std::fs, std::num::NonZeroU64, std::time::Duration};

pub(crate) use {
    worth_store::physical_runtime::BlobReadLimits,
    worth_store::physical_runtime::BlobReadOpenFailure,
    worth_store::physical_runtime::BlobResumeLimits,
    worth_store::physical_runtime::BlobResumeToken,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalRecordId,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
pub(crate) use {
    worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1,
};

pub(crate) use {
    crate::blob_crash::recovery::recover_closed_store,
    crate::physical_blob_journeys::blob_crash::kill_at,
    crate::physical_blob_journeys::blob_crash::resume_token_path,
    crate::physical_blob_journeys::blob_crash::SCOPE_KEY,
    crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_open,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records,
};

pub(crate) const CHUNK: usize = 64 * 1024;

pub(crate) fn resume_killed_prefix(role: &'static str) {
    let world = kill_at(role, std::time::Duration::from_secs(90));
    let token = worth_store::physical_runtime::BlobResumeToken::decode(
        &std::fs::read(resume_token_path(&world.root)).unwrap(),
    )
    .expect("caller sidecar carries one syntactically valid token");
    let wire = token.encode();
    assert_eq!(&wire[24..40], world.session.as_slice());
    recover_closed_store(&world.root);

    let serving = serving_from_open(&world.root);
    let scope = admitted_blob_scope(SCOPE_KEY);
    let limits =
        worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap());
    let resume_limits = worth_store::physical_runtime::BlobResumeLimits::new(
        std::num::NonZeroU64::new(128).unwrap(),
        std::num::NonZeroU64::new(1024 * 1024).unwrap(),
    );
    let before = selected_chunks(&serving, world.session);
    assert_eq!(before.len(), 1, "C8 must retain exactly one prefix chunk");
    assert_eq!(before[0].0, 0);
    assert_eq!(before[0].2, expected_chunk(role, 0));
    let blobs = serving.blobs().unwrap();
    assert!(matches!(
        blobs.resolve_publication(world.object, 1, &scope, limits),
        Err(worth_store::physical_runtime::BlobReadOpenFailure::PublicationNotFound)
    ));

    let mut resumed = blobs
        .resume_ingest(
            token,
            &scope,
            placement(),
            CHUNK as u64,
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
                .unwrap(),
            resume_limits,
        )
        .unwrap_or_else(|error| panic!("selected prefix must readmit after C8: {error:?}"));
    assert_eq!(resumed.session_id().bytes(), world.session);
    assert_eq!(resumed.frontier().bytes(), CHUNK as u64);
    assert_eq!(resumed.frontier().next_chunk_ordinal(), 1);
    let observed = resumed.resume_observation().unwrap();
    assert_eq!(observed.rehashed_chunks(), 1);
    assert_eq!(observed.rehashed_bytes(), CHUNK as u64);
    assert_eq!(selected_chunks(&serving, world.session), before);
    resumed.push(&expected_chunk(role, 1)).unwrap();
    let published = resumed.finish().unwrap();
    assert_eq!(published.object().bytes(), world.object);
    assert_eq!(published.session().bytes(), world.session);
    let after = selected_chunks(&serving, world.session);
    assert_eq!(after.len(), 2, "resume must not duplicate ordinal zero");
    assert_eq!(
        after[0], before[0],
        "resume must reuse exact selected RecordId"
    );
    assert_eq!(after[1].0, 1);
    assert_ne!(after[1].1, before[0].1);
    assert_eq!(after[1].2, expected_chunk(role, 1));
    serving.close();

    let reopened = serving_from_open(&world.root);
    let blobs = reopened.blobs().unwrap();
    let resolved = blobs
        .resolve_publication(world.object, 1, &scope, limits)
        .unwrap();
    assert_eq!(resolved, published);
    let mut reader = blobs
        .read(resolved, &scope, 0, (2 * CHUNK) as u64, limits)
        .unwrap();
    let mut actual = Vec::new();
    let mut frame = [0_u8; 4096];
    loop {
        let copied = reader.read_next(&mut frame).unwrap();
        if copied == 0 {
            break;
        }
        actual.extend_from_slice(&frame[..copied]);
    }
    assert_eq!(actual.len(), 2 * CHUNK);
    for (index, byte) in actual.iter().enumerate() {
        let ordinal = index / CHUNK;
        let within_chunk = index % CHUNK;
        let expected = expected_byte(role, ordinal, within_chunk);
        assert_eq!(*byte, expected, "byte {index} must match original source");
    }
    drop(reader);
    reopened.close();

    let report = observe_closed_store_named(&world.root, "c11-blob-resume", role);
    assert_eq!(report["completeness"], "complete", "{report}");
    let artifacts = report["artifacts"].as_array().unwrap();
    for (family, count) in [
        (
            "blob_resume_session",
            if role == "crash-frontier-partial" {
                2
            } else {
                1
            },
        ),
        ("blob_chunk_frame", 2),
        ("blob_generation_publication", 1),
    ] {
        let matching = artifacts
            .iter()
            .filter(|artifact| artifact["family"] == family)
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), count, "{family}: {report}");
        for artifact in matching {
            assert_eq!(artifact["outcome"]["posture"], "intact", "{artifact}");
        }
    }
}

pub(crate) fn selected_chunks(
    serving: &ServingPhysicalRuntime,
    session: [u8; 16],
) -> Vec<(u64, PhysicalRecordId, Vec<u8>)> {
    let mut chunks = selected_blob_records(serving)
        .into_iter()
        .filter_map(|(record, bytes)| match decode_blob_record(&bytes) {
            Ok(worth_store_physical_format::BlobRecordV1::Chunk(chunk))
                if chunk.occurrence().session() == session =>
            {
                Some((chunk.occurrence().ordinal(), record, chunk.bytes().to_vec()))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    chunks.sort_by_key(|(ordinal, _, _)| *ordinal);
    chunks
}

pub(crate) fn expected_chunk(role: &str, ordinal: usize) -> Vec<u8> {
    (0..CHUNK)
        .map(|index| expected_byte(role, ordinal, index))
        .collect()
}

pub(crate) fn expected_byte(role: &str, ordinal: usize, index: usize) -> u8 {
    if role == "crash-frontier-partial" {
        // The killed writer buffered 17 bytes of 0xa5 after the first 0x3c
        // chunk. Refeeding the whole second chunk must not duplicate them.
        if ordinal == 0 {
            0x3c
        } else {
            0xa5
        }
    } else {
        (ordinal as u8).wrapping_mul(17).wrapping_add(index as u8)
    }
}
