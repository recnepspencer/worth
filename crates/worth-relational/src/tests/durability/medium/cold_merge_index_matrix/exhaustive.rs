use super::*;

#[test]
fn residency_failure_matrix_merge_planning_stays_cold_and_native_commit_requires_explicit_index_recovery(
) {
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
        let refused = fixture
            .runtime
            .execute_prepared_merge(execution, plan)
            .unwrap_err();
        assert!(format!("{refused:?}").contains("ColdReconstructionRequired"));
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
        fixture.source.failures.lock().unwrap().clear();
        let (_, basis) = fixture
            .runtime
            .observe_branch(&fixture.runtime.main_branch_identity())
            .unwrap();
        let recovered = fixture.runtime.index_authority().build_for_basis(
            execution,
            DerivedIndexBuildRequest {
                source_commit_id: basis.observation().commit_id().unwrap(),
                branch_id: BranchId("main".into()),
                index_ids: vec![fixture.index.unwrap()],
            },
            &basis,
        );
        assert!(recovered.failed_indexes.is_empty(), "{recovered:?}");
        assert!(fixture
            .source
            .member_reads
            .lock()
            .unwrap()
            .contains(&member));
        let plan = fixture
            .runtime
            .prepare_merge_execution(execution, request())
            .unwrap();
        let outcome = fixture
            .runtime
            .execute_prepared_merge(execution, plan)
            .unwrap();
        let actual = fixture
            .runtime
            .read_truth()
            .read_snapshot(&outcome.commit.snapshot, execution)
            .unwrap()
            .unwrap();
        assert_merge_records(
            actual.entities(),
            &fixture.expected,
            fixture.expected_merge_version,
            outcome.commit.version_id,
        );
        fixture.source.failures.lock().unwrap().clear();
        let rebuilt = fixture.runtime.index_authority().build_for_commit(
            execution,
            DerivedIndexBuildRequest {
                source_commit_id: outcome.commit.commit.commit_id,
                branch_id: BranchId("main".into()),
                index_ids: vec![fixture.index.unwrap()],
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
            .read_latest_generation(execution, fixture.index.unwrap(), &BranchId("main".into()))
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
