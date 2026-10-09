use std::{
    fs,
    num::NonZeroU64,
    path::Path,
    thread,
    time::{Duration, Instant},
};

use worth_store::physical_runtime::{
    production::PhysicalMutationCheckpoint, BlobReadLimits, BlobReadOpenFailure,
    BlobReclaimDisposition, BlobResumeToken,
};
use worth_store_physical_format::{decode_blob_record, BlobRecordV1, PersistedRecordIdentity};

use super::{
    blob_crash::{
        establish_recovery_frontier, kill_at, marker_path, recover_closed_store, resume_token_path,
        write_marker,
    },
    blob_frontier::selected_blob_records,
    blob_ingest_process::observe_closed_store_named,
    blob_reclaim::{abandoned_prefix, request},
    fixture::{
        admitted_blob_scope, serving_from_initialization,
        serving_from_initialization_with_wal_segment_bytes, serving_from_open,
        serving_from_open_with_wal_segment_bytes,
    },
};

const SCOPE_KEY: &str = "c11.blob.reclaim.failed.scope";
const AGED_RESERVATION: &str = "crash-reclaim-reservation-aged";
const SHORT_WAL_SEGMENT_BYTES: u64 = 256 * 1024;

#[path = "blob_reclaim_crash/aging.rs"]
mod aging;

#[test]
fn selected_manifest_kill_retains_failed_ingest_payload_without_a_drop() {
    assert_killed_reclaim("crash-reclaim-manifest", false);
}

#[test]
fn selected_reservation_kill_without_descriptor_does_not_strand_payload() {
    assert_killed_reclaim("crash-reclaim-reservation", false);
}

#[test]
fn selected_reservation_still_cleans_after_lease_expiry_and_wal_pruning() {
    assert_killed_reclaim(AGED_RESERVATION, false);
}

#[test]
fn durable_descriptor_wal_kill_replays_exact_failed_ingest_drop() {
    assert_killed_reclaim("crash-reclaim-wal", true);
}

fn assert_killed_reclaim(role: &'static str, dropped: bool) {
    let aged = role == AGED_RESERVATION;
    let reservation_only = role == "crash-reclaim-reservation" || aged;
    let world = kill_at(role, Duration::from_secs(180));
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
            BlobReadLimits::new(NonZeroU64::new(128).unwrap()),
        ),
        Err(BlobReadOpenFailure::PublicationNotFound)
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
            BlobRecordV1::SessionDeclared(value) if value.session() == world.session => {
                assert_eq!(value.object(), world.object);
                declaration += 1;
            }
            BlobRecordV1::SessionAbandoned(value) if value.session() == world.session => {
                terminal += 1;
            }
            BlobRecordV1::Chunk(value) if value.occurrence().session() == world.session => {
                assert!(value.occurrence().ordinal() < 2);
                chunks += 1;
            }
            BlobRecordV1::SessionFrontier(value) if value.session() == world.session => {
                assert!(frontier.replace(persisted(*record)).is_none());
            }
            BlobRecordV1::DropSetManifestV2(value)
                if value.source_basis().session() == world.session =>
            {
                assert!(manifest.replace((*record, value)).is_none());
            }
            BlobRecordV1::OriginalDropReserved(value) if value.source_basis_digest() != [0; 32] => {
                assert!(reservation.replace((*record, value)).is_none());
            }
            BlobRecordV1::ReclaimDescriptor(value) => {
                assert!(descriptor.replace((*record, value)).is_none());
            }
            BlobRecordV1::GenerationPublished(value) if value.session() == world.session => {
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
            BlobResumeToken::decode(&fs::read(resume_token_path(&world.root)).unwrap()).unwrap();
        let serving = if aged {
            serving_from_open_with_wal_segment_bytes(&world.root, SHORT_WAL_SEGMENT_BYTES)
        } else {
            serving_from_open(&world.root)
        };
        let serving = if aged {
            aging::age_and_prune_original_reservation(
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
            assert_eq!(receipt.disposition(), BlobReclaimDisposition::Dropped);
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

fn persisted(record: worth_store::physical_runtime::PhysicalRecordId) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new(record.allocation_epoch(), record.ordinal()).unwrap()
}

pub(super) fn child(root: &Path, role: &str) {
    let serving = if role == AGED_RESERVATION {
        serving_from_initialization_with_wal_segment_bytes(root, SHORT_WAL_SEGMENT_BYTES)
    } else {
        serving_from_initialization(root)
    };
    establish_recovery_frontier(&serving);
    let scope = admitted_blob_scope(SCOPE_KEY);
    let token = abandoned_prefix(&serving, &scope);
    fs::write(resume_token_path(root), token.encode()).unwrap();
    let session: [u8; 16] = token.encode()[24..40].try_into().unwrap();
    let object = selected_blob_records(&serving)
        .into_iter()
        .find_map(|(_, bytes)| match decode_blob_record(&bytes) {
            Ok(BlobRecordV1::SessionDeclared(value)) if value.session() == session => {
                Some(value.object())
            }
            _ => None,
        })
        .expect("selected abandoned declaration binds object");
    let marker = marker_path(root);
    let first_checkpoint = match role {
        "crash-reclaim-manifest" | "crash-reclaim-reservation" | AGED_RESERVATION => {
            PhysicalMutationCheckpoint::AfterRootReplacement
        }
        "crash-reclaim-wal" => PhysicalMutationCheckpoint::AfterWalDurability,
        _ => panic!("unknown reclaim crash role: {role}"),
    };
    let first_gate = serving.pause_physical_mutation_at(first_checkpoint);
    thread::scope(|workers| {
        workers.spawn(|| {
            let deadline = Instant::now() + Duration::from_secs(150);
            while Instant::now() < deadline {
                if first_gate.await_arrival() {
                    if role == "crash-reclaim-manifest" {
                        write_marker(&marker, object, session);
                        return;
                    }
                    if role == "crash-reclaim-reservation" || role == AGED_RESERVATION {
                        let reservation_gate = serving.pause_physical_mutation_at(
                            PhysicalMutationCheckpoint::AfterRootReplacement,
                        );
                        first_gate.release();
                        while Instant::now() < deadline {
                            if reservation_gate.await_arrival() {
                                write_marker(&marker, object, session);
                                return;
                            }
                        }
                        panic!("reservation never reached selected-root seam");
                    }
                    let reservation_gate = serving
                        .pause_physical_mutation_at(PhysicalMutationCheckpoint::AfterWalDurability);
                    first_gate.release();
                    while Instant::now() < deadline {
                        if reservation_gate.await_arrival() {
                            let descriptor_gate = serving.pause_physical_mutation_at(
                                PhysicalMutationCheckpoint::AfterWalDurability,
                            );
                            reservation_gate.release();
                            while Instant::now() < deadline {
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
