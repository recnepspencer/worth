//! Owned V1 pair values remain historical while V2 reconstructs changed values.
use super::*;
use worth_query_host::facade::primary_graph::WorthQueryManagedDerivedViewDenial;
#[test]
fn changed_codec_pair_reconstruction_retains_old_owned_values_and_refuses_partial_images() {
    let original = host::install(journey::heavy()).unwrap();
    let _old = host::produce(&original);
    let retained = paired_native::reconstruct(&original, 96, false);
    let target = journey::successor(host::capture(&original));
    assert_eq!(successor::output::decodes(), 0);
    assert_eq!(
        retained
            .snapshot
            .get(&retained.keys[0])
            .unwrap()
            .unwrap()
            .first,
        96
    );
    assert_eq!(retained.owned.first, 96);
    assert!(matches!(
        target
            .runtime()
            .observe_managed_derived_view(&retained.view),
        Err(WorthQueryManagedDerivedViewDenial::ForeignApplication)
    ));
    let _new = successor::host::produce(&target);
    let current = successor::paired_native::reconstruct(&target, 1096, true);
    assert_eq!(current.owned.first, 1096);
    for key in &current.keys {
        assert_eq!(current.snapshot.get(key).unwrap().unwrap().second, 1096);
    }
    current.view.discard();
    assert!(current.snapshot.get(&current.keys[0]).is_err());
    assert_eq!(current.owned.second, 1096);
    retained.view.discard();
    assert!(retained.snapshot.get(&retained.keys[0]).is_err());
    assert_eq!(retained.owned.first, 96);
    drop(retained.view);
    assert_eq!(retained.owned.second, 96);
}
