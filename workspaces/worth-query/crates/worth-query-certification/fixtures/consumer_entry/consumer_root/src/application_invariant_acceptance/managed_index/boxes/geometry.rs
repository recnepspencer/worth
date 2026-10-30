//! The ordinary Source scan is the independent inclusive-box membership oracle.
use super::super::super::authentication;
use super::super::{host, prefix};
use super::{
    coordinates::{Binding, Coordinates},
    provider::{BoxIndex, Calls},
    query,
};
use std::sync::{atomic::Ordering, Arc};
use worth_query_consumer_values::{BoundsDenial, Box2, BoxIndexDenial};
use worth_query_decl::facade::{
    application_query::ApplicationManagedIndexBinding,
    application_schema::ApplicationScalarValueBinding,
};
use worth_query_host::facade::{application_entry::*, primary_graph::*};

pub(super) fn input(directory: &str, probe: Coordinates) -> query::FindBoxes {
    query::FindBoxes {
        directory: directory.to_owned(),
        probe,
    }
}
pub(super) fn not_ready(error: WorthQueryApplicationRequestQueryDenial) {
    let WorthQueryApplicationRequestQueryDenial::ManagedIndex(error) = error else {
        panic!("actual managed refusal: {error:?}")
    };
    assert!(matches!(
        error.cause(),
        WorthQueryManagedIndexDenialCause::Refused(WorthQueryManagedIndexRefusal::NotReady)
    ));
}
fn bounds_denial(error: WorthQueryApplicationRequestQueryDenial) {
    let WorthQueryApplicationRequestQueryDenial::ManagedIndex(error) = error else {
        panic!("actual provider refusal: {error:?}")
    };
    let WorthQueryManagedIndexDenialCause::Work(WorthQueryManagedIndexWorkDenial::Provider(cause)) =
        error.cause()
    else {
        panic!("typed numerical cause: {error:?}")
    };
    assert_eq!(
        cause.downcast_ref::<BoxIndexDenial>(),
        Some(&BoxIndexDenial::Bounds(BoundsDenial::ReversedAxis {
            axis: 0
        }))
    );
}
/// Test-only oracle uses authored raw coordinate comparisons, never the disputed BVH/predicate.
pub(super) fn expected(source: &[query::Row], probe: Coordinates) -> Vec<query::Row> {
    let mut expected = source
        .iter()
        .filter(|row| {
            row.enabled
                && row.bounds.lower_x <= probe.upper_x
                && probe.lower_x <= row.bounds.upper_x
                && row.bounds.lower_y <= probe.upper_y
                && probe.lower_y <= row.bounds.upper_y
        })
        .cloned()
        .collect::<Vec<_>>();
    expected.sort_by(|a, b| a.identity.cmp(&b.identity));
    expected
}
#[test]
fn public_box_provider_preserves_closed_boundaries_source_slots_and_typed_bounds_refusals() {
    std::thread::Builder::new()
        .name("managed-box-consumer".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(run)
        .unwrap()
        .join()
        .unwrap();
}
fn run() {
    let calls = Arc::new(Calls::default());
    let world = host::install_boxes(Arc::new(prefix::Calls::default()), Arc::clone(&calls));
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    assert_eq!(Binding::UNIT.unwrap().as_str(), Box2::LENGTH_UNIT);
    assert_eq!(Binding::FRAME.unwrap().as_str(), Box2::FRAME);
    let descriptor = BoxIndex::descriptor();
    let profile = descriptor.meaning().parts().probe.parts();
    assert_eq!(profile.unit.as_deref(), Some(Box2::LENGTH_UNIT));
    assert_eq!(profile.frame.as_deref(), Some(Box2::FRAME));
    let source = request
        .query(query::ReadSource {
            directory: "boxes".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(source.rows().len(), 6);
    assert_eq!(
        source
            .rows()
            .iter()
            .map(|row| row.identity.as_str())
            .collect::<Vec<_>>(),
        ["box-f", "box-e", "box-d", "box-c", "box-b", "box-a"]
    );
    assert!(!source
        .rows()
        .windows(2)
        .all(|rows| rows[0].coordinate < rows[1].coordinate));
    let broad = Coordinates::new(i64::MIN, i64::MIN, i64::MAX, i64::MAX);
    let index = request
        .query(input("boxes", broad))
        .open_index::<BoxIndex>()
        .unwrap();
    let before = calls.totals();
    not_ready(
        request
            .query(input("boxes", broad))
            .execute()
            .err()
            .expect("Created refuses"),
    );
    assert_eq!(calls.totals(), before);
    request
        .query(input("boxes", broad))
        .rebuild_index(&index)
        .unwrap();
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    let mut canonical = source
        .rows()
        .iter()
        .map(|row| row.coordinate)
        .collect::<Vec<_>>();
    canonical.sort();
    assert_eq!(
        &calls.coordinates.lock().unwrap()[..6],
        canonical.as_slice()
    );
    for probe in [
        broad,
        Coordinates::new(2, 2, 2, 2),
        Coordinates::new(8, 8, 8, 8),
        Coordinates::new(-5, -4, -1, 0),
        Coordinates::new(0, 0, 0, 0),
        Coordinates::new(100, 100, 101, 101),
    ] {
        let actual = request.query(input("boxes", probe)).execute().unwrap();
        assert_eq!(actual.rows(), expected(source.rows(), probe));
    }
    let touching = request
        .query(input("boxes", Coordinates::new(2, 2, 2, 2)))
        .execute()
        .unwrap();
    assert_eq!(
        touching
            .rows()
            .iter()
            .map(|row| row.identity.as_str())
            .collect::<Vec<_>>(),
        ["box-a", "box-b", "box-c"]
    );
    let navigation = calls.navigation.load(Ordering::SeqCst);
    bounds_denial(
        request
            .query(input("boxes", Coordinates::new(2, 0, 1, 1)))
            .execute()
            .err()
            .expect("raw but well-encoded invalid interval"),
    );
    assert_eq!(
        calls.navigation.load(Ordering::SeqCst),
        navigation,
        "numerical admission precedes node navigation"
    );
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    let before = calls.totals();
    let sibling = request
        .query(input("boxes", Coordinates::new(2, 2, 2, 2)))
        .open_index::<BoxIndex>()
        .unwrap();
    assert_eq!(sibling.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    assert_eq!(calls.totals(), before);
    sibling.evict().unwrap();
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Evicted);
    let before = calls.totals();
    not_ready(
        request
            .query(input("boxes", broad))
            .execute()
            .err()
            .expect("Evicted refuses"),
    );
    assert_eq!(calls.totals(), before);
    request
        .query(input("boxes", broad))
        .rebuild_index(&index)
        .unwrap();
    assert_eq!(
        request
            .query(input("boxes", broad))
            .execute()
            .unwrap()
            .rows(),
        expected(source.rows(), broad)
    );
    sibling.close();
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Disposed);
    let fresh = request
        .query(input("boxes", broad))
        .open_index::<BoxIndex>()
        .unwrap();
    assert_eq!(fresh.lifecycle(), WorthQueryManagedIndexLifecycle::Created);
    fresh.close();
    let empty = request
        .query(input("empty-boxes", broad))
        .open_index::<BoxIndex>()
        .unwrap();
    request
        .query(input("empty-boxes", broad))
        .rebuild_index(&empty)
        .unwrap();
    assert!(request
        .query(input("empty-boxes", broad))
        .execute()
        .unwrap()
        .rows()
        .is_empty());
    empty.close();
    let invalid = request
        .query(input("invalid-boxes", broad))
        .open_index::<BoxIndex>()
        .unwrap();
    bounds_denial(
        request
            .query(input("invalid-boxes", broad))
            .rebuild_index(&invalid)
            .expect_err("invalid Source interval"),
    );
    assert_eq!(invalid.lifecycle(), WorthQueryManagedIndexLifecycle::Failed);
    invalid.close();
}
