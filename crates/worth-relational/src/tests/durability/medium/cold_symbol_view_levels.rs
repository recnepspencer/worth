//! Requested symbol closures stay local even when another fork warmed Full.
use super::*;
use crate::storage::residency::{probe_full, RelationalResidencyDemandFailure};
use crate::symbols::data::Symbol;
use crate::transactions::data::{EntityMutationIntent, UpdateEntityFieldsIntent};
use worth_execution::ExecutionImmutableBytes;
use worth_foundational::facade::{AspectValue, InternedString};

pub(in super::super) struct ViewFixture {
    pub(in super::super) runtime: crate::runtime::RelationalRuntime,
    pub(in super::super) source: Arc<MemberSource>,
    pub(in super::super) entity: crate::identity::data::EntityId,
    pub(in super::super) historical: Symbol,
    pub(in super::super) current: Symbol,
    pub(in super::super) historical_page: RelationalDurableMemberLocator,
}

#[test]
fn head_symbols_never_cover_full_history_or_inherit_its_page_damage() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for damage in [false, true] {
        let fixture = reopened(execution);
        let mut transaction = test_owner_begin_transaction_for_main(&fixture.runtime);
        let selected = fixture
            .runtime
            .selected_branch_state(&transaction.basis)
            .unwrap();
        let source = selected.with_symbol_admission(&fixture.runtime.services.symbols);
        let frontier = fixture.runtime.history().latest_patch_stream_position();
        if damage {
            *fixture.source.failed_member.lock().unwrap() = Some(fixture.historical_page);
        }
        let current = transaction
            .read_entity(execution, fixture.entity)
            .unwrap()
            .effective()
            .cloned()
            .expect("a Head miss cannot escape as an absent record");
        assert_eq!(current.entity_id, fixture.entity);
        let value = crate::visibility::materialization::read_records::entity_query_locus_value(
            &current,
            &aspect_field_locator(aspect_key("name"), field_key("name")),
        )
        .unwrap();
        assert!(
            matches!(value, AspectValue::String(InternedString::Symbol(symbol)) if symbol.0 == fixture.current.0)
        );
        let symbols = fixture.runtime.services.symbols.interner_snapshot();
        assert_eq!(
            symbols.resolve(fixture.current),
            Some("paged-view-value-0600")
        );
        assert!(
            symbols.resolve(fixture.historical).is_none(),
            "Head must not admit the history-only page of a physically Full cell"
        );
        assert!(
            fixture.runtime.services.symbols.snapshot().entries.len() < 400,
            "one current closure has ample headroom below the 601 admitted texts"
        );
        let read_full = || {
            probe_full(execution, &source, |view| {
                view.get_partition(fixture.entity.partition_id)
                    .map(|partition| {
                        partition
                            .entity_arena
                            .metadata_history_at(fixture.entity.slot_index())
                            .unwrap()
                            .len()
                    })
            })
        };
        if damage {
            let RelationalResidencyDemandFailure::StorageUnavailable(failure) =
                read_full().unwrap_err()
            else {
                panic!("the selected bad symbol page must preserve its storage failure");
            };
            assert!(
                matches!(failure.subject, RelationalStorageSubject::Symbols { first, last }
                if first <= fixture.historical && fixture.historical <= last)
            );
            assert_eq!(failure.cause, Cause::Damaged);
            *fixture.source.failed_member.lock().unwrap() = None;
            let reads = fixture.source.reads.load(Ordering::Relaxed);
            assert!(
                matches!(read_full(), Err(RelationalResidencyDemandFailure::StorageUnavailable(retry)) if retry == failure)
            );
            let again = transaction
                .read_entity(execution, fixture.entity)
                .unwrap()
                .effective()
                .cloned()
                .unwrap();
            assert_eq!(again, current);
            assert_eq!(fixture.source.reads.load(Ordering::Relaxed), reads);
        } else {
            assert!(
                read_full().unwrap().unwrap() > 1,
                "Full must retain both real committed field versions"
            );
            assert_eq!(
                fixture
                    .runtime
                    .services
                    .symbols
                    .interner_snapshot()
                    .resolve(fixture.historical),
                Some("paged-view-value-0300")
            );
        }
        if !damage {
            let covered = fixture.runtime.fork().unwrap();
            let transaction = test_owner_begin_transaction_for_main(&covered);
            let selected = covered.selected_branch_state(&transaction.basis).unwrap();
            let source = selected.with_symbol_admission(&covered.services.symbols);
            assert!(probe_full(execution, &source, |view| view
                .get_partition(fixture.entity.partition_id)
                .is_some())
            .unwrap());
            assert!(
                matches!(
                    selected.state().root().probe_head_region_with_symbols(
                        fixture.entity.partition_id,
                        &covered.services.symbols,
                    ),
                    crate::storage::residency::PartitionHeadProbe::Resident(_)
                ),
                "a Full symbol guard must cover Head without recollecting the view"
            );
        }
        assert_eq!(
            fixture.runtime.history().latest_patch_stream_position(),
            frontier
        );
    }
}

pub(in super::super) fn reopened(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> ViewFixture {
    let producer = durable_runtime_with_test_schema();
    let (_, basis) = producer
        .observe_branch(&producer.main_branch_identity())
        .unwrap();
    for index in 0..=600 {
        let text = format!("paged-view-value-{index:04}");
        let input = ExecutionImmutableBytes::from_external_bytes(Arc::new(text.as_bytes().into()));
        producer
            .admit_field_text(execution, &basis, &input)
            .unwrap();
    }
    drop(basis);
    let (entity, _retained_before_update) = publish_text(
        execution,
        &producer,
        TextMutation::Create,
        "paged-view-value-0300",
    );
    let (_, _retained_after_update) = publish_text(
        execution,
        &producer,
        TextMutation::Update(entity),
        "paged-view-value-0600",
    );
    let symbols = producer.services.symbols.interner_snapshot();
    let historical = symbols.symbol("paged-view-value-0300").unwrap();
    let current = symbols.symbol("paged-view-value-0600").unwrap();
    let mirror = producer.acquire_partition_edition();
    assert!(
        mirror
            .partition(entity.partition_id)
            .unwrap()
            .entity_arena
            .metadata_history_at(entity.slot_index())
            .unwrap()
            .len()
            > 1,
        "the retained producer snapshot must keep real historical mirror content"
    );
    drop(mirror);
    let (recovery, source, _) = fixture_from_runtime(execution, producer, false, 0);
    #[derive(serde::Deserialize)]
    struct Input {
        manifest: crate::durability::data::RelationalCheckpointManifest,
    }
    let input: Input = rmp_serde::from_slice(
        recovery
            .checkpoint()
            .unwrap()
            .checkpoint()
            .bytes()
            .strip_prefix(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC)
            .unwrap(),
    )
    .unwrap();
    let page = |symbol: Symbol| {
        input
            .manifest
            .symbols
            .iter()
            .find(|page| page.first <= symbol.0 as u64 && symbol.0 as u64 <= page.last)
            .unwrap()
            .member
    };
    let historical_page = page(historical);
    assert_ne!(
        historical_page,
        page(current),
        "fixture values must occupy different real pages"
    );
    source.damaged.store(false, Ordering::Relaxed);
    let mut receiving = durable_runtime_with_test_schema();
    let plan = receiving
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    receiving
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(
        source.reads.load(Ordering::Relaxed),
        0,
        "reopen reads no dictionary or region member"
    );
    let warmed = receiving.fork().unwrap();
    let transaction = test_owner_begin_transaction_for_main(&warmed);
    let selected = warmed.selected_branch_state(&transaction.basis).unwrap();
    selected
        .state()
        .root()
        .demand_all_full(execution, &warmed.services.symbols)
        .unwrap();
    assert!(receiving
        .services
        .symbols
        .interner_snapshot()
        .resolve(historical)
        .is_none());
    ViewFixture {
        runtime: receiving,
        source,
        entity,
        historical,
        current,
        historical_page,
    }
}

enum TextMutation {
    Create,
    Update(crate::identity::data::EntityId),
}

fn publish_text(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    mutation: TextMutation,
    text: &str,
) -> (
    crate::identity::data::EntityId,
    crate::snapshots::data::SnapshotHandle,
) {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let input = ExecutionImmutableBytes::from_external_bytes(Arc::new(text.as_bytes().into()));
    let text = runtime.admit_field_text(execution, &basis, &input).unwrap();
    let fields = AspectFieldPatch::from_locator(
        crate::transactions::data::planned_single_field_locator(
            aspect_key("name"),
            field_key("name"),
        ),
        text.field_value_for(&basis).unwrap(),
    );
    let batch = match mutation {
        TextMutation::Create => {
            let mut batch = batch_create("symbol-view-record");
            let MutationIntent::Create(CreateIntent::Entity(entity)) = &mut batch.intents[0] else {
                panic!("producer declares one entity create");
            };
            entity.fields = fields;
            batch
        }
        TextMutation::Update(entity_id) => {
            WorkerIntentBatch::new("symbol-view-update").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent { entity_id, fields }),
            ))
        }
    };
    let mut transaction = runtime
        .begin_branch_transaction(&basis, crate::mvcc::RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    let outcome = transaction.commit(execution, runtime).unwrap();
    let entity = changed_entities(&outcome)[0];
    (entity, outcome.snapshot.clone())
}
