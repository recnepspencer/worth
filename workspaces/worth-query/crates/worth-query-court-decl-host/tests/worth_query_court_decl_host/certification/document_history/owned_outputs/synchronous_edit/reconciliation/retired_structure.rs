//! Actual owned aggregates and walks can end while their producer root survives.
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_relational::facade::inspection::observe_maintenance_work;

#[test]
fn whole_owned_structural_reads_retire_in_the_same_coupled_commit() {
    let host = world::install_configured::<program::Synchronous>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2, None,
        world::Configuration { reconciliation_owned_orbits: true, ..Default::default() },
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    assert_eq!(kept_refusals::demand(&host, &request).0, Ok(()));
    let initial = request
        .query(snapshot::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    let root = initial
        .rows
        .iter()
        .find(|row| row.row.key == "reconciled:root:a")
        .unwrap()
        .row
        .native_id;
    let old_members = initial
        .rows
        .iter()
        .filter(|row| row.row.key.starts_with("reconciled:member:"))
        .map(|row| row.row.native_id)
        .collect::<Vec<_>>();
    assert_eq!(old_members.len(), 2);
    for count in [1, 0] {
        let (outcome, writes) = observe_maintenance_work(64, || {
            request
                .advanced()
                .mutate(edit::Intent {
                    key: "a".into(),
                    value: count,
                })
                .without_source()
                .idempotency(&format!("retire-owned-structure-{count}"))
                .execute_performed::<program::Synchronous, program::Tree>(
                    &host,
                    ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        });
        assert!(
            matches!(
                outcome,
                WorthQueryApplicationPerformedMutationOutcome::Performed(_)
            ),
            "{outcome:?}"
        );
        assert_eq!(writes.published_rows.len() + writes.omitted_publications, 1);
        let current = request
            .query(snapshot::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        assert_eq!(current.rows.len(), count as usize + 1);
        assert_eq!(
            current
                .rows
                .iter()
                .find(|row| row.row.key == "reconciled:root:a")
                .unwrap()
                .row
                .native_id,
            root
        );
        if count == 1 {
            assert!(current
                .rows
                .iter()
                .any(|row| old_members.contains(&row.row.native_id)));
        } else {
            assert!(current
                .rows
                .iter()
                .all(|row| !old_members.contains(&row.row.native_id)));
        }
        assert_eq!(
            kept_refusals::demand(&host, &request).0,
            Ok(()),
            "the real current output remains deliverable"
        );
    }
}
