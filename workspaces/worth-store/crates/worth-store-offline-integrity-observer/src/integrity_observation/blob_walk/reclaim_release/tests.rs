//! The first descriptor of a release, judged beside the generation that it
//! released from. A descriptor says that the records its manifest dropped are
//! gone: a record that the selected root still routes is not.

use super::super::reuse_source_fixture::{
    graphed, record, released_source, selected_source, unread, walk_over, RELEASE_DESCRIPTOR,
    RELEASE_MANIFEST, SOURCE_PUBLICATION,
};
use super::super::BlobRecordWalk;
use super::*;
use crate::integrity_observation::OfflineIndeterminatePhysicalReason as Indeterminate;

/// The root generation that `released_source` was released from.
const SOURCE_ROOT: u64 = 6;

/// A record that the release dropped beside the publication.
const DROPPED_BESIDE: u64 = 9;

/// The descriptor of the batch before a later one.
const PRIOR_DESCRIPTOR: u64 = 10;

fn aliased() -> Outcome {
    Outcome::Unknown(Unknown::PhysicalAliasNotReinspected)
}

fn bound() -> Outcome {
    Outcome::Indeterminate(Indeterminate::EntryBoundExceeded)
}

/// An inventory of routes that were all read, with their arena tiers.
fn routes_read() -> RouteInventory {
    let mut routes = RouteInventory::new();
    routes.note_free_space_header(&[0; 48 + 152]);
    routes
}

/// The descriptor's outcome in a walk over `released_source` once `change`
/// is written into it. The walk read every route of the selected root and of
/// the root that the release left, which still selects the publication.
fn descriptor_after(change: impl FnOnce(&mut BlobRecordWalk)) -> Outcome {
    let publication = selected_source().swap_remove(SOURCE_PUBLICATION);
    let released_from = walk_over(vec![publication])
        .into_historical_source(SOURCE_ROOT, true)
        .with_routes(routes_read());
    let mut walk = walk_over(released_source())
        .with_historical_source(released_from)
        .with_route_inventory(routes_read());
    change(&mut walk);
    let rows = graphed(walk).selected;
    assert_eq!(rows[RELEASE_MANIFEST].outcome, Outcome::Intact);
    rows[RELEASE_DESCRIPTOR].outcome.clone()
}

/// The release's first batch as one that also dropped `DROPPED_BESIDE`.
fn drop_beside(walk: &mut BlobRecordWalk) {
    if let Some(BlobFact::ReleasedReclaimDescriptor { manifest_count, .. }) =
        walk.selected[RELEASE_DESCRIPTOR].fact.as_mut()
    {
        *manifest_count = 2;
    }
    if let Some(BlobFact::ReleasedDropSetManifest { dropped, .. }) =
        walk.selected[RELEASE_MANIFEST].fact.as_mut()
    {
        dropped.push(record(DROPPED_BESIDE));
    }
}

/// Selected records cannot say that the WAL selected the release.
fn joined() -> Outcome {
    Outcome::Unknown(Unknown::WalCoverageUnavailable)
}

#[test]
fn a_release_descriptor_is_contradicted_by_a_dropped_record_that_is_still_routed() {
    assert_eq!(descriptor_after(|_| ()), joined());
    assert_eq!(descriptor_after(drop_beside), joined());

    // The selected root routes a record that the walk has a row of, read or
    // not: the record is not gone.
    let still_selected = descriptor_after(|walk| {
        let publication = selected_source().swap_remove(SOURCE_PUBLICATION);
        walk.selected.push(publication);
    });
    assert_eq!(still_selected, damage(Cause::Pointer));
    for outcome in [aliased(), damage(Cause::ChecksumMismatch)] {
        let still_routed = descriptor_after(|walk| {
            drop_beside(walk);
            let row = unread(record(DROPPED_BESIDE), None, outcome.clone());
            walk.selected.push(row);
        });
        assert_eq!(still_routed, damage(Cause::Pointer), "{outcome:?}");
    }
    // An unread record that the release did not drop says nothing of it.
    let beside = descriptor_after(|walk| {
        let row = unread(record(DROPPED_BESIDE), None, aliased());
        walk.selected.push(row);
    });
    assert_eq!(beside, joined());
}

/// A later batch of the release, which drops `DROPPED_BESIDE` alone, while the
/// publication is still selected.
fn later_batch(walk: &mut BlobRecordWalk) {
    let publication = selected_source().swap_remove(SOURCE_PUBLICATION);
    walk.selected.push(publication);
    if let Some(BlobFact::ReleasedReclaimDescriptor { predecessor, .. }) =
        walk.selected[RELEASE_DESCRIPTOR].fact.as_mut()
    {
        *predecessor = Some((record(PRIOR_DESCRIPTOR), [25; 32]));
    }
    if let Some(BlobFact::ReleasedDropSetManifest { dropped, .. }) =
        walk.selected[RELEASE_MANIFEST].fact.as_mut()
    {
        *dropped = vec![record(DROPPED_BESIDE)];
    }
}

/// A walk that stopped has not visited every routed record, so a dropped
/// record with no row may still be routed. That decides nothing: the
/// descriptor is judged as in a walk that saw the record gone.
#[test]
fn a_dropped_record_the_walk_did_not_visit_does_not_damage_its_release_descriptor() {
    // The root that the release left selects no prior descriptor.
    let prior_unavailable = Outcome::Unknown(Unknown::ParentScopeUnavailable);
    assert_eq!(descriptor_after(later_batch), prior_unavailable);
    let stopped = descriptor_after(|walk| {
        later_batch(walk);
        walk.note_walk_stopped(&bound());
    });
    assert_eq!(stopped, prior_unavailable);

    // A record that the walk has a row of is routed, stopped or not.
    let stopped_at_the_record = descriptor_after(|walk| {
        later_batch(walk);
        walk.note_walk_stopped(&bound());
        let row = unread(record(DROPPED_BESIDE), None, bound());
        walk.selected.push(row);
    });
    assert_eq!(stopped_at_the_record, damage(Cause::Pointer));
}
