//! A real `v2 -> v3` Directory Home transition whose successor declares a
//! stock managed index: the target step commits, the index readies over the
//! retained records, and the indexed query answers exactly across reopen.

use std::path::Path;

use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    application_entry::WorthQueryApplicationRequestExt,
    application_installation::ApplicationBuildIdentity,
    primary_graph::WorthQueryManagedIndexLifecycle,
    program_transition::{
        open_directory_with_transition, worth_query_default_build_id,
        WorthQueryProgramTransitionOpenConfig, WorthQueryTransitionOpenOutcome,
    },
    WorthQueryApplicationHome,
};
use worth_query_testkit_host::facade::host::{
    application_home as fixture, application_runtime_surface as identity, checkpoint_capture,
};

use super::driver::request_scope;
use super::journey::seed_v2_tasks;
use super::v2_program::v2_request;
use super::v3_indexed_program::{
    v3_indexed_request, DaymarkV2ToV3Indexed, DaymarkV3IndexedProgram, DaymarkV3IndexedSchema,
    FindTasksByValuePrefix, ValuePrefixIndex,
};

pub(super) type IndexedHome =
    WorthQueryApplicationHome<DaymarkV3IndexedSchema, DaymarkV3IndexedProgram>;

pub(super) fn open_indexed(
    location: &Path,
    build: ApplicationBuildIdentity,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> (IndexedHome, WorthQueryTransitionOpenOutcome) {
    let opened = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            open_directory_with_transition::<DaymarkV2ToV3Indexed>(
                &phase,
                v2_request(location.to_path_buf(), build),
                v3_indexed_request(location.to_path_buf(), build),
                WorthQueryProgramTransitionOpenConfig::default_bounded(),
            )
            .expect("v2 -> v3 with a successor managed index opens"),
        )
    })
    .unwrap();
    (*opened).into_parts()
}

/// Readies the prefix index, then returns `(task_key, v2_value, priority)` of
/// every task whose `V2Value` starts with `prefix` (ASCII case-insensitive).
fn indexed_prefix_rows(home: &IndexedHome, prefix: &str) -> Vec<(String, String, String)> {
    let scope = request_scope();
    let runtime = home.runtime();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        &scope,
    );
    let request = runtime.request(&principal, &scope);
    let query = || FindTasksByValuePrefix {
        project: "p-main".to_owned(),
        prefix: prefix.to_owned(),
    };
    let index = request
        .query(query())
        .ensure_index_ready::<ValuePrefixIndex>()
        .expect("successor managed index readies over retained records");
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    let rows = request
        .query(query())
        .execute()
        .expect("indexed prefix query executes")
        .rows()
        .iter()
        .map(|row| {
            (
                row.task_key.clone(),
                row.v2_value.clone(),
                row.priority.clone(),
            )
        })
        .collect();
    let _ = index.close();
    rows
}

#[test]
fn transition_to_successor_with_managed_index_commits_and_index_answers_retained_rows() {
    let location = fixture::location();
    let build = worth_query_default_build_id!();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    seed_v2_tasks(
        &location,
        build,
        execution,
        &[
            ("t1", "Mill housing"),
            ("t2", "Drill flange"),
            ("t3", "mill base"),
        ],
    );
    let expected_mill = vec![
        (
            "t1".to_owned(),
            "Mill housing".to_owned(),
            "Normal".to_owned(),
        ),
        ("t3".to_owned(), "mill base".to_owned(), "Normal".to_owned()),
    ];

    let (home, outcome) = open_indexed(&location, build, execution);
    assert!(
        matches!(
            outcome,
            WorthQueryTransitionOpenOutcome::UpgradedInPlace {
                from_version: 2,
                to_version: 3,
                backfilled_records: 3,
                ..
            }
        ),
        "expected UpgradedInPlace(2 -> 3, 3 backfilled), got {outcome:?}"
    );
    assert_eq!(indexed_prefix_rows(&home, "MILL"), expected_mill);
    assert_eq!(
        indexed_prefix_rows(&home, "drill"),
        vec![(
            "t2".to_owned(),
            "Drill flange".to_owned(),
            "Normal".to_owned()
        )]
    );
    assert!(indexed_prefix_rows(&home, "lathe").is_empty());
    home.close(&execution).unwrap().reclaim();

    // The same transition entry reopens the persisted v3 Home idempotently.
    let (reopened, outcome) = open_indexed(&location, build, execution);
    assert_eq!(
        outcome,
        WorthQueryTransitionOpenOutcome::OpenedCurrent { program_version: 3 }
    );
    assert_eq!(indexed_prefix_rows(&reopened, "mill"), expected_mill);
    reopened.close(&execution).unwrap().reclaim();
}
