//! Establish the old multi-commit final-row oracle before changing continuation.
use super::*;
use std::collections::BTreeMap;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_relational::facade::inspection::observe_maintenance_work;

fn settle(
    host: &WorthQueryProgramApplicationRuntime<Schema, program::Lazy>,
    request: &family::Request<'_, '_, '_>,
) -> WorthQueryApplicationProgramOutputHandle<Schema, program::Lazy, program::LazyTree> {
    let mut demand = request
        .advanced()
        .start_program_outputs::<program::Lazy, program::LazyTree>(
            host,
            Demand("a".into()),
            WorthQueryOutputDemandControls::default(),
        )
        .unwrap_or_else(|cause| panic!("{cause:?}"));
    for _ in 0..128 {
        if let WorthQueryApplicationProgramOutputProgress::Settled(_) =
            demand.advance(host, request).unwrap()
        {
            return demand;
        }
    }
    panic!("the reconciliation tree did not settle");
}

pub(super) fn with_legacy_rows(
    worker: bool,
    verify: impl FnOnce(
        &WorthQueryProgramApplicationRuntime<Schema, program::Lazy>,
        &family::Request<'_, '_, '_>,
        snapshot::Snapshot,
    ),
) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host = world::install_program_with_value::<program::Lazy>(limits, 5);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let _original = settle(&host, &request);
    let before = request
        .query(snapshot::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    assert_eq!(before.rows.len(), 6);
    let original_ids = before
        .rows
        .iter()
        .map(|row| (row.row.key.clone(), row.row.native_id))
        .collect::<BTreeMap<_, _>>();
    let original_dependents = before
        .rows
        .iter()
        .flat_map(|row| row.sources.iter().chain(&row.children))
        .map(|row| (row.key.clone(), row.native_id))
        .collect::<BTreeMap<_, _>>();
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let (_revised, publications) = observe_maintenance_work(64, || {
        let edited = request
            .advanced()
            .mutate(edit::Intent {
                key: "a".into(),
                value: 2,
            })
            .without_source()
            .idempotency(&"legacy-reconciliation-shrink".to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(
                edited,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "{edited:?}"
        );
        settle(&host, &request)
    });
    let snapshot = request
        .query(snapshot::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    let expected = BTreeMap::from([
        ("reconciled:root:a".to_owned(), (2, Vec::new(), Vec::new())),
        (
            "reconciled:member:a:0".to_owned(),
            (
                42,
                vec![("reconciled-child:reconciled:source:a:0".to_owned(), 4200)],
                vec![("reconciled:source:a:0".to_owned(), 0)],
            ),
        ),
        (
            "reconciled:member:a:1".to_owned(),
            (
                666,
                vec![("reconciled-child:reconciled:source:a:1".to_owned(), 66600)],
                vec![("reconciled:source:a:1".to_owned(), 0)],
            ),
        ),
    ]);
    let actual = snapshot
        .rows
        .iter()
        .map(|owned| {
            for dependent in owned.sources.iter().chain(&owned.children) {
                if let Some(original) = original_dependents.get(&dependent.key) {
                    assert_eq!(
                        &dependent.native_id, original,
                        "surviving sources and children keep their identities"
                    );
                }
            }
            assert_eq!(
                Some(&owned.row.native_id),
                original_ids.get(&owned.row.key),
                "survivors keep their identities"
            );
            (
                owned.row.key.clone(),
                (
                    owned.row.value,
                    owned
                        .children
                        .iter()
                        .map(|row| (row.key.clone(), row.value))
                        .collect::<Vec<_>>(),
                    owned
                        .sources
                        .iter()
                        .map(|row| (row.key.clone(), row.value))
                        .collect::<Vec<_>>(),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        actual, expected,
        "complete named rows and adjacency, not just their count"
    );
    for ordinal in 2..5 {
        assert!(request
            .query(query::first_output::Read(observation::member_key(
                "a", ordinal
            )))
            .execute()
            .is_err());
        assert!(request
            .query(query::child_output::Read(format!(
                "reconciled-child:reconciled:source:a:{ordinal}"
            )))
            .execute()
            .is_err());
        assert!(request
            .query(query::second_output::Read(observation::source_key(
                "a", ordinal
            )))
            .execute()
            .is_err());
        assert!(
            request
                .query(query::child_output::Read(format!(
                    "reconciled:retired:a:{ordinal}"
                )))
                .execute()
                .is_err(),
            "no temporary retirement row remains orphaned"
        );
    }
    assert_eq!(
        request
            .query(query::model::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        2
    );
    eprintln!(
        "legacy reconciliation: contacts={} commits={} rows={:?}",
        host.producer_contacts_on_this_thread_for_test() - contacts,
        publications.published_rows.len() + publications.omitted_publications,
        actual
    );
    verify(&host, &request, snapshot);
}

fn legacy(worker: bool) {
    with_legacy_rows(worker, |host, request, snapshot| {
        let contacts = host.producer_contacts_on_this_thread_for_test();
        let (_, ready) = observe_maintenance_work(8, || {
            settle(&host, &request);
            assert_eq!(
                request
                    .query(snapshot::Read("a".into()))
                    .execute()
                    .unwrap()
                    .rows()[0],
                snapshot
            );
        });
        assert_eq!(
            host.producer_contacts_on_this_thread_for_test(),
            contacts,
            "a current demand needs no producer"
        );
        assert_eq!(
            ready.published_rows.len() + ready.omitted_publications,
            0,
            "current demand certification and measure reads publish on no branch"
        );
    });
}
#[test]
fn legacy_shrink_clears_one_round_facts_and_respaces_surviving_rows() {
    legacy(false);
}
#[test]
fn worker_only_legacy_shrink_clears_one_round_facts_and_respaces_surviving_rows() {
    legacy(true);
}
