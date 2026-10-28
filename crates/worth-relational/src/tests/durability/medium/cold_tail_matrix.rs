//! Recovery stages cold admissions without changing the receiving runtime.
use super::cold_read_matrix::FAILURES;
use super::cold_source::ColdRead;
use super::*;
use crate::indexes::data::{
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
};
use crate::storage::data::{RelationalStorageSubject as Subject, RelationalStorageUnavailable};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug)]
enum Input {
    Head,
    History,
    Symbols,
    Buckets,
    Nodes,
    Events,
    Index,
    Envelope,
    TailEnvelope,
}
impl Input {
    const ALL: [Self; 9] = [
        Self::Head,
        Self::History,
        Self::Symbols,
        Self::Buckets,
        Self::Nodes,
        Self::Events,
        Self::Index,
        Self::Envelope,
        Self::TailEnvelope,
    ];
}
struct Fixture {
    recovery: RelationalDurableRecoverySource,
    source: Arc<MemberSource>,
    manifest: crate::durability::data::RelationalCheckpointManifest,
    entity: crate::identity::data::EntityId,
    symbol: crate::symbols::data::Symbol,
    index: DerivedIndexId,
}
fn fixture(execution: &worth_execution::ExecutionRequest<'_, '_>) -> Fixture {
    use super::cold_symbols::view_levels::{admit_population, publish_text, TextMutation};
    let producer = durable_runtime_with_test_schema();
    admit_population(execution, &producer);
    let (entity, _) = publish_text(
        execution,
        &producer,
        TextMutation::Create,
        "paged-view-value-0300",
    );
    let (_, _) = publish_text(
        execution,
        &producer,
        TextMutation::Update(entity),
        "paged-view-value-0600",
    );
    let symbol = producer
        .services
        .symbols
        .interner_snapshot()
        .symbol("paged-view-value-0600")
        .unwrap();
    let index = producer
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "tail-matrix-name".into(),
            branch_scoped: true,
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
        })
        .expect("index fixture has identity capacity")
        .index_id;
    let branch = crate::history::data::BranchId("main".into());
    let built = producer.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: producer.history().branch_head(&branch).unwrap().commit_id,
            branch_id: branch,
            index_ids: vec![index],
        },
    );
    assert!(built.failed_indexes.is_empty());
    create_entity(execution, &producer, "tail-matrix-unrelated-head");
    let (recovery, source, entity) = checkpoint_fixture(execution, producer, entity, true);
    source.damaged.store(false, Ordering::Relaxed);
    #[derive(serde::Deserialize)]
    struct Image {
        manifest: crate::durability::data::RelationalCheckpointManifest,
    }
    let manifest = rmp_serde::from_slice::<Image>(
        recovery
            .checkpoint()
            .unwrap()
            .checkpoint()
            .bytes()
            .strip_prefix(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC)
            .unwrap(),
    )
    .unwrap()
    .manifest;
    Fixture {
        recovery,
        source,
        manifest,
        entity,
        symbol,
        index,
    }
}
fn recover(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut crate::runtime::RelationalRuntime,
    recovery: RelationalDurableRecoverySource,
) -> Result<(), RelationalRecoveryStop> {
    let plan = runtime.durability().recovery_plan_from_source(
        execution,
        recovery,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )?;
    runtime
        .durability_recovery()
        .recover(execution, plan)
        .map(|_| ())
}
#[test]
fn residency_failure_matrix_tail_recovery_stages_every_consumed_subject_before_effects() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for input in Input::ALL {
        for (cause, _) in FAILURES {
            let fixture = fixture(execution);
            let targets: BTreeSet<_> = match input {
                Input::Head | Input::History => fixture
                    .manifest
                    .roots
                    .iter()
                    .flat_map(|root| &root.regions)
                    .chain(&fixture.manifest.mirror)
                    .map(|region| {
                        ColdRead::Member(if matches!(input, Input::Head) {
                            region.head
                        } else {
                            region.history
                        })
                    })
                    .collect(),
                Input::Symbols => fixture
                    .manifest
                    .symbols
                    .iter()
                    .filter(|page| {
                        page.first <= u64::from(fixture.symbol.0)
                            && u64::from(fixture.symbol.0) <= page.last
                    })
                    .map(|page| ColdRead::Member(page.member))
                    .collect(),
                Input::Buckets => fixture
                    .manifest
                    .symbol_buckets
                    .iter()
                    .flat_map(|bucket| bucket.pages.iter().copied())
                    .map(ColdRead::Member)
                    .collect(),
                Input::Nodes => fixture
                    .manifest
                    .lineage
                    .iter()
                    .map(|page| ColdRead::Member(page.member))
                    .collect(),
                Input::Events => fixture
                    .manifest
                    .lineage_events
                    .iter()
                    .map(|page| ColdRead::Member(page.member))
                    .collect(),
                Input::Index => {
                    BTreeSet::from([ColdRead::Member(fixture.manifest.derived_indexes)])
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
            let mut runtime = durable_runtime_with_test_schema();
            let frontier = runtime.history().latest_patch_stream_position();
            let offers = attached_test_medium(&runtime).offered().len();
            let reservations = runtime.patch_position_reservation_counters();
            let result = recover(execution, &mut runtime, fixture.recovery.clone());
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
                        .manifest
                        .symbols
                        .iter()
                        .find(|page| {
                            page.first <= u64::from(fixture.symbol.0)
                                && u64::from(fixture.symbol.0) <= page.last
                        })
                        .unwrap();
                    Subject::Symbols {
                        first: crate::symbols::data::Symbol(page.first.try_into().unwrap()),
                        last: crate::symbols::data::Symbol(page.last.try_into().unwrap()),
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
                    .branch_head(&crate::history::data::BranchId("main".into()))
                    .is_none());
                assert_eq!(runtime.patch_position_reservation_counters(), reservations);
                assert_eq!(attached_test_medium(&runtime).offered().len(), offers);
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
            let mut transaction = test_owner_begin_transaction_for_main(&runtime);
            let record = transaction
                .read_entity(execution, fixture.entity)
                .unwrap()
                .effective()
                .cloned()
                .expect("the recovered record exists");
            let value = crate::visibility::materialization::read_records::entity_query_locus_value(
                &record,
                &aspect_field_locator(aspect_key("name"), field_key("name")),
            )
            .unwrap();
            assert_eq!(
                value,
                &worth_foundational::facade::AspectValue::String(
                    worth_foundational::facade::InternedString::Raw("member-after".into())
                )
            );
            assert_eq!(
                attached_test_medium(&runtime).offered().len(),
                offers,
                "recovery never offers an artifact"
            );
        }
    }
}
