//! A non-leaf required wave keeps original roles while its aggregate follows cleanup.
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
fn run(worker: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host = world::install_program_with_value::<program::Lazy>(limits, 4);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let _initial = courts::settle(&host, &request);
    let edit = request
        .advanced()
        .mutate(edit::Intent {
            key: "a".into(),
            value: 2,
        })
        .without_source()
        .idempotency(&"shrink-reconciled-members-with-moving-survivor".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(edit, WorthQueryApplicationMutationOutcome::Committed { .. }),
        "{edit:?}"
    );
    let model = request
        .query(query::model::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .native_id;
    let root_before = host.producer_contacts_at_root_on_this_thread_for_test(model);
    let _settled = courts::settle_with_result(&host, &request, |settled| {
        assert!(
            host.producer_contacts_at_root_on_this_thread_for_test(model) - root_before
                > u64::try_from(settled.root_settlement().producer_contacts_in_this_demand())
                    .unwrap(),
            "this root really executes cleanup, including upstream execution opened by its child"
        );
        assert_eq!(
            settled.output_count(),
            2,
            "the moved survivor enters the real required wave"
        );
        assert!(
            settled.observation().selected_commit().ordinal()
                > settled.root_observation().selected_commit().ordinal(),
            "aggregate observation follows the original root's real cleanup publication"
        );
        let roles = settled
            .root_settlement()
            .outputs_of::<preserve::Outputs>()
            .unwrap();
        let actual = roles
            .family_entries::<Rows<preserve::Outputs, ChildOutput>>()
            .unwrap()
            .iter()
            .filter(|row| row.role().contains("reconciled:retired:a:"))
            .map(|row| (row.role().to_owned(), row.posture()))
            .collect::<Vec<_>>();
        let expected = vec![
            (
                "retired.reconciled:retired:a:2".to_owned(),
                WorthQueryApplicationOutputPosture::Create,
            ),
            (
                "retired.reconciled:retired:a:3".to_owned(),
                WorthQueryApplicationOutputPosture::Create,
            ),
        ];
        assert_eq!(
            actual, expected,
            "the immutable original root describes its actual created facts"
        );
        // Admission still resolves the live parent at C. Its retained read at
        // the original root observation projects the historical child rows.
        let original = request
            .at(settled.root_observation())
            .query(snapshot::Read("a".into()))
            .execute()
            .unwrap();
        let temporary = original.rows()[0]
            .rows
            .iter()
            .flat_map(|row| &row.children)
            .filter(|row| row.key.starts_with("reconciled:retired:a:"))
            .map(|row| row.key.clone())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            temporary,
            std::collections::BTreeSet::from([
                "reconciled:retired:a:2".to_owned(),
                "reconciled:retired:a:3".to_owned(),
            ])
        );
        drop(original);
        for i in 2..4 {
            assert!(
                request
                    .at(settled.observation())
                    .query(query::child_output::Read(format!(
                        "reconciled:retired:a:{i}"
                    )))
                    .execute()
                    .is_err(),
                "final aggregate observation contains no temporary retirement rows"
            );
        }
        let snapshot = request
            .at(settled.observation())
            .query(snapshot::Read("a".into()))
            .execute()
            .unwrap();
        let actual = snapshot.rows()[0]
            .rows
            .iter()
            .map(|owned| {
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
            .collect::<std::collections::BTreeMap<_, _>>();
        let expected = std::collections::BTreeMap::from([
            (
                "reconciled:member:a:1".to_owned(),
                (
                    666,
                    vec![("reconciled-child:reconciled:source:a:1".to_owned(), 66600)],
                    vec![("reconciled:source:a:1".to_owned(), 0)],
                ),
            ),
            ("reconciled:root:a".to_owned(), (2, Vec::new(), Vec::new())),
            (
                "reconciled:member:a:0".to_owned(),
                (
                    42,
                    vec![("reconciled-child:reconciled:source:a:0".to_owned(), 4200)],
                    vec![("reconciled:source:a:0".to_owned(), 0)],
                ),
            ),
        ]);
        assert_eq!(
            actual, expected,
            "all named rows and adjacency at the final observation"
        );
    });
}
#[test]
fn required_wave_original_roles_are_separate_from_aggregate_currentness() {
    run(false);
}
#[test]
fn worker_only_required_wave_original_roles_are_separate_from_aggregate_currentness() {
    run(true);
}
