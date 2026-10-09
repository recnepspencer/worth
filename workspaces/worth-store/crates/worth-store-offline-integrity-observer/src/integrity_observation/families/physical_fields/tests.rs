use super::{blob_record_kind_is_declared, route_metadata_valid};

fn blob_route(kind: u8) -> [u8; 7] {
    [2, kind, 0, 0, 0, 0, 0]
}

#[test]
fn a_route_may_classify_exactly_the_blob_kinds_the_format_declares() {
    for kind in 0..=u8::MAX {
        assert_eq!(
            route_metadata_valid(&blob_route(kind)),
            blob_record_kind_is_declared(kind),
            "blob kind {kind}"
        );
    }
    // The current reuse claim and the current released reclaim descriptor: a
    // store that routes either is not damaged routing.
    assert!(route_metadata_valid(&blob_route(15)));
    assert!(route_metadata_valid(&blob_route(16)));
    assert!(!route_metadata_valid(&blob_route(0)));
}

#[test]
fn route_metadata_admits_only_declared_classes_tiers_and_zero_reserve() {
    assert!(route_metadata_valid(&[0, 0, 0, 0, 0, 0, 0]));
    assert!(
        !route_metadata_valid(&[0, 0, 0, 0, 1, 0, 0]),
        "an unclassified legacy route is primary"
    );
    assert!(route_metadata_valid(&[1, 0, 0, 0, 2, 0, 0]));
    assert!(route_metadata_valid(&[4, 0, 0, 0, 0, 0, 0]));
    assert!(route_metadata_valid(&[3, 0, 1, 0, 0, 0, 0]));
    assert!(
        !route_metadata_valid(&[3, 0, 0, 0, 0, 0, 0]),
        "a tree node route names its family"
    );
    assert!(!route_metadata_valid(&[5, 0, 0, 0, 0, 0, 0]));
    assert!(!route_metadata_valid(&[1, 0, 0, 0, 3, 0, 0]));
    assert!(!route_metadata_valid(&[1, 0, 0, 0, 0, 1, 0]));
    assert!(!route_metadata_valid(&[2, 2, 1, 0, 0, 0, 0]));
    assert!(!route_metadata_valid(&[1, 0, 0, 0, 0, 0]));
}
