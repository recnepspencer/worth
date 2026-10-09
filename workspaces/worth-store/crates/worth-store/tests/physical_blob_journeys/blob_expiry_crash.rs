use std::{
    fs,
    num::NonZeroU64,
    path::Path,
    thread,
    time::{Duration, Instant},
};

use worth_store::physical_runtime::{
    production::PhysicalMutationCheckpoint, BlobReadLimits, BlobReadOpenFailure, BlobResumeToken,
    BlobTerminalDisposition, ServingPhysicalRuntime,
};
use worth_store_physical_format::{
    decode_blob_record, BlobAbandonmentReasonV1, BlobRecordV1, PersistedRecordIdentity,
};

use super::{
    blob_abort::assert_resume_abandoned,
    blob_crash::{
        establish_recovery_frontier, kill_at, marker_path, recover_closed_store, resume_token_path,
        write_marker, SCOPE_KEY,
    },
    blob_expiry::{completed_checkpoint, deadline, limits, unfinished},
    blob_frontier::selected_blob_records,
    blob_ingest_process::observe_closed_store_named,
    fixture::{admitted_blob_scope, placement, serving_from_initialization, serving_from_open},
};

const CHUNK: usize = 64 * 1024;

#[test]
fn expiry_wal_durable_before_root_replacement_replays_one_terminal() {
    assert_killed_expiry("crash-expiry-wal");
}

#[test]
fn expiry_root_replaced_before_completion_reopens_one_terminal() {
    assert_killed_expiry("crash-expiry-root");
}

fn assert_killed_expiry(role: &'static str) {
    let world = kill_at(role, Duration::from_secs(180));
    let token = BlobResumeToken::decode(&fs::read(resume_token_path(&world.root)).unwrap())
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
            BlobReadLimits::new(NonZeroU64::new(128).unwrap())
        ),
        Err(BlobReadOpenFailure::PublicationNotFound)
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
            BlobRecordV1::SessionDeclared(value) if value.session() == world.session => {
                assert!(declaration.replace(record).is_none());
                assert_eq!(value.object(), world.object);
                assert_eq!(value.max_checkpoint_sequence(), declared_max);
            }
            BlobRecordV1::Chunk(value) if value.occurrence().session() == world.session => {
                assert!(chunk.replace(record).is_none());
                assert_eq!(value.bytes(), &[0x5a; CHUNK]);
            }
            BlobRecordV1::SessionAbandoned(value) if value.session() == world.session => {
                assert!(terminal.replace((record, value)).is_none());
            }
            BlobRecordV1::GenerationPublished(value) if value.session() == world.session => {
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
    let BlobAbandonmentReasonV1::CheckpointExpired {
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
        BlobTerminalDisposition::AlreadyAbandoned
    );
    drop(blobs);
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

fn persisted(record: worth_store::physical_runtime::PhysicalRecordId) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new(record.allocation_epoch(), record.ordinal()).unwrap()
}

pub(super) fn child(root: &Path, role: &str) {
    let checkpoint = match role {
        "crash-expiry-wal" => PhysicalMutationCheckpoint::AfterWalDurability,
        "crash-expiry-root" => PhysicalMutationCheckpoint::AfterRootReplacement,
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
    fs::write(&pending, token.encode()).unwrap();
    fs::rename(pending, token_path).unwrap();
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
    thread::scope(|workers| {
        workers.spawn(|| {
            let deadline = Instant::now() + Duration::from_secs(150);
            while Instant::now() < deadline {
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

fn selected_object(serving: &ServingPhysicalRuntime, session: [u8; 16]) -> [u8; 16] {
    selected_blob_records(serving)
        .into_iter()
        .find_map(|(_, bytes)| match decode_blob_record(&bytes) {
            Ok(BlobRecordV1::SessionDeclared(value)) if value.session() == session => {
                Some(value.object())
            }
            _ => None,
        })
        .expect("child selected declaration binds object")
}
