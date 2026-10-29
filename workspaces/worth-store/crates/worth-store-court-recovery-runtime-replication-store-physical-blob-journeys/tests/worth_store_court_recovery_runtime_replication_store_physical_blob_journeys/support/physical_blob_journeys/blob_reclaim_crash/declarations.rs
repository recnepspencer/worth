pub(crate) use {std::fs, std::num::NonZeroU64, std::path::Path, std::thread, std::time::Duration, std::time::Instant};

pub(crate) use {worth_store::physical_runtime::production::PhysicalMutationCheckpoint, worth_store::physical_runtime::BlobReadLimits, worth_store::physical_runtime::BlobReadOpenFailure, worth_store::physical_runtime::BlobReclaimDisposition, worth_store::physical_runtime::BlobResumeToken};
pub(crate) use {worth_store_physical_format::decode_blob_record, worth_store_physical_format::BlobRecordV1, worth_store_physical_format::PersistedRecordIdentity};

pub(crate) use {crate::physical_blob_journeys::blob_crash::establish_recovery_frontier, crate::physical_blob_journeys::blob_crash::kill_at, crate::physical_blob_journeys::blob_crash::marker_path, crate::blob_crash::recovery::recover_closed_store, crate::physical_blob_journeys::blob_crash::resume_token_path, crate::physical_blob_journeys::blob_crash::write_marker, worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records, crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named, crate::physical_blob_journeys::blob_reclaim::abandoned_prefix, crate::physical_blob_journeys::blob_reclaim::request, worth_store_testkit_store::fixture::admitted_blob_scope, worth_store_testkit_store::fixture::serving_from_initialization, worth_store_testkit_store::fixture::serving_from_initialization_with_wal_segment_bytes, worth_store_testkit_store::fixture::serving_from_open, worth_store_testkit_store::fixture::serving_from_open_with_wal_segment_bytes};

pub(crate) const SCOPE_KEY: &str = "c11.blob.reclaim.failed.scope";
pub(crate) const AGED_RESERVATION: &str = "crash-reclaim-reservation-aged";
pub(crate) const SHORT_WAL_SEGMENT_BYTES: u64 = 256 * 1024;

pub(crate) fn assert_killed_reclaim(role: &'static str, dropped: bool) {
    let aged = role == AGED_RESERVATION;
    let reservation_only = role == "crash-reclaim-reservation" || aged;
    let world = kill_at(role, std::time::Duration::from_secs(180));
    recover_closed_store(&world.root);
    let serving = if aged {
        serving_from_open_with_wal_segment_bytes(&world.root, SHORT_WAL_SEGMENT_BYTES)
    } else {
        serving_from_open(&world.root)
    };
    let scope = admitted_blob_scope(SCOPE_KEY);
    assert!(matches!(
        serving.blobs().unwrap().resolve_publication(
            world.object,
            1,
            &scope,
            worth_store::physical_runtime::BlobReadLimits::new(std::num::NonZeroU64::new(128).unwrap()),
        ),
        Err(worth_store::physical_runtime::BlobReadOpenFailure::PublicationNotFound)
    ));

    let selected = selected_blob_records(&serving);
    let mut declaration = 0;
    let mut terminal = 0;
    let mut chunks = 0;
    let mut frontier = None;
    let mut manifest = None;
    let mut reservation = None;
    let mut descriptor = None;
    for (record, bytes) in &selected {
        if !bytes.starts_with(b"WRC11BLB") {
            continue;
        }
        match decode_blob_record(bytes).expect("selected blob record must decode") {
            worth_store_physical_format::BlobRecordV1::SessionDeclared(value) if value.session() == world.session => {
                assert_eq!(value.object(), world.object);
                declaration += 1;
            }
            worth_store_physical_format::BlobRecordV1::SessionAbandoned(value) if value.session() == world.session => {
                terminal += 1;
            }
            worth_store_physical_format::BlobRecordV1::Chunk(value) if value.occurrence().session() == world.session => {
                assert!(value.occurrence().ordinal() < 2);
                chunks += 1;
            }
            worth_store_physical_format::BlobRecordV1::SessionFrontier(value) if value.session() == world.session => {
                assert!(frontier.replace(persisted(*record)).is_none());
            }
            worth_store_physical_format::BlobRecordV1::DropSetManifestV2(value)
                if value.source_basis().session() == world.session =>
            {
                assert!(manifest.replace((*record, value)).is_none());
            }
            worth_store_physical_format::BlobRecordV1::OriginalDropReserved(value) if value.source_basis_digest() != [0; 32] => {
                assert!(reservation.replace((*record, value)).is_none());
            }
            worth_store_physical_format::BlobRecordV1::ReclaimDescriptor(value) => {
                assert!(descriptor.replace((*record, value)).is_none());
            }
            worth_store_physical_format::BlobRecordV1::GenerationPublished(value) if value.session() == world.session => {
                panic!("failed ingest cannot publish a generation")
            }
            _ => {}
        }
    }
    assert_eq!((declaration, terminal, chunks), (1, 1, 2));
    let (manifest_record, manifest) =
        manifest.expect("manifest root must remain selected after kill");
    let reserved_record = reservation.map(|(record, _)| persisted(record));
    let original_lease_expiry =
        reservation.map(|(_, value)| value.request().lease_expiry_generation());
    assert_eq!(manifest.count(), 1);
    let dropped_frontier = manifest.dropped()[0];
    assert_eq!(frontier, (!dropped).then_some(dropped_frontier));
    if dropped {
        let (_, reserved) = reservation.expect("selected reservation precedes durable drop WAL");
        assert_eq!(reserved.reclaim_attempt(), manifest.reclaim_attempt());
        let (_, descriptor) = descriptor.expect("durable descriptor WAL must replay drop root");
        assert_eq!(descriptor.reclaim_attempt(), manifest.reclaim_attempt());
        assert_eq!(
            descriptor.source_basis_digest(),
            manifest.source_basis_digest()
        );
        assert_eq!(descriptor.manifest_count(), 1);
    } else if reservation_only {
        let (_, reserved) = reservation.expect("reservation must remain selected");
        assert_eq!(reserved.reclaim_attempt(), manifest.reclaim_attempt());
        assert!(
            descriptor.is_none(),
            "preparation did not reach descriptor WAL"
        );
    } else {
        assert!(
            reservation.is_none(),
            "manifest-only crash cannot invent a reservation"
        );
        assert!(
            descriptor.is_none(),
            "manifest selection is not drop authority"
        );
    }
    serving.close();

    let report = observe_closed_store_named(&world.root, "c11-blob-reclaim-crash", role);
    assert_eq!(report["completeness"], "complete", "{report}");
    let artifacts = report["artifacts"].as_array().unwrap();
    let count = |family: &str| {
        artifacts
            .iter()
            .filter(|row| row["family"] == family)
            .count()
    };
    assert_eq!(count("blob_drop_set_manifest"), 1, "{report}");
    assert_eq!(
        count("blob_original_drop_reserved"),
        usize::from(dropped || reservation_only),
        "{report}"
    );
    assert_eq!(
        count("blob_reclaim_descriptor"),
        usize::from(dropped),
        "{report}"
    );
    assert_eq!(count("blob_chunk_frame"), 2, "{report}");
    for row in artifacts.iter().filter(|row| {
        row["family"]
            .as_str()
            .is_some_and(|family| family.starts_with("blob_"))
    }) {
        assert_eq!(row["outcome"]["posture"], "intact", "{row}");
    }
    if !dropped {
        // Neither a selected NeverReserved manifest nor a recovered no-binding
        // reservation may strand the failed ingest's payload.
        let token =
            worth_store::physical_runtime::BlobResumeToken::decode(&std::fs::read(resume_token_path(&world.root)).unwrap()).unwrap();
        let serving = if aged {
            serving_from_open_with_wal_segment_bytes(&world.root, SHORT_WAL_SEGMENT_BYTES)
        } else {
            serving_from_open(&world.root)
        };
        let serving = if aged {
            crate::physical_blob_journeys::blob_reclaim_crash::aging::age_and_prune_original_reservation(
                &serving,
                original_lease_expiry.expect("selected reservation has a lease"),
                persisted(manifest_record),
                reserved_record.expect("selected reservation has an identity"),
            );
            serving.close();
            recover_closed_store(&world.root);
            serving_from_open_with_wal_segment_bytes(&world.root, SHORT_WAL_SEGMENT_BYTES)
        } else {
            serving
        };
        let mut dropped_once = Vec::new();
        for remaining in [2, 1, 0] {
            let receipt = serving
                .blobs()
                .unwrap()
                .reclaim(request(token, &scope))
                .unwrap()
                .wait()
                .unwrap();
            assert_eq!(receipt.disposition(), worth_store::physical_runtime::BlobReclaimDisposition::Dropped);
            assert_eq!(receipt.remaining_payload_records(), remaining);
            if aged {
                assert_eq!(
                    receipt.dropped_records().len(),
                    1,
                    "one-record bound cannot duplicate the original drop"
                );
                assert!(!dropped_once.contains(&receipt.dropped_records()[0]));
                dropped_once.push(receipt.dropped_records()[0]);
            }
        }
        if aged {
            assert_eq!(dropped_once.len(), 3);
            assert!(dropped_once.contains(&dropped_frontier));
        }
        let cleanup = serving
            .blobs()
            .unwrap()
            .reclaim(request(token, &scope))
            .unwrap()
            .wait()
            .unwrap();
        assert!(cleanup
            .dropped_records()
            .contains(&persisted(manifest_record)));
        if aged {
            assert_eq!(cleanup.dropped_records().len(), 2);
            assert!(cleanup
                .dropped_records()
                .contains(&reserved_record.expect("cleanup removes exact original reservation")));
        }
        assert_eq!(cleanup.remaining_payload_records(), 0);
        serving.close();
    }
}

pub(crate) fn persisted(record: worth_store::physical_runtime::PhysicalRecordId) -> PersistedRecordIdentity {
    worth_store_physical_format::PersistedRecordIdentity::new(record.allocation_epoch(), record.ordinal()).unwrap()
}

pub(crate) fn child(root: &Path, role: &str) {
    let serving = if role == AGED_RESERVATION {
        serving_from_initialization_with_wal_segment_bytes(root, SHORT_WAL_SEGMENT_BYTES)
    } else {
        serving_from_initialization(root)
    };
    establish_recovery_frontier(&serving);
    let scope = admitted_blob_scope(SCOPE_KEY);
    let token = abandoned_prefix(&serving, &scope);
    std::fs::write(resume_token_path(root), token.encode()).unwrap();
    let session: [u8; 16] = token.encode()[24..40].try_into().unwrap();
    let object = selected_blob_records(&serving)
        .into_iter()
        .find_map(|(_, bytes)| match decode_blob_record(&bytes) {
            Ok(worth_store_physical_format::BlobRecordV1::SessionDeclared(value)) if value.session() == session => {
                Some(value.object())
            }
            _ => None,
        })
        .expect("selected abandoned declaration binds object");
    let marker = marker_path(root);
    let first_checkpoint = match role {
        "crash-reclaim-manifest" | "crash-reclaim-reservation" | AGED_RESERVATION => {
            worth_store::physical_runtime::production::PhysicalMutationCheckpoint::AfterRootReplacement
        }
        "crash-reclaim-wal" => worth_store::physical_runtime::production::PhysicalMutationCheckpoint::AfterWalDurability,
        _ => panic!("unknown reclaim crash role: {role}"),
    };
    let first_gate = serving.pause_physical_mutation_at(first_checkpoint);
    std::thread::scope(|workers| {
        workers.spawn(|| {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(150);
            while std::time::Instant::now() < deadline {
                if first_gate.await_arrival() {
                    if role == "crash-reclaim-manifest" {
                        write_marker(&marker, object, session);
                        return;
                    }
                    if role == "crash-reclaim-reservation" || role == AGED_RESERVATION {
                        let reservation_gate = serving.pause_physical_mutation_at(
                            worth_store::physical_runtime::production::PhysicalMutationCheckpoint::AfterRootReplacement,
                        );
                        first_gate.release();
                        while std::time::Instant::now() < deadline {
                            if reservation_gate.await_arrival() {
                                write_marker(&marker, object, session);
                                return;
                            }
                        }
                        panic!("reservation never reached selected-root seam");
                    }
                    let reservation_gate = serving
                        .pause_physical_mutation_at(worth_store::physical_runtime::production::PhysicalMutationCheckpoint::AfterWalDurability);
                    first_gate.release();
                    while std::time::Instant::now() < deadline {
                        if reservation_gate.await_arrival() {
                            let descriptor_gate = serving.pause_physical_mutation_at(
                                worth_store::physical_runtime::production::PhysicalMutationCheckpoint::AfterWalDurability,
                            );
                            reservation_gate.release();
                            while std::time::Instant::now() < deadline {
                                if descriptor_gate.await_arrival() {
                                    write_marker(&marker, object, session);
                                    return;
                                }
                            }
                            panic!("descriptor never reached durable WAL seam");
                        }
                    }
                    panic!("reservation never reached durable WAL seam");
                }
            }
            panic!("manifest never reached {first_checkpoint:?}");
        });
        let _ = serving
            .blobs()
            .unwrap()
            .reclaim(request(token, &scope))
            .unwrap()
            .wait();
        panic!("reclaim escaped {first_checkpoint:?} before child kill");
    });
}