#[allow(unused_imports)] use crate::physical_record_journeys::maintenance_interference::held::*;
use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::PhysicalRetirementDenial, worth_store::physical_runtime::RecordReadDenial};

use {worth_store_testkit_store::physical_record_journeys::maintenance_interference::append, worth_store_testkit_store::physical_record_journeys::maintenance_interference::append_copies, worth_store_testkit_store::physical_record_journeys::maintenance_interference::checkpoint, worth_store_testkit_store::physical_record_journeys::maintenance_interference::initialize, worth_store_testkit_store::physical_record_journeys::maintenance_interference::limits, worth_store_testkit_store::physical_record_journeys::maintenance_interference::placement, worth_store_testkit_store::physical_record_journeys::maintenance_interference::segment_files, worth_store_testkit_store::physical_record_journeys::maintenance_interference::segment_ids};

#[test]
fn held_readers_block_retirement_without_scanning_every_root() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = initialize(&root);
    let policy = placement();
    let mut payload = 0_u64;
    let mut ordinal = 1_u64;
    let mut older = Vec::new();
    let mut checkpointed_rotations = 0_u64;
    let record = [0x11; 4096];
    let copies = 4;
    while payload < 2 * 1024 * 1024 || segment_ids(&root).len() < 8 || older.len() < 7 {
        if append_copies(&serving, policy, ordinal, &record, copies).is_none() {
            checkpoint(&serving, ordinal);
            ordinal += 1;
            assert!(
                append_copies(&serving, policy, ordinal, &record, copies).is_some(),
                "batch {ordinal} stayed denied after reclaim charged={}",
                serving.certification_charged_growth_bytes()
            );
        }
        payload += copies as u64 * record.len() as u64;
        let rotations = wal_rotations(&serving);
        if rotations > checkpointed_rotations {
            checkpoint(&serving, ordinal + 20_000);
            checkpointed_rotations = rotations;
        }
        if segment_ids(&root).len() >= 8 && older.len() < 7 {
            older.push(serving.records().unwrap());
        }
        ordinal += 1;
        assert!(ordinal < 400, "world did not reach the scale predicates");
    }
    let tail_record = append(&serving, policy, ordinal, b"phase6-tail");
    ordinal += 1;
    let tail = serving.records().unwrap();
    let second = serving.records().unwrap();
    let mut held = tail.open(tail_record, limits()).unwrap();
    let _borrowed = held.next_chunk().unwrap().unwrap();
    assert_eq!(
        serving
            .read_protection_observer()
            .snapshot()
            .protected_roots(),
        older.len() as u32 + 1
    );
    let extent = append(&serving, policy, ordinal, &[0x5A; 20_000]);
    assert!(
        root.join("families/records/arenas")
            .read_dir()
            .unwrap()
            .next()
            .is_some(),
        "the large record must land in an arena"
    );
    assert!(segment_ids(&root).len() >= 8);
    assert!(payload >= 32 * 64 * 1024);
    let rotations = serving
        .record_submission()
        .wal_observation()
        .map(|observation| observation.rotations())
        .unwrap_or(0);
    assert!(
        rotations >= 2,
        "two WAL rotations are part of the world, observed {rotations}"
    );
    assert!(matches!(
        tail.open(extent, limits()),
        Err(error) if error.denial() == worth_store::physical_runtime::RecordReadDenial::RecordNotFound
    ));
    let examined = serving
        .read_protection_observer()
        .snapshot()
        .examined_entries();
    let protected = serving
        .read_protection_observer()
        .snapshot()
        .protected_roots();
    let submission = serving.record_submission();
    let mut material = [0x6B; 32];
    material[..8].copy_from_slice(&ordinal.to_le_bytes());
    let key = submission
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(material))
        .unwrap();
    let prepared = match submission
        .rewrite_selected_inline_segment(
            policy,
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            ),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        worth_proof::TransitionOutcome::Success(_) => {
            panic!("the first rewrite must still perform its publication")
        }
        worth_proof::TransitionOutcome::Denied(denial) => panic!("rewrite denied: {denial:?}"),
        worth_proof::TransitionOutcome::Deferred(deferred) => panic!("rewrite deferred: {deferred:?}"),
        worth_proof::TransitionOutcome::Stale(stale) => panic!("rewrite stale: {stale:?}"),
        worth_proof::TransitionOutcome::RebindRequired(rebind) => panic!("rewrite rebind: {rebind:?}"),
        worth_proof::TransitionOutcome::Failed(failure) => panic!("rewrite failed: {failure:?}"),
    };
    match prepared.execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_) => {}
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            panic!("rewrite had no effect: {:?}", fate.cause())
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!("rewrite became indeterminate at {:?}", fate.stage())
        }
    }
    let before = serving.media_counters().deletions();
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(worth_store::physical_runtime::PhysicalRetirementDenial::Protected)
    );
    let examined_delta = serving
        .read_protection_observer()
        .snapshot()
        .examined_entries()
        - examined;
    assert!(
        examined_delta <= 4,
        "retirement examined {examined_delta} entries across {protected} protected roots"
    );
    assert_eq!(serving.media_counters().deletions(), before);
    drop(held);
    drop(tail);
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(worth_store::physical_runtime::PhysicalRetirementDenial::Protected)
    );
    drop(second);
    drop(older);
    let occupied = segment_files(&root);
    serving
        .retire_displaced_segment()
        .expect("the last protector must admit bounded retirement");
    let remaining = segment_files(&root);
    assert!(
        occupied.difference(&remaining).next().is_some(),
        "eligible retirement must remove the displaced segment file"
    );
    let resident = serving
        .certification_physical_residency()
        .counters()
        .peak_resident_bytes();
    assert!(
        resident <= 64 * 1024,
        "resident frames peaked at {resident}"
    );
    serving.close();
}

#[test]
fn tail_index_cost_stays_flat_from_one_root_to_sixteen() {
    let one = retirement_examination(1);
    let sixteen = retirement_examination(16);
    assert_eq!(one, sixteen);
    assert!(one >= 1, "lookup cost was {one}");
    assert!(one <= 4, "lookup cost {one} grew with the root set");
}
