//! Selected-state merge reads do not promote an unconsulted generation cache.
use super::super::cold_read_matrix::FAILURES;
use super::super::cold_source::ColdRead;
use super::*;
#[test]
fn residency_failure_matrix_merge_planning_and_commit_leave_unconsulted_generations_cold() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for (cause, _) in FAILURES {
        let fixture = recovered(execution);
        let member = fixture.manifest.derived_indexes;
        fixture
            .source
            .failures
            .lock()
            .unwrap()
            .insert(ColdRead::Member(member), cause);
        let frontier = fixture.runtime.history().latest_patch_stream_position();
        let offers = attached_test_medium(&fixture.runtime).offered().len();
        let positions = fixture.runtime.patch_position_reservation_counters();
        let plan = fixture
            .runtime
            .prepare_merge_execution(execution, request())
            .unwrap();
        assert!(!fixture
            .source
            .member_reads
            .lock()
            .unwrap()
            .contains(&member));
        assert_eq!(
            fixture.runtime.history().latest_patch_stream_position(),
            frontier
        );
        assert_eq!(
            attached_test_medium(&fixture.runtime).offered().len(),
            offers
        );
        assert_eq!(
            fixture.runtime.patch_position_reservation_counters(),
            positions
        );
        let outcome = fixture
            .runtime
            .execute_prepared_merge(execution, plan)
            .unwrap();
        assert!(
            !fixture
                .source
                .member_reads
                .lock()
                .unwrap()
                .contains(&member),
            "selected-state merge consumes no generation payload even with {cause:?} armed"
        );
        let actual = fixture
            .runtime
            .read_truth()
            .read_snapshot(&outcome.commit.snapshot, execution)
            .unwrap()
            .unwrap();
        assert_merge_records(actual.entities(), &fixture.expected);
        fixture.source.failures.lock().unwrap().clear();
        let rebuilt = fixture.runtime.index_authority().build_for_commit(
            execution,
            DerivedIndexBuildRequest {
                source_commit_id: outcome.commit.commit.commit_id,
                branch_id: BranchId("main".into()),
                index_ids: vec![fixture.index],
            },
        );
        assert!(rebuilt.failed_indexes.is_empty());
        assert!(
            fixture
                .source
                .member_reads
                .lock()
                .unwrap()
                .contains(&member),
            "explicit reconstruction actually admits the image"
        );
        let generation = fixture
            .runtime
            .index_access()
            .read_latest_generation(execution, fixture.index, &BranchId("main".into()))
            .unwrap()
            .unwrap();
        assert_eq!(generation.source_commit_id, outcome.commit.commit.commit_id);
        let DerivedIndexEntries::EntityField(entries) = &generation.entries else {
            panic!("actual entity index");
        };
        assert_eq!(
            entries.keys().cloned().collect::<Vec<_>>(),
            fixture.expected_index_keys
        );
        let indexed = entries
            .values()
            .flat_map(|rows| rows.iter().copied())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            indexed,
            actual.entities().iter().map(|row| row.entity_id).collect()
        );
    }
}
