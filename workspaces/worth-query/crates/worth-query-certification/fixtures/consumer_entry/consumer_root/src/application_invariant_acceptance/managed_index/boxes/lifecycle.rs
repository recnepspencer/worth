//! BVH-specific stopped construction and physical backing ownership.
use super::super::{host, lifecycle, prefix};
use super::{geometry, query, BoxIndex, Calls, Coordinates};
use crate::application_invariant_acceptance::authentication;
use lifecycle::{authenticate, isolated_allocation_case, ALLOCATOR};
use stats_alloc::Region;
use std::sync::{Arc, Mutex};
use worth_query_host::facade::{
    admission::authenticated_principal::*,
    application_contribution::*,
    application_entry::*,
    primary_graph::*,
    runtime::{MapKernelStop, WorkCeilingDenial},
};

fn probe() -> Coordinates {
    Coordinates::new(2, 2, 2, 2)
}
fn find() -> query::FindBoxes {
    geometry::input("boxes", probe())
}

#[test]
fn cancelled_cold_box_build_releases_partial_bvh_and_rebuilds_with_fresh_request() {
    isolated_allocation_case(
        cancelled_build,
        concat!(
            module_path!(),
            "::cancelled_cold_box_build_releases_partial_bvh_and_rebuilds_with_fresh_request"
        ),
    );
}
fn cancelled_build() {
    let calls = Arc::new(Calls::default());
    let world = host::install_boxes(Arc::new(prefix::Calls::default()), Arc::clone(&calls));
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token());
    let principal = authenticate(&world, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let index = request.query(find()).open_index::<BoxIndex>().unwrap();
    // Prime the real ordinary Source read before observing partial BVH ownership.
    let source = request
        .query(query::ReadSource {
            directory: "boxes".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(source.rows().len(), 6);
    let reached = Arc::new(Mutex::new(None));
    let observation = Arc::clone(&reached);
    let region = Region::new(ALLOCATOR);
    *calls.after_first_node.lock().unwrap() = Some(Box::new(move || {
        // A leaf was actually pushed. The remaining recursive BVH spans must stop.
        *observation.lock().unwrap() = Some(ALLOCATOR.stats());
        cancellation.cancel();
    }));
    let error = request.query(find()).rebuild_index(&index).unwrap_err();
    assert!(
        matches!(
            error,
            WorthQueryApplicationRequestQueryDenial::ManagedIndex(ref cause)
                if matches!(cause.cause(), WorthQueryManagedIndexDenialCause::Work(
                    WorthQueryManagedIndexWorkDenial::Controlled(
                        WorthQueryManagedIndexStop::Request(
                            WorkCeilingDenial::Stopped(MapKernelStop::Cancelled)
                        )
                    )
                ))
        ),
        "original BVH construction request stopped: {error:?}"
    );
    assert!(
        reached.lock().unwrap().is_some(),
        "a real BVH leaf preceded cancellation"
    );
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Failed);
    assert_eq!(calls.totals(), (1, 1, 0));
    drop(error);
    let released = region.change();
    assert_eq!(
        released.bytes_allocated, released.bytes_deallocated,
        "cancelled BVH releases actual partial Source/node/diagnostic backing"
    );
    let fresh_scope = authentication::request_scope();
    let fresh = world
        .runtime()
        .request(&principal, &fresh_scope)
        .on_branch(world.current_world());
    geometry::not_ready(
        fresh
            .query(find())
            .execute()
            .err()
            .expect("managed read must refuse"),
    );
    assert_eq!(
        calls.totals(),
        (1, 1, 0),
        "no Ready publication or silent reconstruction"
    );
    fresh.query(find()).rebuild_index(&index).unwrap();
    let actual = fresh.query(find()).execute().unwrap();
    assert_eq!(actual.rows(), geometry::expected(source.rows(), probe()));
    assert_eq!(
        actual
            .rows()
            .iter()
            .map(|row| row.identity.as_str())
            .collect::<Vec<_>>(),
        ["box-a", "box-b", "box-c"]
    );
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    assert_eq!(calls.totals(), (2, 2, 1));
    index.close();
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Disposed);
}

#[test]
fn registered_box_backing_releases_after_results_and_last_runtime_close() {
    isolated_allocation_case(
        final_release,
        concat!(
            module_path!(),
            "::registered_box_backing_releases_after_results_and_last_runtime_close"
        ),
    );
}
fn final_release() {
    warm_allocation_observation();
    let region = Region::new(ALLOCATOR);
    let calls = Arc::new(Calls::default());
    let world = host::install_boxes(Arc::new(prefix::Calls::default()), Arc::clone(&calls));
    let scope = authentication::request_scope();
    let principal = authenticate(&world, &scope);
    let (index, sibling, rows, expected) = {
        let request = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(world.current_world());
        let source = request
            .query(query::ReadSource {
                directory: "boxes".into(),
            })
            .execute()
            .unwrap();
        let expected = geometry::expected(source.rows(), probe());
        assert_eq!(source.rows().len(), 6);
        let index = request.query(find()).open_index::<BoxIndex>().unwrap();
        request.query(find()).rebuild_index(&index).unwrap();
        let sibling = request.query(find()).open_index::<BoxIndex>().unwrap();
        let rows = request.query(find()).execute().unwrap();
        assert_eq!(rows.rows(), expected);
        // Source and request leave scope; only these real retained owners escape.
        (index, sibling, rows, expected)
    };
    assert_eq!(calls.totals(), (1, 1, 1));
    let before_close = ALLOCATOR.stats();
    sibling.close();
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Disposed);
    assert_eq!(
        sibling.lifecycle(),
        WorthQueryManagedIndexLifecycle::Disposed
    );
    assert!(
        ALLOCATOR.stats().bytes_deallocated > before_close.bytes_deallocated,
        "shared close physically releases BVH backing"
    );
    assert_eq!(
        rows.rows(),
        expected,
        "owned box rows survive registration disposal"
    );
    drop((rows, index, sibling));
    let fresh = {
        let request = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(world.current_world());
        let fresh = request.query(find()).open_index::<BoxIndex>().unwrap();
        assert_eq!(fresh.lifecycle(), WorthQueryManagedIndexLifecycle::Created);
        request.query(find()).rebuild_index(&fresh).unwrap();
        let rows = request.query(find()).execute().unwrap();
        assert_eq!(rows.rows(), expected);
        drop(rows);
        fresh
    };
    // There is exactly one Program/runtime owner: no clone or request escapes.
    // The index owns state plus a weak registry link, not the runtime itself.
    drop((principal, scope, expected));
    assert_eq!(fresh.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    let before_runtime_drop = ALLOCATOR.stats();
    let before_disposal = calls.totals();
    drop(world);
    assert_eq!(fresh.lifecycle(), WorthQueryManagedIndexLifecycle::Disposed);
    assert_eq!(
        calls.totals(),
        before_disposal,
        "runtime destruction invokes no provider callback"
    );
    assert!(
        ALLOCATOR.stats().bytes_deallocated > before_runtime_drop.bytes_deallocated,
        "last runtime releases still-Ready BVH backing without explicit close"
    );
    drop((fresh, calls));
    let released = region.change();
    assert_eq!(
        released.bytes_allocated, released.bytes_deallocated,
        "all actual box Source/node/runtime owners have released their backing"
    );
}
fn warm_allocation_observation() {
    // Reuse the existing thread-local Execution/codec capacities before measuring.
    let world = host::install_boxes(
        Arc::new(prefix::Calls::default()),
        Arc::new(Calls::default()),
    );
    let scope = authentication::request_scope();
    let principal = authenticate(&world, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let index = request.query(find()).open_index::<BoxIndex>().unwrap();
    request.query(find()).rebuild_index(&index).unwrap();
    drop(request.query(find()).execute().unwrap());
    index.close();
}
