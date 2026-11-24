//! End-to-end Decl+Host courts for cold Created lifecycle and bounded native/binary watches.
use super::queries::*;
use super::schema::*;
use super::{install, limits, names, rename, seed_graph, Program};
use worth_execution::{CancellationToken, ExecutionRequest, ExecutionResourceLease, LeaseRequest};
use worth_foundational::facade::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_decl::facade::{application_program::*, application_schema::*};
use worth_query_host::facade::application_contribution::with_bootstrap_advancement_from_request;
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::application_installation::*;
use worth_query_host::facade::primary_graph::*;
use worth_query_host::facade::runtime::MapKernelStop;
use worth_query_host::facade::{
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

fn home_lease() -> ExecutionResourceLease<'static> {
    ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(128 * 1024 * 1024), None),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    })
    .unwrap()
}

fn open_directory_home(
    location: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
    initialize: bool,
) -> WorthQueryApplicationHome<CatalogSchema, Program> {
    let declaration = CatalogSchema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<CatalogSchema, Program>::begin()
        .with_schema(&declaration)
        .validated_program()
        .unwrap();
    let request = WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location.to_path_buf()),
        program,
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        ((),),
        ApplicationBuildIdentity::parse("6a6a6a6a-6a6a-6a6a-6a6a-6a6a6a6a6a6a").unwrap(),
        limits(None),
        move |_, graph, installed| {
            assert!(initialize, "reopened Home must not replay initializer");
            seed_graph(graph, installed)
        },
    );
    with_bootstrap_advancement_from_request(*execution, |phase| request.open_in_advancement(&phase))
        .unwrap()
        .unwrap_or_else(|refused| panic!("directory Home open failed: {refused:?}"))
}

#[test]
fn cold_reopened_home_opens_created_and_rebuilds_native_and_binary_probes() {
    let location = std::env::temp_dir().join(format!(
        "worth-managed-probe-cold-home-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let lease = home_lease();
    let execution = ExecutionRequest::leased(&lease);
    let first = open_directory_home(&location, &execution, true);
    first
        .close(&execution)
        .unwrap_or_else(|refused| panic!("first directory Home close failed: {refused:?}"))
        .reclaim();

    let recovered = open_directory_home(&location, &execution, false);
    let scope = request_scope();
    let principal = authenticate_operator(recovered.runtime().installed_schema(), &scope);
    let request = recovered.runtime().runtime().request(&principal, &scope);
    let native_q = || FindNative {
        directory: "catalog".into(),
        prefix: "al".into(),
    };
    let binary_q = || FindBinary {
        directory: "catalog".into(),
        tag: TagBytes(*b"al"),
    };
    let native_index = request
        .query(native_q())
        .open_index::<NativePrefixIndex>()
        .expect("cold readmitted Home opens native probe index in Created lifecycle");
    let binary_index = request
        .query(binary_q())
        .open_index::<BinaryTagIndex>()
        .expect("cold readmitted Home opens binary probe index in Created lifecycle");
    assert_eq!(
        native_index.lifecycle(),
        WorthQueryManagedIndexLifecycle::Created
    );
    assert_eq!(
        binary_index.lifecycle(),
        WorthQueryManagedIndexLifecycle::Created
    );
    match request.query(native_q()).execute() {
        Err(WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause))
            if matches!(
                cause.cause(),
                WorthQueryManagedIndexDenialCause::Refused(
                    WorthQueryManagedIndexRefusal::NotReady
                        | WorthQueryManagedIndexRefusal::RootUnavailable
                )
            ) => {}
        Err(other) => panic!("unexpected cold native execute denial: {other:?}"),
        Ok(_) => panic!("cold native index must not execute before explicit rebuild"),
    }
    match request.query(binary_q()).execute() {
        Err(WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause))
            if matches!(
                cause.cause(),
                WorthQueryManagedIndexDenialCause::Refused(
                    WorthQueryManagedIndexRefusal::NotReady
                        | WorthQueryManagedIndexRefusal::RootUnavailable
                )
            ) => {}
        Err(other) => panic!("unexpected cold binary execute denial: {other:?}"),
        Ok(_) => panic!("cold binary index must not execute before explicit rebuild"),
    }
    request
        .query(native_q())
        .rebuild_index(&native_index)
        .expect("explicit cold rebuild refills native index directly on restored head");
    request
        .query(binary_q())
        .rebuild_index(&binary_index)
        .expect("explicit cold rebuild refills binary index directly on restored head");
    assert_eq!(
        native_index.lifecycle(),
        WorthQueryManagedIndexLifecycle::Ready
    );
    assert_eq!(
        binary_index.lifecycle(),
        WorthQueryManagedIndexLifecycle::Ready
    );
    assert_eq!(
        names(request.query(native_q()).execute().unwrap().rows()),
        ["alpha", "alpine"]
    );
    assert_eq!(
        names(request.query(binary_q()).execute().unwrap().rows()),
        ["alpha", "alpine"]
    );
    native_index.close();
    binary_index.close();
    recovered
        .close(&execution)
        .unwrap_or_else(|refused| panic!("reopened directory Home close failed: {refused:?}"))
        .reclaim();
    std::fs::remove_dir_all(location).unwrap();
}

#[test]
fn native_and_binary_watches_deliver_repeatedly_and_obey_bounded_work_ceilings() {
    let host = install(None);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let native_q = || FindNative {
        directory: "catalog".into(),
        prefix: "al".into(),
    };
    let binary_q = || FindBinary {
        directory: "catalog".into(),
        tag: TagBytes(*b"al"),
    };
    let native_index = request
        .query(native_q())
        .open_index::<NativePrefixIndex>()
        .unwrap();
    let binary_index = request
        .query(binary_q())
        .open_index::<BinaryTagIndex>()
        .unwrap();
    request
        .query(native_q())
        .rebuild_index(&native_index)
        .unwrap();
    request
        .query(binary_q())
        .rebuild_index(&binary_index)
        .unwrap();

    let (initial_native, mut native_watch) = request
        .query(native_q())
        .watch_index::<NativePrefixIndex>()
        .unwrap();
    let (initial_binary, mut binary_watch) = request
        .query(binary_q())
        .watch_index::<BinaryTagIndex>()
        .unwrap();
    assert_eq!(names(initial_native.rows()), ["alpha", "alpine"]);
    assert_eq!(names(initial_binary.rows()), ["alpha", "alpine"]);

    // First and repeat mutations both deliver through retained native and binary watches.
    rename(&host, 3, "alder", 1);
    let WorthQueryManagedIndexWatchOutcome::Delivered(n1) = request
        .query(native_q())
        .next_watch(&mut native_watch)
        .unwrap()
    else {
        panic!("native watch must deliver first update");
    };
    let WorthQueryManagedIndexWatchOutcome::Delivered(b1) = request
        .query(binary_q())
        .next_watch(&mut binary_watch)
        .unwrap()
    else {
        panic!("binary watch must deliver first update");
    };
    assert_eq!(names(n1.read().rows()), ["alder", "alpha", "alpine"]);
    assert_eq!(names(b1.read().rows()), ["alder", "alpha", "alpine"]);

    rename(&host, 4, "aloe", 2);
    let WorthQueryManagedIndexWatchOutcome::Delivered(n2) = request
        .query(native_q())
        .next_watch(&mut native_watch)
        .unwrap()
    else {
        panic!("native watch must deliver repeat update");
    };
    let WorthQueryManagedIndexWatchOutcome::Delivered(b2) = request
        .query(binary_q())
        .next_watch(&mut binary_watch)
        .unwrap()
    else {
        panic!("binary watch must deliver repeat update");
    };
    assert_eq!(
        names(n2.read().rows()),
        ["alder", "aloe", "alpha", "alpine"]
    );
    assert_eq!(
        names(b2.read().rows()),
        ["alder", "aloe", "alpha", "alpine"]
    );
    native_watch.close().unwrap();
    binary_watch.close().unwrap();
    native_index.close();
    binary_index.close();

    // Bounded work ceiling: under a 500_000-unit ceiling, short native probes
    // and their repeat watch deliveries succeed, while multiple retained large
    // variable-length native String watches exhaust the work ceiling during
    // watch routing because each probe decode charges its per-byte profile.
    let bounded = install(Some(500_000));
    let scope = request_scope();
    let principal = authenticate_operator(bounded.installed_schema(), &scope);
    let request = bounded.runtime().request(&principal, &scope);
    let idx = request
        .query(native_q())
        .open_index::<NativePrefixIndex>()
        .unwrap();
    request.query(native_q()).rebuild_index(&idx).unwrap();
    assert_eq!(
        names(request.query(native_q()).execute().unwrap().rows()),
        ["alpha", "alpine"]
    );
    let (_, mut bounded_watch) = request
        .query(native_q())
        .watch_index::<NativePrefixIndex>()
        .unwrap();
    rename(&bounded, 3, "alto", 10);
    let WorthQueryManagedIndexWatchOutcome::Delivered(bw1) = request
        .query(native_q())
        .next_watch(&mut bounded_watch)
        .unwrap()
    else {
        panic!("bounded native watch must deliver under sufficient work ceiling");
    };
    assert_eq!(names(bw1.read().rows()), ["alpha", "alpine", "alto"]);
    rename(&bounded, 4, "ally", 11);
    let WorthQueryManagedIndexWatchOutcome::Delivered(bw2) = request
        .query(native_q())
        .next_watch(&mut bounded_watch)
        .unwrap()
    else {
        panic!("bounded native watch must deliver repeat update under sufficient work ceiling");
    };
    assert_eq!(
        names(bw2.read().rows()),
        ["ally", "alpha", "alpine", "alto"]
    );
    bounded_watch.close().unwrap();

    // Register watches with distinct 50 KiB native String probes (each under
    // the 64 KiB parameter byte limit). Inserting/comparing retained native
    // probes in the predicate tree charges each probe's per-byte native profile
    // and deterministically stops with ValueDecode(Checkpoint(WorkCeiling))
    // under the 500_000-unit ceiling instead of charging 1 unit per probe.
    let mut heavy_watches = Vec::new();
    let mut hit_work_ceiling = false;
    for ch in ['m', 'n', 'o', 'p', 'q', 'r', 's', 't'] {
        let query = FindNative {
            directory: "catalog".into(),
            prefix: std::iter::repeat_n(ch, 50_000).collect(),
        };
        match request.query(query).watch_index::<NativePrefixIndex>() {
            Ok((_, watch)) => heavy_watches.push(watch),
            Err(WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause))
                if matches!(
                    cause.cause(),
                    WorthQueryManagedIndexDenialCause::Work(
                        WorthQueryManagedIndexWorkDenial::Controlled(
                            WorthQueryManagedIndexStop::ValueDecode(
                                WorthQueryValueDecodeDenial::Checkpoint(MapKernelStop::WorkCeiling)
                            )
                        )
                    )
                ) =>
            {
                hit_work_ceiling = true;
                break;
            }
            Err(other) => panic!("unexpected denial while opening heavy native watches: {other:?}"),
        }
    }
    assert!(
        !heavy_watches.is_empty(),
        "initial 50 KiB native String watch must succeed under 500_000 work ceiling"
    );
    assert!(
        hit_work_ceiling,
        "retained 50 KiB native String probes must hit ValueDecode(Checkpoint(WorkCeiling))"
    );
    for watch in heavy_watches {
        watch.close().unwrap();
    }
    idx.close();
}
