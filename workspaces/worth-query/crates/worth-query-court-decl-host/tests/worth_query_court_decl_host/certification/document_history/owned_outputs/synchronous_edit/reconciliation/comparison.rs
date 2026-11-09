//! Compare the caller-driven row oracle with the declared synchronous entry.
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_relational::facade::inspection::observe_maintenance_work;

type NamedRows = Vec<(String, u64, Vec<(String, u64)>, Vec<(String, u64)>)>;
fn named(snapshot: &snapshot::Snapshot) -> NamedRows {
    snapshot
        .rows
        .iter()
        .map(|owned| {
            (
                owned.row.key.clone(),
                owned.row.value,
                owned
                    .children
                    .iter()
                    .map(|row| (row.key.clone(), row.value))
                    .collect(),
                owned
                    .sources
                    .iter()
                    .map(|row| (row.key.clone(), row.value))
                    .collect(),
            )
        })
        .collect()
}
fn compare(worker: bool) {
    courts::with_legacy_rows(worker, |_, _, legacy| {
        let limits = if worker {
            world::worker_limits()
        } else {
            host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
        };
        let host = world::install_program_with_value::<program::Synchronous>(limits, 5);
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        let mut original = request
            .advanced()
            .start_program_outputs::<program::Synchronous, program::Tree>(
                &host,
                Demand("a".into()),
                WorthQueryOutputDemandControls::default(),
            )
            .unwrap();
        let mut settled = false;
        for _ in 0..128 {
            if matches!(
                original.advance(&host, &request).unwrap(),
                WorthQueryApplicationProgramOutputProgress::Settled(_)
            ) {
                settled = true;
                break;
            }
        }
        assert!(settled);
        let before = request
            .query(snapshot::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        let (outcome, publications) = observe_maintenance_work(64, || {
            request
                .advanced()
                .mutate(edit::Intent {
                    key: "a".into(),
                    value: 2,
                })
                .without_source()
                .idempotency(&"synchronous-reconciliation-shrink".to_owned())
                .execute_performed::<program::Synchronous, program::Tree>(
                    &host,
                    ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        });
        let _performed = match outcome {
            WorthQueryApplicationPerformedMutationOutcome::Performed(done) => done,
            WorthQueryApplicationPerformedMutationOutcome::NotPerformed(cause) => {
                panic!("source refused: {cause:?}")
            }
            WorthQueryApplicationPerformedMutationOutcome::RequiredOutputDenied(cause) => {
                panic!("output refused: {:?}", cause.denial())
            }
            WorthQueryApplicationPerformedMutationOutcome::Blocked(_) => panic!("source blocked"),
            WorthQueryApplicationPerformedMutationOutcome::ProductUnpublished(_) => {
                panic!("source unpublished")
            }
            WorthQueryApplicationPerformedMutationOutcome::CleanupRequired { .. } => {
                panic!("cleanup required")
            }
        };
        let after = request
            .query(snapshot::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .clone();
        assert_eq!(named(&after), named(&legacy), "the independent multi-commit journey and synchronous edit end with the same complete rows and adjacency");
        for survivor in &after.rows {
            let original = before
                .rows
                .iter()
                .find(|row| row.row.key == survivor.row.key)
                .unwrap();
            assert_eq!(survivor.row.native_id, original.row.native_id);
            for source in survivor.sources.iter().chain(&survivor.children) {
                assert_eq!(
                    source.native_id,
                    original
                        .sources
                        .iter()
                        .chain(&original.children)
                        .find(|row| row.key == source.key)
                        .unwrap()
                        .native_id
                );
            }
        }
        assert_eq!(
            publications.published_rows.len() + publications.omitted_publications,
            1,
            "one edit, across all branches"
        );
    });
}
#[test]
#[ignore = "Postponed: public #119 / proprietary #100 retirement metadata; assertions retained"]
fn synchronous_shrink_matches_the_legacy_complete_row_oracle() {
    compare(false);
}
#[test]
#[ignore = "Postponed: public #119 / proprietary #100 retirement metadata; assertions retained"]
fn worker_only_synchronous_shrink_matches_the_legacy_complete_row_oracle() {
    compare(true);
}
