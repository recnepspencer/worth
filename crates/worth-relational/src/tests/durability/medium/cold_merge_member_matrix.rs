//! Historical merge planning admits its Full inputs before executing the merge.
use super::super::cold_read_matrix::FAILURES;
use super::super::cold_source::ColdRead;
use super::*;
use crate::storage::data::{RelationalStorageSubject as Storage, RelationalStorageUnavailable};
#[derive(Clone, Copy, Debug)]
enum Subject {
    Head,
    History,
    Symbols,
    Nodes,
    Events,
}
impl Subject {
    const ALL: [Self; 5] = [
        Self::Head,
        Self::History,
        Self::Symbols,
        Self::Nodes,
        Self::Events,
    ];
    fn targets(
        self,
        fixture: &Fixture,
    ) -> std::collections::BTreeSet<RelationalDurableMemberLocator> {
        match self {
            Self::Head | Self::History => fixture
                .manifest
                .roots
                .iter()
                .flat_map(|root| &root.regions)
                .chain(&fixture.manifest.mirror)
                .map(|region| {
                    if matches!(self, Self::Head) {
                        region.head
                    } else {
                        region.history
                    }
                })
                .collect(),
            Self::Symbols => {
                let symbol = fixture.symbol.unwrap();
                fixture
                    .manifest
                    .symbols
                    .iter()
                    .filter(|page| {
                        page.first <= u64::from(symbol.0) && u64::from(symbol.0) <= page.last
                    })
                    .map(|page| page.member)
                    .collect()
            }
            Self::Nodes => fixture
                .manifest
                .lineage
                .iter()
                .map(|page| page.member)
                .collect(),
            Self::Events => fixture
                .manifest
                .lineage_events
                .iter()
                .map(|page| page.member)
                .collect(),
        }
    }
    fn expected(self, fixture: &Fixture, cause: Cause) -> Option<MergeExecutionPreparationError> {
        let subject = match self {
            Self::Head | Self::History => Storage::Region(PartitionId::main()),
            Self::Symbols => {
                let symbol = fixture.symbol.unwrap();
                let page = fixture
                    .manifest
                    .symbols
                    .iter()
                    .find(|page| {
                        page.first <= u64::from(symbol.0) && u64::from(symbol.0) <= page.last
                    })
                    .unwrap();
                Storage::Symbols {
                    first: crate::symbols::data::Symbol(page.first.try_into().unwrap()),
                    last: crate::symbols::data::Symbol(page.last.try_into().unwrap()),
                }
            }
            Self::Nodes | Self::Events => return None,
        };
        Some(MergeExecutionPreparationError::Planning(
            MergePlanningError::Execution(RelationalExecutionDenialCause::StorageUnavailable(
                RelationalStorageUnavailable { subject, cause },
            )),
        ))
    }
}
#[test]
fn residency_failure_matrix_merge_planning_preserves_versioned_footprints() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for subject in Subject::ALL {
        for (cause, sticky) in FAILURES {
            let fixture = recovered_shape(execution, matches!(subject, Subject::Symbols));
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
            let offered = attached_test_medium(&fixture.runtime).offered().len();
            let reservations = fixture.runtime.patch_position_reservation_counters();
            let prepared = fixture
                .runtime
                .prepare_merge_execution(execution, request());
            assert_eq!(
                fixture.runtime.history().latest_patch_stream_position(),
                frontier
            );
            assert_eq!(
                attached_test_medium(&fixture.runtime).offered().len(),
                offered
            );
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
                            .prepare_merge_execution(execution, request())
                            .unwrap_err(),
                        expected
                    );
                    assert_eq!(failed_reads(), before);
                    None
                } else {
                    let accepted = fixture
                        .runtime
                        .prepare_merge_execution(execution, request())
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
                assert_merge_records(actual.entities(), &fixture.expected);
            }
        }
    }
}
