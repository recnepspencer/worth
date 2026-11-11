//! Every versioned merge member refusal preserves planning, retry and execute meaning.
use super::{
    lineage_event_matrix::FAILURES,
    merge_member_targets::Subject,
    merge_publications::request,
    merge_source::{assert_merge_records, MergeRecipe},
};
use worth_testkit_relational::durable_medium::immutable_members::ColdRead;

#[test]
fn residency_failure_matrix_merge_planning_preserves_versioned_footprints() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    // No derived index: a recovered indexed runtime refuses Native merge commit
    // until explicit index recovery, which merge_index_reads owns.
    let plain = MergeRecipe::new(execution, false, false);
    let symbols = MergeRecipe::new(execution, true, false);
    for subject in Subject::ALL {
        for (cause, sticky) in FAILURES {
            let fixture = if matches!(subject, Subject::Symbols) {
                symbols.reopen(execution)
            } else {
                plain.reopen(execution)
            };
            let targets = subject.targets(&fixture);
            assert!(
                !targets.is_empty(),
                "each cell injects actual named members"
            );
            fixture.source.failures.lock().unwrap().extend(
                targets
                    .iter()
                    .map(|member| (ColdRead::Member(*member), cause)),
            );
            let frontier = fixture.runtime.history().latest_patch_stream_position();
            let offered = fixture.medium.offered().len();
            let reservations = fixture.runtime.patch_position_reservation_counters();
            let prepared = fixture
                .runtime
                .prepare_merge_execution(execution, request(&fixture.runtime));
            assert_eq!(
                fixture.runtime.history().latest_patch_stream_position(),
                frontier
            );
            assert_eq!(fixture.medium.offered().len(), offered);
            assert_eq!(
                fixture.runtime.patch_position_reservation_counters(),
                reservations
            );
            let accepted = if let Some(expected) = subject.expected(&fixture, cause) {
                assert_eq!(prepared.unwrap_err(), expected, "{subject:?} x {cause:?}");
                fixture.source.failures.lock().unwrap().clear();
                let failed_reads = || {
                    fixture
                        .source
                        .member_reads
                        .lock()
                        .unwrap()
                        .iter()
                        .filter(|member| targets.contains(member))
                        .count()
                };
                let before = failed_reads();
                if sticky {
                    assert_eq!(
                        fixture
                            .runtime
                            .prepare_merge_execution(execution, request(&fixture.runtime))
                            .unwrap_err(),
                        expected
                    );
                    assert_eq!(failed_reads(), before);
                    None
                } else {
                    let accepted = fixture
                        .runtime
                        .prepare_merge_execution(execution, request(&fixture.runtime))
                        .unwrap();
                    assert!(failed_reads() > before);
                    Some(accepted)
                }
            } else {
                assert!(
                    !fixture
                        .source
                        .member_reads
                        .lock()
                        .unwrap()
                        .iter()
                        .any(|member| targets.contains(member)),
                    "unconsulted {subject:?}"
                );
                Some(prepared.unwrap_or_else(|failure| {
                    panic!("current merge touched {subject:?}: {failure:?}")
                }))
            };
            assert_eq!(
                fixture.runtime.patch_position_reservation_counters(),
                reservations
            );
            if let Some(accepted) = accepted {
                let reads = fixture.source.member_reads.lock().unwrap().len();
                let commits = fixture.source.commit_reads.lock().unwrap().len();
                let outcome = fixture
                    .runtime
                    .execute_prepared_merge(execution, accepted)
                    .unwrap();
                assert_eq!(
                    fixture.source.member_reads.lock().unwrap().len(),
                    reads,
                    "combined execution reuses admitted planning members"
                );
                assert_eq!(
                    fixture.source.commit_reads.lock().unwrap().len(),
                    commits,
                    "execution reuses admitted planning envelopes"
                );
                assert_eq!(outcome.structural_summary.adopted_source_record_count, 1);
                fixture.source.failures.lock().unwrap().clear();
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
            }
        }
    }
}
