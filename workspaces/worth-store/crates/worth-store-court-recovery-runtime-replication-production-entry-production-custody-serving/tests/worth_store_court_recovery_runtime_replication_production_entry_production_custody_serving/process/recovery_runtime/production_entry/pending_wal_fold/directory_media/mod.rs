//! A released V3 edge's replacement directory frame is reread from selected
//! media by Store. A structurally valid substitute frame keeps every C.8/C.9
//! proof intact, so only Store's independent media rejoin can deny it.

#[allow(unused_imports)]
use crate::pending_wal_fold::directory_media::*;

#[test]
fn substituted_replacement_directory_frame_denies_store_rejoin() {
    let world = worth_store_testkit_recovery_runtime::production_entry::pending_wal_world::first();
    world.kill_distinct_release_before_checkpoint();
    let request = || {
        worth_store_testkit_recovery_runtime::production_entry::certified_release_serving::recovery_request::request(world.root())
    };
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::Recovered(handoff) =
        worth_store_recovery_runtime::WorthStoreRecovery::recover(request())
    else {
        panic!("genuine two-batch release must recover before media substitution");
    };
    drop(handoff);
    // The first released edge whose drop invalidated the watermark published
    // an unindexed replacement directory; a later edge superseded it, so only
    // the historical rejoin of that edge rereads this frame.
    let manifests = root_manifests(world.root());
    let (replacement_root, replacement) = manifests
        .iter()
        .find_map(|manifest| {
            manifest
                .derived_family_directory()
                .filter(|binding| binding.indexed_through_blob_publication().is_none())
                .map(|binding| (manifest, binding.directory_record()))
        })
        .expect("a released edge published a replacement directory");
    let selected = manifests.last().unwrap();
    assert_ne!(
        selected
            .derived_family_directory()
            .map(|binding| binding.directory_record()),
        Some(replacement),
        "the substituted frame must not be the selected directory",
    );
    let route = routes(world.root(), replacement_root)
        .into_iter()
        .find(|route| route.record() == replacement)
        .expect("replacement root routes its directory");
    substitute_directory(world.root(), route);

    let outcome = worth_store_recovery_runtime::WorthStoreRecovery::recover(request());
    let worth_store_recovery_runtime::PhysicalRecoveryOutcome::PublicationIndeterminate(
        indeterminate,
    ) = outcome
    else {
        panic!("substituted replacement directory media must deny Store rejoin: {outcome:?}")
    };
    assert_eq!(
        indeterminate.handoff_failure(),
        Some(
            worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial::RejoinSelectedMedia(
                worth_store::physical_runtime::PhysicalRecoverySelectedRejoinMismatch::RoutingFrame,
            )
        ),
    );
}
