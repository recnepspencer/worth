use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial, PhysicalMutationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, PhysicalRetirementDenial,
    RecordReadDenial,
};

use super::{
    append, append_copies, checkpoint, initialize, limits, placement, segment_files, segment_ids,
};

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
        root.join("families/records/extents")
            .read_dir()
            .unwrap()
            .next()
            .is_some(),
        "the large record must land in an extent"
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
        Err(error) if error.denial() == RecordReadDenial::RecordNotFound
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
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(material))
        .unwrap();
    let prepared = match submission
        .rewrite_selected_inline_segment(
            policy,
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            ),
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        TransitionOutcome::Success(_) => {
            panic!("the first rewrite must still perform its publication")
        }
        TransitionOutcome::Denied(denial) => panic!("rewrite denied: {denial:?}"),
        TransitionOutcome::Deferred(deferred) => panic!("rewrite deferred: {deferred:?}"),
        TransitionOutcome::Stale(stale) => panic!("rewrite stale: {stale:?}"),
        TransitionOutcome::RebindRequired(rebind) => panic!("rewrite rebind: {rebind:?}"),
        TransitionOutcome::Failed(failure) => panic!("rewrite failed: {failure:?}"),
    };
    match prepared.execute() {
        PhysicalMutationOutcome::Completed(_) => {}
        PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            panic!("rewrite had no effect: {:?}", fate.cause())
        }
        PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!("rewrite became indeterminate at {:?}", fate.stage())
        }
    }
    let before = serving.media_counters().deletions();
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(PhysicalRetirementDenial::Protected)
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
    drop(_borrowed);
    drop(held);
    drop(tail);
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(PhysicalRetirementDenial::Protected)
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

fn retirement_examination(roots: u32) -> u64 {
    use std::num::NonZeroU32;
    use worth_store::physical_runtime::PhysicalReadProtectionPolicy;
    let parent = tempfile::tempdir().unwrap();
    let serving = super::initialize_protection(
        &parent.path().join("store"),
        PhysicalReadProtectionPolicy::new(
            NonZeroU32::new(64).unwrap(),
            NonZeroU32::new(roots).unwrap(),
        ),
    );
    let policy = placement();
    let mut readers = Vec::new();
    // Segment capacity is 8 pages. Stop on the first page of a segment so the
    // rewrite displaces that generation, and keep a reader on that root.
    let total = 8 * (roots - 1) + 1;
    for index in 0..total {
        append(&serving, policy, u64::from(index) + 1, b"root");
        if readers.len() as u32 == roots {
            readers.remove(0);
        }
        readers.push(serving.records().unwrap());
    }
    rewrite_current(&serving, policy, 20_000 + u64::from(roots));
    let examined = serving
        .read_protection_observer()
        .snapshot()
        .examined_entries();
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(worth_store::physical_runtime::PhysicalRetirementDenial::Protected)
    );
    let delta = serving
        .read_protection_observer()
        .snapshot()
        .examined_entries()
        - examined;
    assert!(
        delta >= 1,
        "retirement returned Protected without consulting the index"
    );
    drop(readers);
    serving.close();
    delta
}

fn rewrite_current(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    policy: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    ordinal: u64,
) {
    let mut material = [0x6B; 32];
    material[..8].copy_from_slice(&ordinal.to_le_bytes());
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(material))
        .unwrap();
    let prepared = match submission
        .rewrite_selected_inline_segment(
            policy,
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            ),
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        TransitionOutcome::Success(_) => panic!("rewrite {ordinal} did not stay prepared"),
        TransitionOutcome::Denied(_) => panic!("rewrite {ordinal} denied"),
        TransitionOutcome::Deferred(_) => panic!("rewrite {ordinal} deferred"),
        TransitionOutcome::Stale(_) => panic!("rewrite {ordinal} stale"),
        TransitionOutcome::RebindRequired(_) => panic!("rewrite {ordinal} rebind"),
        TransitionOutcome::Failed(_) => panic!("rewrite {ordinal} failed"),
    };
    match prepared.execute() {
        PhysicalMutationOutcome::Completed(_) => {}
        PhysicalMutationOutcome::ProvenNoEffect(_) => panic!("rewrite {ordinal} had no effect"),
        PhysicalMutationOutcome::Indeterminate(_) => panic!("rewrite {ordinal} indeterminate"),
    }
}

fn wal_rotations(serving: &worth_store::physical_runtime::ServingPhysicalRuntime) -> u64 {
    serving
        .record_submission()
        .wal_observation()
        .map(|observation| observation.rotations())
        .unwrap_or(0)
}
