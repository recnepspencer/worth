pub(crate) use {std::fs, std::num::NonZeroU64, std::path::Path, std::sync::mpsc, std::sync::Arc, std::sync::Barrier, std::thread, std::time::Duration, std::time::Instant};

pub(crate) use {worth_store::physical_runtime::production::PhysicalMutationCheckpoint, worth_store::physical_runtime::BlobIngestClaimDenial, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReadOpenFailure, worth_store::physical_runtime::BlobResumeFailure, worth_store::physical_runtime::BlobResumeLimits, worth_store::physical_runtime::BlobResumeToken, worth_store::physical_runtime::BlobTerminalDisposition, worth_store::physical_runtime::BlobTerminalFailure, worth_store::physical_runtime::ServingPhysicalRuntime};
pub(crate) use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1, worth_store_physical_format::PersistedRecordIdentity};

pub(crate) use {crate::physical_blob_journeys::blob_abort::assert_resume_abandoned, crate::physical_blob_journeys::blob_abort::deadline, crate::physical_blob_journeys::blob_abort::limits, crate::physical_blob_journeys::blob_abort::unfinished, crate::physical_blob_journeys::blob_crash::establish_recovery_frontier, crate::physical_blob_journeys::blob_crash::kill_at, crate::physical_blob_journeys::blob_crash::marker_path, crate::blob_crash::recovery::recover_closed_store, crate::physical_blob_journeys::blob_crash::resume_token_path, crate::physical_blob_journeys::blob_crash::write_marker, crate::physical_blob_journeys::blob_crash::SCOPE_KEY, worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records, crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named, worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization, worth_store_testkit_store::fixture::serving_from_open};

pub(crate) const CHUNK: usize = 64 * 1024;

pub(crate) fn assert_killed_terminal(role: &'static str) {
    let world = kill_at(role, std::time::Duration::from_secs(180));
    let token = worth_store::physical_runtime::BlobResumeToken::decode(&std::fs::read(resume_token_path(&world.root)).unwrap())
        .expect("child wrote one syntactically valid token before abort");
    assert_eq!(&token.encode()[24..40], world.session.as_slice());
    recover_closed_store(&world.root);

    let serving = serving_from_open(&world.root);
    let scope = admitted_blob_scope(SCOPE_KEY);
    let blobs = serving.blobs().unwrap();
    assert!(matches!(
        blobs.resolve_publication(
            world.object,
            1,
            &scope,
            worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap())
        ),
        Err(worth_store::physical_runtime::BlobReadOpenFailure::PublicationNotFound)
    ));
    let selected = selected_blob_records(&serving);
    let mut declaration = None;
    let mut chunk = None;
    let mut terminal = None;
    for (record, bytes) in selected {
        if !bytes.starts_with(b"WRC11BLB") {
            continue;
        }
        match decode_blob_record(&bytes).expect("selected blob control/frame must decode") {
            worth_store_physical_format::BlobRecordV1::SessionDeclared(value) if value.session() == world.session => {
                assert!(declaration.replace(record).is_none());
                assert_eq!(value.object(), world.object);
            }
            worth_store_physical_format::BlobRecordV1::Chunk(value) if value.occurrence().session() == world.session => {
                assert!(chunk.replace(record).is_none());
                assert_eq!(value.occurrence().ordinal(), 0);
                assert_eq!(value.bytes(), &[0x5a; CHUNK]);
            }
            worth_store_physical_format::BlobRecordV1::SessionAbandoned(value) if value.session() == world.session => {
                assert!(terminal.replace((record, value)).is_none());
            }
            worth_store_physical_format::BlobRecordV1::GenerationPublished(value) if value.session() == world.session => {
                panic!("aborted session cannot publish a generation")
            }
            _ => {}
        }
    }
    let declaration = declaration.expect("selected declaration must survive C8");
    chunk.expect("claimed chunk remains selected, not reclaimed by abort");
    let (terminal_record, terminal) = terminal.expect("C8 selects exactly one abort terminal");
    assert_eq!(terminal.declaration_record(), persisted(declaration));
    let expected_digest: [u8; 32] = token.encode()[64..96].try_into().unwrap();
    assert_eq!(terminal.declaration_digest(), expected_digest);
    assert_resume_abandoned(&serving, token, &scope);
    let repeated = blobs
        .abort_ingest(token, &scope, placement(), deadline(), limits())
        .unwrap();
    assert_eq!(repeated.record(), persisted(terminal_record));
    assert_eq!(
        repeated.disposition(),
        worth_store::physical_runtime::BlobTerminalDisposition::AlreadyAbandoned
    );
    serving.close();

    let report = observe_closed_store_named(&world.root, "c11-blob-abort-crash", role);
    assert_eq!(report["completeness"], "complete", "{report}");
    let artifacts = report["artifacts"].as_array().unwrap();
    let count = |family: &str| {
        artifacts
            .iter()
            .filter(|row| row["family"] == family)
            .count()
    };
    assert_eq!(count("blob_resume_session"), 2, "{report}");
    assert_eq!(count("blob_chunk_frame"), 1, "{report}");
    assert_eq!(count("blob_generation_publication"), 0, "{report}");
    for row in artifacts.iter().filter(|row| {
        row["family"]
            .as_str()
            .is_some_and(|family| family.starts_with("blob_"))
    }) {
        assert_eq!(row["outcome"]["posture"], "intact", "{row}");
    }
}

pub(crate) fn persisted(record: worth_store::physical_runtime::PhysicalRecordId) -> PersistedRecordIdentity {
    worth_store_physical_format::PersistedRecordIdentity::new(record.allocation_epoch(), record.ordinal()).unwrap()
}

pub(crate) fn child(root: &Path, role: &str) {
    let checkpoint = match role {
        "crash-abort-wal" => worth_store::physical_runtime::production::PhysicalMutationCheckpoint::AfterWalDurability,
        "crash-abort-root" => worth_store::physical_runtime::production::PhysicalMutationCheckpoint::AfterRootReplacement,
        _ => panic!("unknown abort crash role: {role}"),
    };
    let serving = serving_from_initialization(root);
    establish_recovery_frontier(&serving);
    let scope = admitted_blob_scope(SCOPE_KEY);
    let ingest = unfinished(&serving, &scope);
    let token = ingest.resume_token();
    let session = ingest.session_id().bytes();
    let object = selected_object(&serving, session);
    let token_path = resume_token_path(root);
    let pending = token_path.with_extension("pending");
    std::fs::write(&pending, token.encode()).unwrap();
    std::fs::rename(pending, token_path).unwrap();
    drop(ingest);

    let gate = serving.pause_physical_mutation_at(checkpoint);
    let marker = marker_path(root);
    std::thread::scope(|workers| {
        workers.spawn(|| {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(150);
            while std::time::Instant::now() < deadline {
                if gate.await_arrival() {
                    write_marker(&marker, object, session);
                    return;
                }
            }
            panic!("explicit abort never reached {checkpoint:?}");
        });
        let _ =
            serving
                .blobs()
                .unwrap()
                .abort_ingest(token, &scope, placement(), deadline(), limits());
        panic!("abort escaped {checkpoint:?} before process kill");
    });
}

pub(crate) fn selected_object(serving: &ServingPhysicalRuntime, session: [u8; 16]) -> [u8; 16] {
    selected_blob_records(serving)
        .into_iter()
        .find_map(|(_, bytes)| match decode_blob_record(&bytes) {
            Ok(worth_store_physical_format::BlobRecordV1::SessionDeclared(value)) if value.session() == session => {
                Some(value.object())
            }
            _ => None,
        })
        .expect("child selected declaration binds its object")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RaceResult {
    AbortWon,
    AbortDenied,
    ResumeHeld,
    ResumeDenied,
}