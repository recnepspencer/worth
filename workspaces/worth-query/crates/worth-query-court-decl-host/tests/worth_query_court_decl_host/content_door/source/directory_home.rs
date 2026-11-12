//! The receiving compiler and real filesystem Home share nonempty Source issuers.
use super::*;
use std::sync::atomic::Ordering;
use worth_execution::{CancellationToken, ExecutionRequest, ExecutionResourceLease, LeaseRequest};
use worth_foundational::facade::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::{
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};

type Home = WorthQueryApplicationHome<Schema, world::Program>;
fn open(
    location: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
    calls: Arc<Calls>,
    initialize: bool,
) -> Home {
    let declaration = Schema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<Schema, world::Program>::begin()
        .with_schema(&declaration)
        .validated_program()
        .unwrap();
    let request = WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location.to_path_buf()),
        program,
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        ((None, calls),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host_limits(WorthQueryAdmissionClock::system()),
        move |_, graph, installed| {
            assert!(initialize, "existing Home must not replay its initializer");
            world::seed(graph, installed)
        },
    );
    with_bootstrap_advancement_from_request(*execution, |phase| request.open_in_advancement(&phase))
        .unwrap()
        .unwrap_or_else(|refused| panic!("actual content Source Home: {refused:?}"))
}
fn observe_and_rebuild(host: &world::Host, expected: &[&str]) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let source = request
        .query(query::ReadSource {
            root: "record".into(),
        })
        .execute()
        .unwrap();
    let keys = source
        .rows()
        .iter()
        .map(|row| row.key.as_str())
        .collect::<Vec<_>>();
    assert_eq!(keys, ["alpha"]);
    drop(source);
    let find = || query::Find {
        root: "record".into(),
        minimum: 10,
    };
    let index = request.query(find()).open_index::<Index>().unwrap();
    request.query(find()).rebuild_index(&index).unwrap();
    let exact = request.query(find()).execute().unwrap();
    let keys = exact
        .rows()
        .iter()
        .map(|row| row.key.as_str())
        .collect::<Vec<_>>();
    assert_eq!(keys, expected);
    drop(exact);
    index.close();
}

fn observe_cold_source(host: &world::Host) {
    let selected = world::pin(host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let rows = request
        .query(query::ReadSource {
            root: "record".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        rows.rows()
            .iter()
            .map(|row| row.key.as_str())
            .collect::<Vec<_>>(),
        ["alpha"]
    );
    let refused = match request
        .query(query::Find {
            root: "record".into(),
            minimum: 10,
        })
        .open_index::<Index>()
    {
        Err(cause) => cause,
        Ok(_) => panic!("reopen must not eagerly recreate a live managed Source image"),
    };
    let WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause) = refused else {
        panic!("{refused:?}")
    };
    assert!(matches!(
        cause.cause(),
        WorthQueryManagedIndexDenialCause::Refused(WorthQueryManagedIndexRefusal::RootUnavailable)
    ));
    assert_eq!(
        world::pin(host).selected_commit(),
        selected.selected_commit()
    );
}

#[test]
fn a_nonempty_declared_source_reads_and_mutates_after_directory_home_reopen() {
    let location = std::env::temp_dir().join(format!(
        "worth-original-source-home-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let lease = lease();
    let execution = ExecutionRequest::leased(&lease);
    let first = Arc::new(Calls::default());
    let home = open(&location, &execution, first.clone(), true);
    assert!(matches!(
        world::edit(
            home.runtime(),
            18,
            vec![7, 11],
            b"alpha".to_vec(),
            "original-source-alpha"
        ),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    observe_and_rebuild(home.runtime(), &["alpha"]);
    assert!(!first.records.lock().unwrap().is_empty());
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("original Source close: {refused:?}"));

    let receiving = Arc::new(Calls::default());
    let home = open(&location, &execution, receiving.clone(), false);
    // No Native rule, provider refinement or initializer runs during reopen.
    assert!(receiving.records.lock().unwrap().is_empty());
    assert_eq!(receiving.content_reads.load(Ordering::SeqCst), 0);
    assert_eq!(receiving.exact_reads.load(Ordering::SeqCst), 0);
    assert_eq!(receiving.updates.load(Ordering::SeqCst), 0);
    observe_cold_source(home.runtime());
    receiving.records.lock().unwrap().clear();
    assert!(matches!(
        world::edit(
            home.runtime(),
            19,
            vec![1, 2],
            b"alpha".to_vec(),
            "receiving-source-alpha"
        ),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert!(
        receiving
            .records
            .lock()
            .unwrap()
            .iter()
            .any(
                |(before, after)| before.as_ref() == Some(&vec!["alpha".to_owned()])
                    && after.is_empty()
            ),
        "the actual installed Source must read the original and proposed content rows"
    );
    observe_and_rebuild(home.runtime(), &[]);
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("receiving Source close: {refused:?}"));

    let last = Arc::new(Calls::default());
    let home = open(&location, &execution, last, false);
    observe_cold_source(home.runtime());
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("second Source reopen close: {refused:?}"));
    std::fs::remove_dir_all(location).unwrap();
}

mod compensation_refusal;
fn lease() -> ExecutionResourceLease<'static> {
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
