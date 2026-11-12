//! A genuine C retirement in a synchronous edit stays in the edit's origin
//! as `Retire` and is gone from live state at the edit commit (CP2b, D1474
//! item N1, D1572).
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_relational::facade::inspection::observe_maintenance_work;

type Host = WorthQueryProgramApplicationRuntime<Schema, program::Synchronous>;

/// (role, posture) of every child row in each lineage record's correspondence.
fn child_rosters(host: &Host) -> Vec<Vec<(String, WorthQueryApplicationOutputPosture)>> {
    let graph = host
        .runtime()
        .runtime_for_test()
        .primary_provider_for_test()
        .graph_for_test();
    let lineage = graph.output_lineage.lock().unwrap();
    let mut rosters = Vec::new();
    for branches in lineage.by_source_for_test().values() {
        for generations in branches.values() {
            for records in generations.values() {
                for record in records {
                    let Some(record) = record.get() else { continue };
                    let Ok(outputs) = record
                        .correspondence_for_test()
                        .outputs_of_for_test::<preserve::Outputs>()
                    else {
                        continue;
                    };
                    let mut roster = Vec::new();
                    if let Ok(rows) =
                        outputs.family_entries::<Rows<preserve::Outputs, FirstOutput>>()
                    {
                        roster.extend(
                            rows.iter()
                                .map(|row| (row.role().to_owned(), row.posture())),
                        );
                    }
                    if let Ok(rows) =
                        outputs.family_entries::<Rows<preserve::Outputs, SecondOutput>>()
                    {
                        roster.extend(
                            rows.iter()
                                .map(|row| (row.role().to_owned(), row.posture())),
                        );
                    }
                    if let Ok(rows) =
                        outputs.family_entries::<Rows<preserve::Outputs, ChildOutput>>()
                    {
                        roster.extend(
                            rows.iter()
                                .map(|row| (row.role().to_owned(), row.posture())),
                        );
                    }
                    rosters.push(roster);
                }
            }
        }
    }
    rosters
}

fn genuine_retirement(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host = world::install_program_with_value::<program::Synchronous>(limits, 5);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = super::kept_refusals::demand(&host, &request);
    assert_eq!(before.0, Ok(()));
    let (outcome, work) = observe_maintenance_work(64, || {
        request
            .advanced()
            .mutate(edit::Intent {
                key: "a".into(),
                value: 2,
            })
            .without_source()
            .idempotency(&"genuine-retirement".to_owned())
            .execute_performed::<program::Synchronous, program::Tree>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    });
    assert!(matches!(
        outcome,
        WorthQueryApplicationPerformedMutationOutcome::Performed(_)
    ));
    assert_eq!(
        work.published_rows.len() + work.omitted_publications,
        1,
        "the shrink is one commit"
    );
    drop(outcome);
    // In the edit's origin: the members C held and the edit removed are Retire.
    use WorthQueryApplicationOutputPosture::{Create, Preserve, Retire};
    let expected = [
        ("member.reconciled:member:a:0", Preserve),
        ("member.reconciled:member:a:1", Preserve),
        ("member.reconciled:member:a:2", Retire),
        ("member.reconciled:member:a:3", Retire),
        ("member.reconciled:member:a:4", Retire),
        ("source.reconciled:source:a:0", Preserve),
        ("source.reconciled:source:a:1", Preserve),
        ("source.reconciled:source:a:2", Retire),
        ("source.reconciled:source:a:3", Retire),
        ("source.reconciled:source:a:4", Retire),
        ("retired.reconciled:retired:a:2", Create),
        ("retired.reconciled:retired:a:3", Create),
        ("retired.reconciled:retired:a:4", Create),
    ]
    .map(|(role, posture)| (role.to_owned(), posture));
    assert_eq!(
        child_rosters(&host),
        vec![expected.to_vec()],
        "the edit's one producing record keeps every genuine retirement as Retire"
    );
    // In live state at the edit commit: only the preserved members remain.
    let rows = request
        .query(snapshot::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    let live: Vec<String> = rows
        .rows
        .iter()
        .map(|owned| owned.row.key.clone())
        .collect();
    assert_eq!(
        live,
        ["reconciled:member:a:0", "reconciled:member:a:1", "reconciled:root:a"],
        "the retired members are gone from live state at the edit commit; the preserved ones and the root stay"
    );
}

#[test]
fn a_genuine_retirement_stays_in_the_origin_and_leaves_live_state() {
    genuine_retirement(false);
}
#[test]
fn worker_only_genuine_retirement_stays_in_the_origin_and_leaves_live_state() {
    genuine_retirement(true);
}
