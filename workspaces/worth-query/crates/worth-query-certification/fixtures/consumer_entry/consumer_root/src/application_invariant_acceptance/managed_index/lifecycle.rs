//! Actual registered cold construction, interruption and final-owner cleanup.
use super::super::authentication;
use super::{host, journey, prefix, query};
use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};
use std::sync::{Arc, Mutex};
use worth_query_host::facade::{
    admission::authenticated_principal::*,
    application_entry::*,
    primary_graph::*,
    runtime::{MapKernelStop, WorkCeilingDenial},
};

#[global_allocator]
pub(super) static ALLOCATOR: &StatsAlloc<std::alloc::System> = &INSTRUMENTED_SYSTEM;

mod affinity;
mod installation;

fn find(prefix: &str) -> query::FindPrefix {
    query::FindPrefix {
        directory: "catalog".into(),
        prefix: prefix.into(),
    }
}
pub(super) fn run(test: fn()) {
    std::thread::Builder::new()
        .name("managed-lifecycle-consumer".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(test)
        .unwrap()
        .join()
        .unwrap();
}
pub(super) fn authenticate(
    world: &host::Host,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<super::schema::Catalog> {
    authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            scope,
        ),
    )
    .unwrap()
}
fn not_ready(error: WorthQueryApplicationRequestQueryDenial) {
    refusal(error, WorthQueryManagedIndexRefusal::NotReady);
}
fn refusal(
    error: WorthQueryApplicationRequestQueryDenial,
    expected: WorthQueryManagedIndexRefusal,
) {
    let WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause) = error else {
        panic!("actual managed receiver refusal: {error:?}")
    };
    assert!(
        matches!(cause.cause(), WorthQueryManagedIndexDenialCause::Refused(actual) if actual == expected),
        "{cause:?}"
    );
}

#[test]
fn cancelled_cold_prefix_build_releases_partial_backing_and_rebuilds_with_fresh_request() {
    isolated_allocation_case(cancelled_build, concat!(module_path!(), "::cancelled_cold_prefix_build_releases_partial_backing_and_rebuilds_with_fresh_request"));
}
fn cancelled_build() {
    let calls = Arc::new(prefix::Calls::default());
    let world = host::install(Arc::clone(&calls));
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token());
    let principal = authenticate(&world, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let index = request
        .query(find("al"))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    // Prime genuine ordinary Source reads, so the measured build does not own
    // unrelated first-read Native cache population.
    let source = request
        .query(query::ReadSource {
            directory: "catalog".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(source.rows().len(), 7);
    let reached = Arc::new(Mutex::new(None));
    let observation = Arc::clone(&reached);
    let region = Region::new(ALLOCATOR);
    *calls.after_first_node.lock().unwrap() = Some(Box::new(move || {
        // This runs after NodeWriter::push succeeds, not at admission or capacity.
        *observation.lock().unwrap() = Some(ALLOCATOR.stats());
        cancellation.cancel();
    }));
    let error = request.query(find("al")).rebuild_index(&index).unwrap_err();
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
        "original cold-build cancellation: {error:?}"
    );
    assert!(
        reached.lock().unwrap().is_some(),
        "construction wrote a real node before cancellation"
    );
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Failed);
    assert_eq!(calls.totals(), (1, 1, 0));
    drop(error);
    let released = region.change();
    assert_eq!(
        released.bytes_allocated, released.bytes_deallocated,
        "stopped build releases its partial Source, node scratch and diagnostic owners"
    );
    let fresh_scope = authentication::request_scope();
    let fresh = world
        .runtime()
        .request(&principal, &fresh_scope)
        .on_branch(world.current_world());
    not_ready(
        fresh
            .query(find("al"))
            .execute()
            .err()
            .expect("managed read must refuse"),
    );
    assert_eq!(calls.totals(), (1, 1, 0), "no Ready image or silent retry");
    fresh.query(find("al")).rebuild_index(&index).unwrap();
    let actual = fresh.query(find("al")).execute().unwrap();
    assert_eq!(actual.rows(), journey::expected(source.rows(), "al"));
    assert_eq!(calls.totals(), (2, 2, 1));
    index.close();
}

#[test]
fn registered_prefix_backing_releases_after_results_siblings_and_runtime_owners_close() {
    isolated_allocation_case(
        final_release,
        concat!(
            module_path!(),
            "::registered_prefix_backing_releases_after_results_siblings_and_runtime_owners_close"
        ),
    );
}
fn final_release() {
    warm_allocation_observation();
    // Observe physical allocation, not a reservation estimate or a Drop flag.
    // Exact acquisition runs this case alone with --test-threads=1.
    let region = Region::new(ALLOCATOR);
    let calls = Arc::new(prefix::Calls::default());
    let world = host::install(Arc::clone(&calls));
    let keepalive = world.clone();
    let scope = authentication::request_scope();
    let principal = authenticate(&world, &scope);
    let expected = {
        let request = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(world.current_world());
        let source = request
            .query(query::ReadSource {
                directory: "catalog".into(),
            })
            .execute()
            .unwrap();
        let expected = journey::expected(source.rows(), "al");
        let index = request
            .query(find("al"))
            .open_index::<prefix::PrefixIndex>()
            .unwrap();
        request.query(find("al")).rebuild_index(&index).unwrap();
        let sibling = request
            .query(find("al"))
            .open_index::<prefix::PrefixIndex>()
            .unwrap();
        let result = request.query(find("al")).execute().unwrap();
        assert_eq!(result.rows(), expected);
        // No close/evict: release EVERY application index and Source/result owner.
        // Only plain projected values leave this block; the request borrow ends.
        drop((index, sibling, result, source));
        expected
    };
    let after_last_handle = calls.totals();
    assert_eq!(after_last_handle, (1, 1, 1));
    let (index, sibling, result) = {
        let request = keepalive
            .runtime()
            .request(&principal, &scope)
            .on_branch(keepalive.current_world());
        let index = request
            .query(find("al"))
            .open_index::<prefix::PrefixIndex>()
            .unwrap();
        assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
        let sibling = request
            .query(find("al"))
            .open_index::<prefix::PrefixIndex>()
            .unwrap();
        assert_eq!(sibling.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
        assert_eq!(
            calls.totals(),
            after_last_handle,
            "reopening after the LAST handle dropped reuses the runtime cache without a build"
        );
        let result = request.query(find("al")).execute().unwrap();
        assert_eq!(
            result.rows(),
            expected,
            "reopened Ready root preserves independent scan meaning"
        );
        assert_eq!(
            calls.totals(),
            (
                after_last_handle.0,
                after_last_handle.1,
                after_last_handle.2 + 1
            )
        );
        (index, sibling, result)
    };
    drop(world);
    // This clone is now the only Program runtime owner. The outstanding result
    // still owns its rows/Source backing, which explicit cache disposal may not erase.
    let before_close = ALLOCATOR.stats();
    sibling.close();
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Disposed);
    assert_eq!(
        sibling.lifecycle(),
        WorthQueryManagedIndexLifecycle::Disposed
    );
    assert!(
        ALLOCATOR.stats().bytes_deallocated > before_close.bytes_deallocated,
        "closing the shared registration actually releases backing"
    );
    assert_eq!(
        result.rows(),
        expected,
        "owned rows survive cache disposal without granting a new read"
    );
    drop((result, index, sibling));

    let fresh = {
        let request = keepalive
            .runtime()
            .request(&principal, &scope)
            .on_branch(keepalive.current_world());
        let fresh = request
            .query(find("al"))
            .open_index::<prefix::PrefixIndex>()
            .unwrap();
        assert_eq!(fresh.lifecycle(), WorthQueryManagedIndexLifecycle::Created);
        request.query(find("al")).rebuild_index(&fresh).unwrap();
        let result = request.query(find("al")).execute().unwrap();
        assert_eq!(result.rows(), expected);
        drop(result);
        fresh
    };
    // No request, query result, old registration handle or other Program clone
    // remains. The principal/scope are value/control custody, but release them too.
    // `fresh` owns registered state and only a WEAK registry link, not the runtime;
    // its Ready Source retains a Product observation, not a Query runtime owner.
    drop((principal, scope, expected));
    assert_eq!(fresh.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    let before_runtime_drop = ALLOCATOR.stats();
    let before_disposal = calls.totals();
    drop(keepalive);
    // No explicit close/evict: the last runtime's registry destruction must dispose
    // this STILL-READY registration while its application state handle survives.
    assert_eq!(fresh.lifecycle(), WorthQueryManagedIndexLifecycle::Disposed);
    assert_eq!(
        calls.totals(),
        before_disposal,
        "runtime disposal invokes no provider callback"
    );
    assert!(
        ALLOCATOR.stats().bytes_deallocated > before_runtime_drop.bytes_deallocated,
        "final runtime disposal physically releases its Ready cache backing"
    );
    drop((fresh, calls));
    let released = region.change();
    assert_eq!(
        released.bytes_allocated, released.bytes_deallocated,
        "no registered Source/node/installation backing remains after the final owners close"
    );
}

fn warm_allocation_observation() {
    // Execution/codec thread-local stacks retain their reusable capacity until
    // thread exit. Initialize those existing stacks before measuring World
    // ownership, rather than counting their first-use capacity as an index leak.
    let world = host::install(Arc::new(prefix::Calls::default()));
    let scope = authentication::request_scope();
    let principal = authenticate(&world, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let index = request
        .query(find("al"))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    request.query(find("al")).rebuild_index(&index).unwrap();
    drop(request.query(find("al")).execute().unwrap());
    index.close();
}

pub(super) fn isolated_allocation_case(test: fn(), rust_path: &str) {
    // Follow the existing owner allocation court: another concurrent test's
    // allocator traffic cannot establish either a leak or a successful release.
    // This guard selects only the harness invocation, never an authority mode.
    let name = rust_path.split_once("::").unwrap().1;
    const SELECTOR: &str = "WORTH_MANAGED_INDEX_ALLOCATION_CASE";
    if std::env::var(SELECTOR).ok().as_deref() == Some(name) {
        run(test);
        return;
    }
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .arg(name)
        .args(["--exact", "--nocapture", "--test-threads=1"])
        .env(SELECTOR, name)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success() && stdout.contains("1 passed"),
        "isolated actual registered journey failed:\n{stdout}\n{stderr}"
    );
    print!("{stdout}");
}
