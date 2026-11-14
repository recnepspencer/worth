//! Count every Native publication on the caller-driven cabinet-shaped journey.
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_relational::facade::inspection::observe_maintenance_work;
type Host = WorthQueryProgramApplicationRuntime<Schema, program::Lazy>;

fn settle(host: &Host, request: &family::Request<'_, '_, '_>, model: &str) {
    let mut pending = request
        .advanced()
        .start_program_outputs::<program::Lazy, program::LazyTree>(
            host,
            Demand(model.into()),
            WorthQueryOutputDemandControls::default(),
        )
        .unwrap();
    for _ in 0..128 {
        if matches!(
            pending.advance(host, request).unwrap(),
            WorthQueryApplicationProgramOutputProgress::Settled(_)
        ) {
            return;
        }
    }
    panic!("cabinet-shaped output must settle within the request's bound");
}
fn read(request: &family::Request<'_, '_, '_>, model: &str) -> snapshot::Snapshot {
    request
        .query(snapshot::Read(model.into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone()
}
fn index(request: &family::Request<'_, '_, '_>, source: &str, value: u64) -> query::Row {
    request
        .query(query::child_output::Read(child::index_key(source, value)))
        .execute()
        .unwrap()
        .rows()[0]
        .clone()
}
fn position(ordinal: u64, count: u64) -> u64 {
    if ordinal == 0 {
        42
    } else {
        (ordinal + 1) * 1000 / (count + 1)
    }
}
fn run(worker: bool, survivors: u64, unrelated: u64) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let initial = survivors + 2;
    let host = world::install_configured::<program::Lazy>(
        limits,
        initial,
        Some(unrelated),
        world::Configuration {
            second_derivation: None,
            reconciliation_indices: true,
            reconciliation_peer_caps: false,
            reconciliation_emission: false,
            envelope_records: false,
            ..Default::default()
        },
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    if initial > 5 {
        let cap = request
            .advanced()
            .mutate(member_edit::Intent {
                key: "m".into(),
                value: initial,
            })
            .without_source()
            .idempotency(&"counting-cap".to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(cap, WorthQueryApplicationMutationOutcome::Committed { .. }),
            "{cap:?}"
        );
    }
    settle(&host, &request, "a");
    settle(&host, &request, "b");
    let other = read(&request, "b");
    assert_eq!(other.rows.len(), unrelated as usize + 1);
    let before = read(&request, "a");
    assert_eq!(before.rows.len(), initial as usize + 1);
    let old_indices = (0..initial)
        .map(|ordinal| {
            let source = observation::source_key("a", ordinal);
            index(&request, &source, position(ordinal, initial) * 100)
        })
        .collect::<Vec<_>>();
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let (_, commits) = observe_maintenance_work(128, || {
        let edit = request
            .advanced()
            .mutate(edit::Intent {
                key: "a".into(),
                value: survivors,
            })
            .without_source()
            .idempotency(&"counted-cabinet-shrink".to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(edit, WorthQueryApplicationMutationOutcome::Committed { .. }),
            "{edit:?}"
        );
        settle(&host, &request, "a");
    });
    let actual_contacts = host.producer_contacts_on_this_thread_for_test() - contacts;
    let actual_commits = commits.published_rows.len() + commits.omitted_publications;
    eprintln!("D1450 measured: survivors={survivors} unrelated={unrelated} contacts={actual_contacts} commits={actual_commits}");
    assert_eq!(
        actual_contacts,
        survivors + 1,
        "the edit and its first caller demand retain the existing Ready certifiers"
    );
    assert_eq!(
        actual_commits,
        survivors as usize + 2,
        "all branches: source + two root effects + one effect per moved child"
    );
    if survivors == 3 {
        assert_eq!(
            (actual_commits, actual_contacts),
            (5, 4),
            "the public fixture has five real publications and four producer contacts"
        );
    }
    let after = read(&request, "a");
    assert_eq!(after.rows.len(), survivors as usize + 1);
    for ordinal in 0..survivors {
        let key = observation::member_key("a", ordinal);
        let row = after.rows.iter().find(|row| row.row.key == key).unwrap();
        let original = before.rows.iter().find(|row| row.row.key == key).unwrap();
        assert_eq!(row.row.native_id, original.row.native_id);
        assert_eq!(row.row.value, position(ordinal, survivors));
        assert_eq!(row.children.len(), 2);
        let main = row
            .children
            .iter()
            .find(|child| child.key.starts_with("reconciled-child:"))
            .unwrap();
        let original_main = original
            .children
            .iter()
            .find(|child| child.key.starts_with("reconciled-child:"))
            .unwrap();
        assert_eq!(row.sources.len(), 1);
        assert_eq!(main.native_id, original_main.native_id);
        assert_eq!(main.value, position(ordinal, survivors) * 100);
        assert_eq!(row.sources[0], original.sources[0]);
        let current = index(&request, &row.sources[0].key, main.value);
        assert_eq!(current.value, main.value);
        if ordinal == 0 {
            assert_eq!(current.native_id, old_indices[0].native_id);
        } else {
            assert_ne!(
                current.native_id, old_indices[ordinal as usize].native_id,
                "changed index keys replace identities; they never rewrite the old key"
            );
            assert!(request
                .query(query::child_output::Read(
                    old_indices[ordinal as usize].key.clone()
                ))
                .execute()
                .is_err());
        }
    }
    for ordinal in survivors..initial {
        assert!(request
            .query(query::first_output::Read(observation::member_key(
                "a", ordinal
            )))
            .execute()
            .is_err());
        assert!(request
            .query(query::second_output::Read(observation::source_key(
                "a", ordinal
            )))
            .execute()
            .is_err());
        assert!(request
            .query(query::child_output::Read(format!(
                "reconciled-child:{}",
                observation::source_key("a", ordinal)
            )))
            .execute()
            .is_err());
        assert!(
            request
                .query(query::child_output::Read(
                    old_indices[ordinal as usize].key.clone()
                ))
                .execute()
                .is_err(),
            "departed source's complete publication retires its index under source-scope custody"
        );
        assert!(
            request
                .query(query::child_output::Read(format!(
                    "reconciled:retired:a:{ordinal}"
                )))
                .execute()
                .is_err(),
            "cleanup leaves no temporary entity, even orphaned"
        );
    }
    assert_eq!(
        read(&request, "b"),
        other,
        "unrelated generated graph is unchanged"
    );
    // The required-wave reader of each NRU publication executes once.
    // Adoption commits nothing; the following demand consumes that Ready.
    for expected_contacts in [survivors, 0] {
        let contacts = host.producer_contacts_on_this_thread_for_test();
        let (_, current) = observe_maintenance_work(8, || {
            settle(&host, &request, "a");
            assert_eq!(read(&request, "a"), after);
        });
        let actual = host.producer_contacts_on_this_thread_for_test() - contacts;
        let commits = current.published_rows.len() + current.omitted_publications;
        eprintln!("cabinet-shaped verification: survivors={survivors} expected_contacts={expected_contacts} contacts={actual} commits={commits}");
        assert_eq!(actual, expected_contacts);
        assert_eq!(commits, 0);
    }
    eprintln!("cabinet-shaped count: survivors={survivors} unrelated={unrelated} contacts={actual_contacts} commits={actual_commits}");
}
#[test]
fn caller_started_cabinet_shrink_has_five_commits_and_four_contacts() {
    for unrelated in [1, 7, 13] {
        run(false, 3, unrelated);
    }
}
#[test]
fn commit_count_tracks_affected_producers_across_model_growth() {
    for survivors in [2, 5] {
        run(false, survivors, 13);
    }
}
#[test]
fn worker_only_caller_started_cabinet_shrink() {
    run(true, 3, 13);
}
