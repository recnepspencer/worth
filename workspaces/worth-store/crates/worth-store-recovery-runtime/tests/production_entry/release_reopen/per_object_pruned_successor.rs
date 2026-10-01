//! A non-global-tip A head survives native WAL cutover and authorizes A's successor.

use super::*;
use std::{fmt::Write, fs, path::Path, process::Command};
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, ManifestEntryCapacity, PhysicalRecordFormatDeclaration,
    PhysicalRecordPlacementPolicy, RecordByteLimit, RecordCountLimit, RecordScanOutcome,
    RecordScanRequest,
};
use worth_store_physical_format::{
    decode_blob_record, release_checkpoint_batch_records_digest_v1, BlobReclaimSourceBasisV1,
    BlobRecordV1, ReleaseCustodyHeadKeyV1, ReleasedGenerationReclaimBasisV1,
};
use worth_store_recovery_runtime::{PhysicalRecoveryOutcome, WorthStoreRecovery};

#[path = "per_object_pruned_successor/selected_controls.rs"]
mod selected_controls;

const WAL_SEGMENT_BYTES: u64 = 1 << 20;
const CHILD_TEST: &str = "release_reopen::per_object_pruned_successor::fresh_child_continues_a";
const ROOT_ENV: &str = "WORTH_C11_PRUNED_A_ROOT";
const OBJECT_ENV: &str = "WORTH_C11_PRUNED_A_OBJECT";
const GENERATION_ENV: &str = "WORTH_C11_PRUNED_A_GENERATION";

#[test]
fn non_global_a_head_survives_pruned_wal_and_fresh_successor() {
    let wal_bytes = NonZeroU64::new(WAL_SEGMENT_BYTES).unwrap();
    let (world, first, first_publication, _) = released_world_with_wal_segment_bytes(1, wal_bytes);
    assert!(
        first.remaining_payload_records() > 0,
        "A remains nonterminal"
    );
    let (a_batches, a_accumulator) = selected_release_certificates(&world);
    assert_eq!(a_batches.len(), 1);
    let first_heads = selected_head_oracle::selected_heads(world.root(), a_accumulator);
    let [a_head] = first_heads.as_slice() else {
        panic!("first checkpoint must select exactly A's keyed head");
    };
    assert!(!a_head.terminal());
    let a_key = a_head.key();
    let a_wal = a_batches[0].fate();
    let c1 = fs::read(world.root().join("families/checkpoint.current")).unwrap();

    let (b_accumulator, b_key) =
        two_generations::release_second_object(&world, first_publication, a_accumulator);
    assert_ne!(a_key, b_key);
    assert_eq!(
        b_accumulator.base().tip().descriptor_record(),
        selected_release_certificates(&world).0[0].descriptor_record()
    );
    let b_heads = selected_head_oracle::selected_heads(world.root(), b_accumulator);
    assert_eq!(b_heads.len(), 2);
    assert_eq!(
        b_heads.iter().find(|head| head.key() == a_key),
        Some(a_head)
    );
    let b_head = *b_heads.iter().find(|head| head.key() == b_key).unwrap();
    let c2 = fs::read(world.root().join("families/checkpoint.current")).unwrap();
    assert_ne!(c2, c1, "B must publish a successor checkpoint");

    super::super::selected_tag7_pruned_wal::rotate_completed_wal(world.serving());
    checkpoint(world.serving(), [0xa3; 32]);
    let c3 = fs::read(world.root().join("families/checkpoint.current")).unwrap();
    assert_ne!(
        c3, c2,
        "native checkpoint replacement supersedes B's checkpoint"
    );
    let (idle_batches, idle_accumulator) = selected_release_certificates(&world);
    assert!(idle_batches.is_empty());
    assert_eq!(idle_accumulator.base().tip(), b_accumulator.base().tip());
    assert_eq!(
        selected_head_oracle::selected_heads(world.root(), idle_accumulator),
        b_heads,
    );
    assert!(
        !super::super::selected_tag7_pruned_wal::retained_wal_interval(
            world.root(),
            a_wal.lsn_start(),
            a_wal.lsn_end_exclusive(),
        ),
        "native checkpoint cutover must physically prune A's old C9 member"
    );

    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
        .env(ROOT_ENV, &root)
        .env(OBJECT_ENV, encode_object(a_key.object()))
        .env(GENERATION_ENV, a_key.generation().to_string())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "fresh A successor process: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let c4 = fs::read(root.join("families/checkpoint.current")).unwrap();
    assert_ne!(c4, c3, "A successor must publish its own checkpoint");
    let (successor_batches, successor_accumulator) = selected_release_certificates_from_bytes(&c4);
    let [successor] = successor_batches.as_slice() else {
        panic!("A successor checkpoint must contain one new Batch");
    };
    let predecessor = successor.predecessor().expect("A successor predecessor");
    assert_eq!(predecessor.descriptor_record(), a_head.descriptor_record());
    assert_eq!(
        predecessor.descriptor_frame_sha256(),
        a_head.descriptor_frame_sha256()
    );
    assert_eq!(
        successor_accumulator.base().prior_cumulative_dropped(),
        idle_accumulator.base().cumulative_dropped()
    );
    assert_eq!(
        successor_accumulator.base().prior_cumulative_digest(),
        idle_accumulator.base().cumulative_digest()
    );
    assert_eq!(
        successor_accumulator.base().tip(),
        successor.tip_provenance().unwrap()
    );
    assert_eq!(
        successor_accumulator.base().cumulative_dropped(),
        successor.cumulative_dropped(),
    );
    assert_eq!(
        successor_accumulator.base().cumulative_digest(),
        successor.cumulative_digest(),
    );
    assert_eq!(
        successor_accumulator.base().batch_records_digest(),
        release_checkpoint_batch_records_digest_v1(&successor_batches).unwrap(),
    );
    assert_eq!(
        successor_accumulator.prior_head_count(),
        idle_accumulator.head_count()
    );
    assert_eq!(
        successor_accumulator.prior_head_roster_digest(),
        idle_accumulator.head_roster_digest(),
    );
    let final_heads = selected_head_oracle::selected_heads(&root, successor_accumulator);
    assert_eq!(final_heads.len(), 2);
    assert_eq!(
        final_heads.iter().find(|head| head.key() == b_key),
        Some(&b_head)
    );
    let a_next = final_heads.iter().find(|head| head.key() == a_key).unwrap();
    assert_eq!(a_next.descriptor_record(), successor.descriptor_record());
    assert_eq!(
        a_next.descriptor_frame_sha256(),
        successor.descriptor_frame_sha256(),
    );
    assert_eq!(a_next.cumulative_dropped(), a_head.cumulative_dropped() + 1,);
    assert_eq!(a_next.source_basis_digest(), a_head.source_basis_digest());

    let serving = recover_serving(&root, wal_bytes);
    selected_controls::assert_selected_successor_controls(
        &serving,
        *a_next,
        *successor,
        idle_accumulator,
    );
    checkpoint(&serving, [0xa4; 32]);
    serving.close();
    let (carry_batches, carry_accumulator) = selected_release_certificates_from_bytes(
        &fs::read(root.join("families/checkpoint.current")).unwrap(),
    );
    assert!(carry_batches.is_empty());
    assert_eq!(
        carry_accumulator.base().tip(),
        successor_accumulator.base().tip()
    );
    assert_eq!(
        selected_head_oracle::selected_heads(&root, carry_accumulator),
        final_heads
    );
    recover_serving(&root, wal_bytes).close();
}

#[test]
fn fresh_child_continues_a() {
    let Some(root) = std::env::var_os(ROOT_ENV) else {
        return;
    };
    let root = Path::new(&root);
    let object = decode_object(&std::env::var(OBJECT_ENV).unwrap());
    let generation = std::env::var(GENERATION_ENV).unwrap().parse().unwrap();
    let a_key = ReleaseCustodyHeadKeyV1::new(object, generation).unwrap();
    let wal_bytes = NonZeroU64::new(WAL_SEGMENT_BYTES).unwrap();
    let serving = recover_serving(root, wal_bytes);
    let (_, accumulator) = selected_release_certificates_from_bytes(
        &fs::read(root.join("families/checkpoint.current")).unwrap(),
    );
    let heads = selected_head_oracle::selected_heads(root, accumulator);
    let a_head = heads.iter().find(|head| head.key() == a_key).unwrap();
    assert!(!a_head.terminal());
    let source = selected_source_for_key(&serving, a_key);
    let proof = AdmittedBlobReleaseProof::certification_admit(
        serving.store_identity().bytes(),
        source.object(),
        source.generation(),
        source.publication_record().allocation_epoch(),
        source.publication_record().ordinal(),
        source.publication_frame_sha256(),
        source.issuer_evidence_sha256(),
    )
    .unwrap();
    let placement = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(64).unwrap())
        .admit(AdmittedPhysicalRecordFormat::admit(
            PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
        ))
        .unwrap();
    let request = BlobReclaimRequest::released(
        proof,
        placement,
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
        BlobReclaimLimits::new(
            NonZeroU64::new(128).unwrap(),
            NonZeroU64::new(32 << 20).unwrap(),
            NonZeroU16::new(1).unwrap(),
        )
        .unwrap(),
    );
    let receipt = serving
        .blobs()
        .unwrap()
        .reclaim(request)
        .unwrap()
        .wait()
        .unwrap();
    assert_eq!(receipt.disposition(), BlobReclaimDisposition::Dropped);
    assert_eq!(receipt.dropped_records().len(), 1);
    serving.close();
}

fn recover_serving(
    root: &Path,
    wal_bytes: NonZeroU64,
) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    let outcome =
        WorthStoreRecovery::recover(super::super::certified_release_serving::request(root));
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        match outcome {
            PhysicalRecoveryOutcome::Blocked(block) => panic!(
                "A/B recovery blocked at {:?}: {:?}; limit={:?}; effects={}",
                block.kind,
                block.evidence().planning_denial,
                block.evidence().limit,
                block.recovery_effects(),
            ),
            PhysicalRecoveryOutcome::PublicationIndeterminate(failure) => panic!(
                "A/B recovery indeterminate: reopen={:?}; handoff={:?}; effects={}",
                failure.reopen_failure(),
                failure.handoff_failure(),
                failure.recovery_effects(),
            ),
            other => panic!("fresh process must recover A/B selected custody: {other:?}"),
        }
    };
    let seal = handoff
        .into_core()
        .into_checkpoint_custody()
        .expect("Store seal");
    super::super::certified_release_serving::admit_serving_with_seal_and_wal_segment_bytes(
        root, seal, wal_bytes,
    )
}

fn checkpoint(serving: &worth_store::physical_runtime::ServingPhysicalRuntime, key: [u8; 32]) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new(key),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("native checkpoint must admit")
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
}

fn selected_source_for_key(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    key: ReleaseCustodyHeadKeyV1,
) -> ReleasedGenerationReclaimBasisV1 {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            RecordScanRequest::from_start()
                .with_batch_limit(RecordCountLimit::new(8).unwrap())
                .with_payload_limit(RecordByteLimit::new(4096).unwrap()),
        )
        .unwrap();
    let mut scratch = [0_u8; 8192];
    let mut matches = Vec::new();
    while let RecordScanOutcome::Batch(batch) = scan.read_next_into(&mut scratch).unwrap() {
        for index in 0..batch.records().len() {
            let Some(bytes) = batch.payload(index) else {
                continue;
            };
            let Ok(BlobRecordV1::DropSetManifestV3(manifest)) = decode_blob_record(bytes) else {
                continue;
            };
            let BlobReclaimSourceBasisV1::ReleasedGeneration(source) = manifest.source_basis()
            else {
                continue;
            };
            if source.object() == key.object() && source.generation() == key.generation() {
                matches.push(source);
            }
        }
        if batch.is_complete() {
            break;
        }
    }
    let [source] = matches.as_slice() else {
        panic!("exactly one selected A source basis must match the recorded keyed head")
    };
    *source
}

fn encode_object(object: [u8; 16]) -> String {
    let mut encoded = String::new();
    for byte in object {
        write!(&mut encoded, "{byte:02x}").unwrap();
    }
    encoded
}

fn decode_object(encoded: &str) -> [u8; 16] {
    assert_eq!(encoded.len(), 32);
    std::array::from_fn(|index| u8::from_str_radix(&encoded[index * 2..index * 2 + 2], 16).unwrap())
}
