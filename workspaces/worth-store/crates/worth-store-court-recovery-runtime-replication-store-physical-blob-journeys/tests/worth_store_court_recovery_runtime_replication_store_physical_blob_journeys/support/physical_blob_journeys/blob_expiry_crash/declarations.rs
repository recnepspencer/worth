pub(crate) use {std::fs, std::num::NonZeroU64, std::path::Path, std::thread, std::time::Duration, std::time::Instant};

pub(crate) use {worth_store::physical_runtime::production::PhysicalMutationCheckpoint, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReadOpenFailure, worth_store::physical_runtime::BlobResumeToken, worth_store::physical_runtime::BlobTerminalDisposition, worth_store::physical_runtime::ServingPhysicalRuntime};
pub(crate) use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobAbandonmentReasonV1, worth_store_physical_format::BlobRecordV1, worth_store_physical_format::PersistedRecordIdentity};

pub(crate) use {crate::physical_blob_journeys::blob_abort::assert_resume_abandoned, crate::physical_blob_journeys::blob_crash::establish_recovery_frontier, crate::physical_blob_journeys::blob_crash::kill_at, crate::physical_blob_journeys::blob_crash::marker_path, crate::blob_crash::recovery::recover_closed_store, crate::physical_blob_journeys::blob_crash::resume_token_path, crate::physical_blob_journeys::blob_crash::write_marker, crate::physical_blob_journeys::blob_crash::SCOPE_KEY, crate::physical_blob_journeys::blob_expiry::completed_checkpoint, crate::physical_blob_journeys::blob_expiry::deadline, crate::physical_blob_journeys::blob_expiry::limits, crate::physical_blob_journeys::blob_expiry::unfinished, worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records, crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named, worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::placement, worth_store_testkit_store::fixture::serving_from_initialization, worth_store_testkit_store::fixture::serving_from_open};

pub(crate) const CHUNK: usize = 64 * 1024;

pub(crate) fn assert_killed_expiry(role: &'static str) {
    let world = kill_at(role, std::time::Duration::from_secs(180));
    let token = worth_store::physical_runtime::BlobResumeToken::decode(&std::fs::read(resume_token_path(&world.root)).unwrap())
        .expect("child wrote syntactically valid token before expiry");
    let wire = token.encode();
    assert_eq!(&wire[24..40], world.session.as_slice());
    let declared_max = u64::from_le_bytes(wire[108..116].try_into().unwrap());
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
        match decode_blob_record(&bytes).expect("selected blob frame must decode") {
            worth_store_physical_format::BlobRecordV1::SessionDeclared(value) if value.session() == world.session => {
                assert!(declaration.replace(record).is_none());
                assert_eq!(value.object(), world.object);
                assert_eq!(value.max_checkpoint_sequence(), declared_max);
            }
            worth_store_physical_format::BlobRecordV1::Chunk(value) if value.occurrence().session() == world.session => {
                assert!(chunk.replace(record).is_none());
                assert_eq!(value.bytes(), &[0x5a; CHUNK]);
            }
            worth_store_physical_format::BlobRecordV1::SessionAbandoned(value) if value.session() == world.session => {
                assert!(terminal.replace((record, value)).is_none());
            }
            worth_store_physical_format::BlobRecordV1::GenerationPublished(value) if value.session() == world.session => {
                panic!("expired session cannot publish a generation")
            }
            _ => {}
        }
    }
    let declaration = declaration.expect("selected declaration survives C8");
    chunk.expect("claimed chunk remains selected after terminal");
    let (terminal_record, terminal) = terminal.expect("one selected expiry terminal");
    assert_eq!(terminal.declaration_record(), persisted(declaration));
    let declaration_digest: [u8; 32] = wire[64..96].try_into().unwrap();
    assert_eq!(terminal.declaration_digest(), declaration_digest);
    let worth_store_physical_format::BlobAbandonmentReasonV1::CheckpointExpired {
        checkpoint_sequence,
    } = terminal.reason()
    else {
        panic!("crash seam must retain checkpoint-expiry reason");
    };
    assert!(checkpoint_sequence.get() > declared_max);
    assert_resume_abandoned(&serving, token, &scope);
    let repeated = blobs
        .expire_ingest(token, &scope, placement(), deadline(), limits())
        .unwrap();
    assert_eq!(repeated.record(), persisted(terminal_record));
    assert_eq!(
        repeated.disposition(),
        worth_store::physical_runtime::BlobTerminalDisposition::AlreadyAbandoned
    );
    serving.close();

    let report = observe_closed_store_named(&world.root, "c11-blob-expiry-crash", role);
    assert_eq!(report["completeness"], "complete", "{report}");
    let artifacts = report["artifacts"].as_array().unwrap();
    assert_eq!(
        artifacts
            .iter()
            .filter(|row| row["family"] == "blob_resume_session")
            .count(),
        2,
        "{report}"
    );
    assert!(!artifacts
        .iter()
        .any(|row| row["family"] == "blob_generation_publication"));
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
        "crash-expiry-wal" => worth_store::physical_runtime::production::PhysicalMutationCheckpoint::AfterWalDurability,
        "crash-expiry-root" => worth_store::physical_runtime::production::PhysicalMutationCheckpoint::AfterRootReplacement,
        _ => panic!("unknown expiry crash role: {role}"),
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
    let maximum = u64::from_le_bytes(token.encode()[108..116].try_into().unwrap());
    assert_eq!(
        completed_checkpoint(&serving, 0xc2)
            .footer()
            .identity()
            .sequence()
            .get(),
        maximum
    );
    assert!(
        completed_checkpoint(&serving, 0xc3)
            .footer()
            .identity()
            .sequence()
            .get()
            > maximum
    );

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
            panic!("expiry never reached {checkpoint:?}");
        });
        let _ = serving.blobs().unwrap().expire_ingest(
            token,
            &scope,
            placement(),
            deadline(),
            limits(),
        );
        panic!("expiry escaped {checkpoint:?} before process kill");
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
        .expect("child selected declaration binds object")
}