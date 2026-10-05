//! Ordering the retirements above the checkpoint reads the manifests of every
//! root generation among them. A request that admits too few manifest entries
//! for that walk must be told so, not that a WAL image is an invalid target.

use super::super::*;
use pending_wal_world::{PendingWalWorld, Tail, Workload};
use worth_store::physical_runtime::ServingPhysicalRuntime;
use worth_store_recovery_runtime::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDimension, PhysicalRecoveryOutcome,
    PhysicalRecoveryPageAdmissionDenial, PhysicalRecoveryPlanningDenial, WorthStoreRecovery,
};

/// The entries the two-chunk worlds admit; the idle world recovers under
/// them.
const SUFFICIENT: u64 = 4096;

/// What the idle world needs. Fixed as well as measured: a measured need
/// follows whatever planning counts, so alone it cannot tell whether
/// planning counted the entries discovery charged before it.
const IDLE_NEED: u64 = 524;

/// A limit the released world's walk runs out under, on a charge of one
/// entry.
const RELEASED_WALK_LIMIT: u64 = 193;

/// The limit the released world's walk fits under exactly, which leaves no
/// entry for the source root its drop reads next.
const RELEASED_SOURCE_ROOT_LIMIT: u64 = 209;

/// Whether planning succeeds under this many entries. Every block is under
/// the need, whichever phase raised it and whether or not it named a limit.
fn plans(root: &Path, manifest_entries: u64) -> bool {
    let selected = certified_release_serving::request_with_manifest_entries(root, manifest_entries)
        .admit()
        .ok()
        .and_then(|admitted| admitted.discover().ok())
        .and_then(|discovered| discovered.select().ok());
    match selected.map(|selected| selected.plan()) {
        None => false,
        Some(Err(PhysicalRecoveryOutcome::Blocked(blocked))) => {
            assert_eq!(blocked.recovery_effects(), 0);
            false
        }
        Some(Err(outcome)) => panic!("planning neither blocked nor planned: {outcome:?}"),
        Some(Ok(planned)) => {
            let PhysicalRecoveryOutcome::Refused(cancelled) = planned.cancel_before_execution()
            else {
                panic!("a measured plan must cancel without execution")
            };
            assert_eq!(cancelled.recovery_effects(), 0);
            true
        }
    }
}

/// The fewest manifest entries planning succeeds under. Planning has no
/// effect, so the same media answers every probe.
fn planning_need(root: &Path) -> u64 {
    assert!(plans(root, SUFFICIENT));
    let (mut denied, mut admitted) = (0, SUFFICIENT);
    while admitted - denied > 1 {
        let probe = denied + (admitted - denied) / 2;
        if plans(root, probe) {
            admitted = probe;
        } else {
            denied = probe;
        }
    }
    admitted
}

/// The count the refused charge would have reached, from a recovery that
/// must block on the manifest-entry limit it was admitted under.
fn refused_at(world: &PendingWalWorld, admitted: u64, stage: &str) -> u64 {
    let outcome = WorthStoreRecovery::recover(
        certified_release_serving::request_with_manifest_entries(world.root(), admitted),
    );
    let PhysicalRecoveryOutcome::Blocked(blocked) = outcome else {
        panic!("{stage}: {admitted} entries must block: {outcome:?}")
    };
    assert_eq!(blocked.recovery_effects(), 0);
    let evidence = blocked.evidence();
    // Discovery charged entries of the same limit before planning.
    assert!(
        (1..admitted).contains(&evidence.counters.manifest_entries),
        "{stage}: discovery charged {} of {admitted}",
        evidence.counters.manifest_entries,
    );
    assert_eq!(
        (blocked.kind, &evidence.planning_denial),
        (
            PhysicalRecoveryBlockKind::PageAdmission,
            &Some(PhysicalRecoveryPlanningDenial::Page(
                PhysicalRecoveryPageAdmissionDenial::ManifestEntryLimit
            ))
        ),
        "{stage}: running out of {admitted} entries is that limit, not a denied target",
    );
    let limit = evidence.limit.expect("typed manifest-entry limit");
    assert_eq!(
        limit.dimension,
        PhysicalRecoveryLimitDimension::ManifestEntries
    );
    assert_eq!(limit.admitted, admitted);
    limit.observed
}

/// Just under the need, the walk is the consumer that runs out: the selected
/// inventory before it fit, and with no release above the checkpoint nothing
/// after the walk reads a manifest. Answers the need.
fn assert_one_entry_short_reports_the_limit(world: &PendingWalWorld, stage: &str) -> u64 {
    let need = planning_need(world.root());
    let mut crossings = Vec::new();
    for admitted in [need - 1, need - 2, need - 3] {
        let observed = refused_at(world, admitted, stage);
        // The refused charge crossed the limit without passing the need, and
        // one entry short it is the charge that reaches the need.
        assert!(
            admitted < observed && observed <= need,
            "{stage}: observed {observed} under {admitted} of {need}",
        );
        if admitted + 1 == need {
            assert_eq!(observed, need, "{stage}");
        }
        crossings.push(observed);
    }
    // The walk ends on the records of one routing leaf, charged together,
    // so the two shorter limits fall inside that one charge. The
    // report names the count the charge would have reached, not one more
    // than whichever limit refused it.
    assert_eq!(crossings[1], crossings[2], "{stage}: {crossings:?}");
    need
}

/// Recovers under exactly the need. The blocked attempts left the media as
/// the kill did, and what planning needed is all recovery needs.
fn serve_under(world: &PendingWalWorld, need: u64, stage: &str) -> ServingPhysicalRuntime {
    let outcome = WorthStoreRecovery::recover(
        certified_release_serving::request_with_manifest_entries(world.root(), need),
    );
    let PhysicalRecoveryOutcome::Recovered(handoff) = outcome else {
        panic!("{stage}: {need} entries must recover: {outcome:?}")
    };
    let seal = handoff
        .into_core()
        .into_checkpoint_custody()
        .expect("recovered custody seal");
    certified_release_serving::admit_serving_with_seal(world.root(), seal)
}

/// No release walked the history: the walk runs only for the retired pages.
#[test]
fn a_walk_for_retired_pages_one_manifest_entry_short_reports_that_limit() {
    let world =
        pending_wal_world::published_above_checkpoint(Workload::ThreeSmallObjects, Tail::Idle);
    let need = assert_one_entry_short_reports_the_limit(&world, "published above the checkpoint");
    assert_eq!(
        need, IDLE_NEED,
        "the need counts the entries discovery charged before planning",
    );
    let serving = serve_under(&world, need, "published above the checkpoint");
    world.assert_objects_read_back(&serving);
    serving.close();
}

/// A release completed above the checkpoint: the walk runs for its drop,
/// which then reads the source root it released from. The limits these
/// tests admit are fixed, not measured against a need.
fn released_world() -> PendingWalWorld {
    let world = pending_wal_world::first();
    world.kill_successor_of_first_object();
    world
}

#[test]
fn a_walk_for_a_historical_release_out_of_manifest_entries_reports_that_limit() {
    let observed = refused_at(
        &released_world(),
        RELEASED_WALK_LIMIT,
        "the walk for a release above the checkpoint",
    );
    assert_eq!(observed, RELEASED_WALK_LIMIT + 1);
}

#[test]
fn a_source_root_of_a_historical_release_out_of_manifest_entries_reports_that_limit() {
    let observed = refused_at(
        &released_world(),
        RELEASED_SOURCE_ROOT_LIMIT,
        "the source root of a release above the checkpoint",
    );
    assert_eq!(observed, RELEASED_SOURCE_ROOT_LIMIT + 1);
}
