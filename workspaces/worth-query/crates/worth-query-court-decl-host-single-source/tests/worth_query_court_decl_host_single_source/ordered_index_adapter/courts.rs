//! End-to-end Decl+Host lifecycle and cold-reopen courts for `WorthQueryOrderedIndexAdapter`.
use super::queries::*;
use super::schema::*;
use super::{
    create_task, delete_task, home_lease, install, keys, open_directory_home, rename_task,
    reschedule_task,
};
use worth_execution::ExecutionRequest;
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::primary_graph::*;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

fn prefix_q(prefix: &str) -> FindTitlePrefix {
    FindTitlePrefix {
        workspace: "main".into(),
        prefix: prefix.to_owned(),
    }
}

fn focus_q(start: i64, end: i64) -> FindFocusOverlap {
    FindFocusOverlap {
        workspace: "main".into(),
        window: FocusWindow::new(start, end),
    }
}

fn assert_not_ready<R>(result: Result<R, WorthQueryApplicationRequestQueryDenial>) {
    match result {
        Err(WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause))
            if matches!(
                cause.cause(),
                WorthQueryManagedIndexDenialCause::Refused(
                    WorthQueryManagedIndexRefusal::NotReady
                        | WorthQueryManagedIndexRefusal::RootUnavailable
                )
            ) => {}
        Err(other) => panic!("expected NotReady refusal before rebuild, got Err({other:?})"),
        Ok(_) => panic!("expected NotReady refusal before rebuild, got Ok"),
    }
}

#[test]
fn title_prefix_and_focus_interval_lifecycle_insert_edit_reschedule_delete_and_watches() {
    let seed = [
        (1, "Alpha", 90, 120),
        (2, "alpine", 110, 150),
        (3, "Beta", 200, 260),
        (4, "gamma", 50, 300),
    ];
    let host = install(&seed, None, None);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);

    let title_idx = request
        .query(prefix_q("al"))
        .open_index::<TitlePrefixIndex>()
        .unwrap();
    let focus_idx = request
        .query(focus_q(100, 130))
        .open_index::<FocusIntervalIndex>()
        .unwrap();
    request
        .query(prefix_q("al"))
        .rebuild_index(&title_idx)
        .unwrap();
    request
        .query(focus_q(100, 130))
        .rebuild_index(&focus_idx)
        .unwrap();
    assert_eq!(
        title_idx.lifecycle(),
        WorthQueryManagedIndexLifecycle::Ready
    );
    assert_eq!(
        focus_idx.lifecycle(),
        WorthQueryManagedIndexLifecycle::Ready
    );

    let (init_title, mut title_watch) = request
        .query(prefix_q("AL"))
        .watch_index::<TitlePrefixIndex>()
        .unwrap();
    let (init_focus, mut focus_watch) = request
        .query(focus_q(100, 130))
        .watch_index::<FocusIntervalIndex>()
        .unwrap();
    assert_eq!(keys(init_title.rows()), [1, 2]);
    assert_eq!(keys(init_focus.rows()), [1, 2, 4]);
    assert_eq!(
        keys(request.query(focus_q(270, 290)).execute().unwrap().rows()),
        [4]
    );

    create_task(&host, 5, "Alpha", 125, 140, 1);
    let WorthQueryManagedIndexWatchOutcome::Delivered(tw1) = request
        .query(prefix_q("AL"))
        .next_watch(&mut title_watch)
        .unwrap()
    else {
        panic!("title watch must deliver after insert");
    };
    let WorthQueryManagedIndexWatchOutcome::Delivered(fw1) = request
        .query(focus_q(100, 130))
        .next_watch(&mut focus_watch)
        .unwrap()
    else {
        panic!("focus watch must deliver after insert");
    };
    assert_eq!(keys(tw1.read().rows()), [1, 2, 5]);
    assert_eq!(keys(fw1.read().rows()), [1, 2, 4, 5]);

    rename_task(&host, 3, "Alder", 2);
    let WorthQueryManagedIndexWatchOutcome::Delivered(tw2) = request
        .query(prefix_q("AL"))
        .next_watch(&mut title_watch)
        .unwrap()
    else {
        panic!("title watch must deliver after rename");
    };
    assert_eq!(keys(tw2.read().rows()), [1, 2, 3, 5]);

    reschedule_task(&host, 4, 10, 40, 3);
    let WorthQueryManagedIndexWatchOutcome::Delivered(fw2) = request
        .query(focus_q(100, 130))
        .next_watch(&mut focus_watch)
        .unwrap()
    else {
        panic!("focus watch must deliver after reschedule");
    };
    assert_eq!(keys(fw2.read().rows()), [1, 2, 5]);
    assert!(request
        .query(focus_q(270, 290))
        .execute()
        .unwrap()
        .rows()
        .is_empty());

    for (idem, key) in [(4, 2), (5, 1), (6, 3), (7, 4), (8, 5)] {
        delete_task(&host, key, idem);
    }
    assert!(request
        .query(prefix_q(""))
        .execute()
        .unwrap()
        .rows()
        .is_empty());
    assert!(request
        .query(focus_q(0, 500))
        .execute()
        .unwrap()
        .rows()
        .is_empty());

    create_task(&host, 6, "Aloe", 105, 115, 9);
    assert_eq!(
        keys(request.query(prefix_q("al")).execute().unwrap().rows()),
        [6]
    );
    assert_eq!(
        keys(request.query(focus_q(100, 130)).execute().unwrap().rows()),
        [6]
    );

    title_watch.close().unwrap();
    focus_watch.close().unwrap();
    title_idx.close();
    focus_idx.close();
}

#[test]
fn directory_home_cold_reopen_rebuilds_both_ordered_indexes() {
    let location = std::env::temp_dir().join(format!(
        "worth-ordered-index-home-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let lease = home_lease();
    let execution = ExecutionRequest::leased(&lease);
    let seed = [(1, "Alpha", 10, 50), (2, "Beta", 40, 90)];
    let first = open_directory_home(&location, &execution, true, &seed);
    create_task(first.runtime(), 3, "Alpine", 60, 120, 100);
    reschedule_task(first.runtime(), 1, 80, 110, 101);
    delete_task(first.runtime(), 2, 102);
    first.close(&execution).unwrap().reclaim();

    let reopened = open_directory_home(&location, &execution, false, &[]);
    let scope = request_scope();
    let principal = authenticate_operator(reopened.runtime().installed_schema(), &scope);
    let request = reopened.runtime().runtime().request(&principal, &scope);

    let title_idx = request
        .query(prefix_q("al"))
        .open_index::<TitlePrefixIndex>()
        .unwrap();
    let focus_idx = request
        .query(focus_q(70, 95))
        .open_index::<FocusIntervalIndex>()
        .unwrap();
    assert_eq!(
        title_idx.lifecycle(),
        WorthQueryManagedIndexLifecycle::Created
    );
    assert_eq!(
        focus_idx.lifecycle(),
        WorthQueryManagedIndexLifecycle::Created
    );
    assert_not_ready(request.query(prefix_q("al")).execute());
    assert_not_ready(request.query(focus_q(70, 95)).execute());

    request
        .query(prefix_q("al"))
        .rebuild_index(&title_idx)
        .unwrap();
    request
        .query(focus_q(70, 95))
        .rebuild_index(&focus_idx)
        .unwrap();
    assert_eq!(
        title_idx.lifecycle(),
        WorthQueryManagedIndexLifecycle::Ready
    );
    assert_eq!(
        focus_idx.lifecycle(),
        WorthQueryManagedIndexLifecycle::Ready
    );
    assert_eq!(
        keys(request.query(prefix_q("al")).execute().unwrap().rows()),
        [1, 3]
    );
    assert_eq!(
        keys(request.query(focus_q(70, 95)).execute().unwrap().rows()),
        [1, 3]
    );

    title_idx.close();
    focus_idx.close();
    reopened.close(&execution).unwrap().reclaim();
    std::fs::remove_dir_all(location).unwrap();
}
