//! `G7` / `M4`: checkpoint cutover carries the predecessor's verified manifest
//! roster forward by its routing delta. On a real Store whose roster comes
//! from genuine released blob drops, the carried roster equals an independent
//! full-scan oracle pin for pin, in order, across drop additions, retirement,
//! route additions, rewrites and reopen, while the work it inspects tracks the touched routes
//! rather than every route in the selected root.
use worth_store::physical_runtime::{
    BlobReclaimReceipt, BlobReclaimRetirement, BlobReclaimRetirementBudget,
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest, PhysicalMutationDeadline, ServingPhysicalRuntime,
};
use worth_store_testkit_store::fixture::{
    placement, serving_from_initialization, serving_from_open,
};
use worth_store_testkit_store::physical_blob_journeys::checkpoint_capture_envelope::release_one_batch;
use worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::selected_segment_rewrite::prepare_rewrite;
use worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::{
    completed, prepare_records,
};

/// Appends `count` ordinary records in publications of 32.
fn fill(serving: &ServingPhysicalRuntime, material: u8, count: usize) {
    for chunk in 0..count.div_ceil(32) {
        let payloads = (chunk * 32..count.min(chunk * 32 + 32))
            .map(|index| format!("route-{material}-{index}").into_bytes())
            .collect::<Vec<_>>();
        let borrowed = payloads.iter().map(Vec::as_slice).collect::<Vec<_>>();
        let mut key = [0xc7; 32];
        key[0] = material;
        key[1] = chunk as u8;
        completed(prepare_records(serving, placement(), key, &borrowed).execute());
    }
}

struct Audit {
    carried: bool,
    attempts: Vec<[u8; 16]>,
    inspected: u64,
    routes: u64,
}

/// One production cutover derivation, checked against the full-scan oracle.
fn audit(serving: &ServingPhysicalRuntime) -> Audit {
    let (carried, pins, oracle, inspected, routes) =
        serving.certification_cutover_pin_audit().unwrap();
    assert_eq!(
        pins, oracle,
        "carried roster must equal the full-scan oracle"
    );
    Audit {
        carried,
        attempts: pins,
        inspected,
        routes,
    }
}

fn checkpoint(serving: &ServingPhysicalRuntime, key: u8) {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([key; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        serving.checkpoints().start(request).into_raw()
    else {
        panic!("checkpoint must admit");
    };
    assert!(matches!(
        handle.wait(),
        PhysicalCheckpointOutcome::Completed(_)
    ));
}

fn retire(serving: &ServingPhysicalRuntime, receipt: &mut BlobReclaimReceipt) {
    let budget = BlobReclaimRetirementBudget::new(
        std::num::NonZeroU32::new(2).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    );
    for _ in 0..16 {
        match serving
            .blobs()
            .unwrap()
            .continue_reclaim_retirement(receipt, budget)
            .unwrap()
        {
            BlobReclaimRetirement::Pending(_) => continue,
            _ => return,
        }
    }
    panic!("released drop retirement must settle");
}

/// Real released drops: each holds its manifest (and the exclusive
/// publication lease) until its covering checkpoint lets it retire, so each
/// drop is one roster addition followed by one removal.
#[test]
fn carried_cutover_roster_matches_full_scan_across_released_drops() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    fill(&serving, 1, 48);
    audit(&serving);
    let mut first = release_one_batch(&serving, 0x31, true);
    let added = audit(&serving);
    assert!(added.carried);
    assert_eq!(added.attempts.len(), 1);
    // An unchanged root re-admits the carried roster without reading a route.
    let unchanged = audit(&serving);
    assert!(unchanged.carried);
    assert_eq!(unchanged.inspected, 0);
    assert_eq!(unchanged.attempts, added.attempts);
    // Retirement rewrites the drop's records and removes its batch routes.
    checkpoint(&serving, 0x41);
    retire(&serving, &mut first);
    let retired = audit(&serving);
    assert!(retired.carried);
    // Ordinary route additions leave the roster unchanged.
    fill(&serving, 2, 8);
    let rewritten = audit(&serving);
    assert!(rewritten.carried);
    assert_eq!(rewritten.attempts, retired.attempts);
    let mut second = release_one_batch(&serving, 0x32, true);
    let readded = audit(&serving);
    assert_eq!(readded.attempts.len(), rewritten.attempts.len() + 1);
    assert!(readded
        .attempts
        .iter()
        .any(|attempt| !rewritten.attempts.contains(attempt)));
    checkpoint(&serving, 0x42);
    retire(&serving, &mut second);
    audit(&serving);
    // Released-drop custody reopens only through C.8 rejoin; the ordinary
    // reopen fallback is proven on a record-only Store below.
    serving.close();
}

/// The cheapest of three one-record edits, each carried from the roster
/// derived immediately before it. Background maintenance publishing between
/// the two derivations only adds touched routes, so the minimum isolates the
/// edit's own cost.
fn one_record_edit(serving: &ServingPhysicalRuntime, material: u8) -> Audit {
    (0..3)
        .map(|trial| {
            audit(serving);
            fill(serving, material + trial, 1);
            let edit = audit(serving);
            assert!(edit.carried);
            edit
        })
        .min_by_key(|edit| edit.inspected)
        .unwrap()
}

/// Ten times more routes, the same one-record edit: the carried cutover's
/// work stays bounded by the touched path while the full scan grows with
/// every route; reopen falls back to one full rebuild, then carries again.
#[test]
fn carried_cutover_work_tracks_touched_routes_not_total_routes() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    fill(&serving, 1, 320);
    let small = one_record_edit(&serving, 10);
    fill(&serving, 3, 2_880);
    let large = one_record_edit(&serving, 20);
    assert!(large.carried);
    assert!(
        large.routes >= small.routes * 8,
        "{} vs {}",
        large.routes,
        small.routes
    );
    assert!(
        large.inspected * 4 < large.routes,
        "{} vs {}",
        large.inspected,
        large.routes
    );
    // A publication's fresh random identities land in a few leaves anywhere in
    // the key space; that touched set, not the tree, sets the work, so it
    // grows strictly slower than the routes a full rebuild reads.
    assert!(
        large.inspected * small.routes < small.inspected * large.routes,
        "cutover work must track touched routes: {}/{} vs {}/{}",
        large.inspected,
        large.routes,
        small.inspected,
        small.routes
    );
    serving.close();

    let reopened = serving_from_open(&root);
    let rebuilt = audit(&reopened);
    assert!(!rebuilt.carried);
    assert_eq!(rebuilt.inspected, rebuilt.routes);
    let resumed = one_record_edit(&reopened, 30);
    assert!(
        resumed.inspected * 4 < resumed.routes,
        "{} vs {}",
        resumed.inspected,
        resumed.routes
    );
    // A segment rewrite genuinely moves every route it consolidates; the
    // carried delta follows those moves and still equals the oracle.
    completed(prepare_rewrite(&reopened, placement(), [0x40; 32]).execute());
    assert!(audit(&reopened).carried);
    reopened.close();
}
