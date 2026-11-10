//! Staged recovery preserves every consumed/unconsumed input, refusal and final value.
use super::{
    lineage_event_matrix::FAILURES,
    tail_recovery_source::{recover, TailRecipe},
    tail_recovery_subject::Input,
};
use std::collections::BTreeSet;
use worth_relational::facade::{
    durability::{RecoveryFailureClass, RelationalRecoveryStop},
    storage::{RelationalStorageSubject as Subject, RelationalStorageUnavailable},
};
use worth_testkit_relational::{
    aspect_storage::publications::begin_main_transaction,
    durable_medium::{immutable_members::ColdRead, TestDurableMedium},
    durable_publication::durable_runtime,
};

#[test]
fn residency_failure_matrix_tail_recovery_stages_every_consumed_subject_before_effects() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let recipe = TailRecipe::new(execution);
    for input in Input::ALL {
        for (cause, _) in FAILURES {
            let fixture = recipe.fixture();
            let targets: BTreeSet<_> = match input {
                Input::Head | Input::History => fixture
                    .directory
                    .roots
                    .iter()
                    .flat_map(|root| &root.regions)
                    .chain(&fixture.directory.mirror)
                    .map(|region| {
                        ColdRead::Member(if matches!(input, Input::Head) {
                            region.head
                        } else {
                            region.history
                        })
                    })
                    .collect(),
                Input::Symbols => fixture
                    .directory
                    .symbols
                    .iter()
                    .filter(|page| {
                        page.first <= u64::from(fixture.symbol.0)
                            && u64::from(fixture.symbol.0) <= page.last
                    })
                    .map(|page| ColdRead::Member(page.member))
                    .collect(),
                Input::Buckets => fixture
                    .directory
                    .symbol_buckets
                    .iter()
                    .flat_map(|bucket| bucket.pages.iter().copied())
                    .map(ColdRead::Member)
                    .collect(),
                Input::Nodes => fixture
                    .directory
                    .lineage
                    .iter()
                    .map(|page| ColdRead::Member(page.member))
                    .collect(),
                Input::Events => fixture
                    .directory
                    .lineage_events
                    .iter()
                    .map(|page| ColdRead::Member(page.member))
                    .collect(),
                Input::Index => {
                    BTreeSet::from([ColdRead::Member(fixture.directory.derived_indexes)])
                }
                Input::Envelope => BTreeSet::from([ColdRead::Commit(
                    fixture.recovery.checkpoint().unwrap().covered(),
                )]),
                Input::TailEnvelope => fixture
                    .recovery
                    .tail()
                    .iter()
                    .map(|artifact| ColdRead::Commit(artifact.position()))
                    .collect(),
            };
            assert!(
                !targets.is_empty(),
                "{input:?} names actual producer members"
            );
            fixture
                .source
                .failures
                .lock()
                .unwrap()
                .extend(targets.iter().map(|key| (*key, cause)));
            let medium = TestDurableMedium::new();
            let mut runtime = durable_runtime(&medium);
            let frontier = runtime.history().latest_patch_stream_position();
            let offers = medium.offered().len();
            let reservations = runtime.patch_position_reservation_counters();
            let result = recover(
                execution,
                &mut runtime,
                super::immutable_recovery_input::copy_transport(
                    &fixture.recovery,
                    fixture.source.clone(),
                ),
            );
            if matches!(
                input,
                Input::Head | Input::Symbols | Input::Index | Input::Envelope
            ) {
                let RelationalRecoveryStop::Durability(error) = result.unwrap_err() else {
                    panic!("typed durable refusal required")
                };
                assert_eq!(
                    error.class,
                    RecoveryFailureClass::ColdStorageUnavailable,
                    "{input:?} x {cause:?}"
                );
                let expected_subject = if matches!(input, Input::Head) {
                    Subject::Region(fixture.entity.partition_id)
                } else if matches!(input, Input::Envelope) {
                    Subject::Envelope(fixture.recovery.checkpoint().unwrap().covered())
                } else if matches!(input, Input::Index) {
                    Subject::Index(fixture.index)
                } else {
                    let page = fixture
                        .directory
                        .symbols
                        .iter()
                        .find(|page| {
                            page.first <= u64::from(fixture.symbol.0)
                                && u64::from(fixture.symbol.0) <= page.last
                        })
                        .unwrap();
                    Subject::Symbols {
                        first: worth_relational::facade::symbols::Symbol(
                            page.first.try_into().unwrap(),
                        ),
                        last: worth_relational::facade::symbols::Symbol(
                            page.last.try_into().unwrap(),
                        ),
                    }
                };
                assert_eq!(
                    error.storage_unavailable,
                    Some(RelationalStorageUnavailable {
                        subject: expected_subject,
                        cause
                    })
                );
                assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
                assert!(runtime
                    .history()
                    .branch_head_for_observation(
                        &runtime
                            .observe_branch(&runtime.main_branch_identity())
                            .unwrap()
                            .1
                            .observation()
                    )
                    .unwrap()
                    .is_none());
                assert_eq!(runtime.patch_position_reservation_counters(), reservations);
                assert_eq!(medium.offered().len(), offers);
                fixture.source.failures.lock().unwrap().clear();
                recover(execution, &mut runtime, fixture.recovery).unwrap();
            } else {
                result.unwrap_or_else(|failure| {
                    panic!("tail update consulted {input:?}: {failure:?}")
                });
                assert!(!fixture
                    .source
                    .member_reads
                    .lock()
                    .unwrap()
                    .iter()
                    .any(|member| targets.contains(&ColdRead::Member(*member))));
                assert!(!fixture
                    .source
                    .commit_reads
                    .lock()
                    .unwrap()
                    .iter()
                    .any(|position| targets.contains(&ColdRead::Commit(*position))));
                fixture.source.failures.lock().unwrap().clear();
            }
            let mut transaction = begin_main_transaction(&runtime);
            let record = transaction
                .read_entity(execution, fixture.entity)
                .unwrap()
                .effective()
                .cloned()
                .expect("the recovered record exists");
            let value = super::record_field_values::name_value(&record).unwrap();
            assert_eq!(
                value,
                &worth_foundational::facade::AspectValue::String(
                    worth_foundational::facade::InternedString::Raw("member-after".into())
                )
            );
            assert_eq!(
                medium.offered().len(),
                offers,
                "recovery never offers an artifact"
            );
        }
    }
}
