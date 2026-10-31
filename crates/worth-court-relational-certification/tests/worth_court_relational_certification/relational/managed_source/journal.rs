//! Actual journal settlement loses optional metadata despite retained impact postings.
use super::history::{basis, collect, limits, selected};
use super::*;
use worth_relational::facade::history::*;
mod medium;

#[test]
fn managed_history_refuses_journal_settlement_and_decoded_rebuild() {
    let (registry, program) = registry();
    let court = Arc::new(owner::Court::default());
    let rule = CustomInvariantRegistration::new(owner::Reader {
        maximum_work: None,
        program: program.clone(),
        court: court.clone(),
    })
    .unwrap()
    .with_managed_index_sources(vec![program.clone()])
    .unwrap();
    let bytes = Arc::new(medium::Bytes::default());
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .custom_invariant(rule)
        .build_durable(Box::new(medium::Attachment(bytes.clone())))
        .unwrap();
    let _registration = runtime
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(Arc::new(owner::SourceOwner(court)))
        .unwrap();
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    let born = commit(
        &runtime,
        &execution,
        WorkerIntentBatch::new("journal-scope").push(MutationIntent::Create(CreateIntent::Entity(
            EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw("scope"),
                fields: Default::default(),
            },
        ))),
    );
    let scope = born.created_entity(&created(1, "scope")).unwrap();
    let original = basis(&runtime);
    let selector = runtime
        .authored_managed_source_selector(execution, &program, scope)
        .unwrap();
    runtime
        .snapshots()
        .release_snapshot(&born.snapshot)
        .unwrap();
    drop(born);
    let deleted = commit(
        &runtime,
        &execution,
        WorkerIntentBatch::new("journal-delete").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: scope }),
        )),
    );
    let expected = deleted.commit.commit_id;
    let envelope = deleted.publication().envelope.clone();
    let cloned = envelope.as_ref().clone();
    assert_eq!(
        cloned, *envelope,
        "live metadata does not change canonical value equality"
    );
    drop(cloned);
    // The selected head owns its envelope. A later unrelated head makes the
    // deletion historical, so actual holder release can remove its live body.
    let later = commit(
        &runtime,
        &execution,
        WorkerIntentBatch::new("later-head").push(MutationIntent::Create(CreateIntent::Entity(
            EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(4),
                client_key: ClientKey::raw("unrelated-row"),
                fields: Default::default(),
            },
        ))),
    );
    let target = basis(&runtime);
    runtime
        .snapshots()
        .release_snapshot(&later.snapshot)
        .unwrap();
    drop(later);
    let cursor = || {
        runtime
            .authored_managed_source_history_cursor(
                execution,
                &original,
                selected(&runtime, &target),
                std::slice::from_ref(&selector),
            )
            .unwrap()
    };
    let continuation = cursor();
    assert_eq!(collect(&runtime, execution, cursor()), [expected]);
    let weak = Arc::downgrade(&envelope);
    drop(envelope);
    runtime
        .snapshots()
        .release_snapshot(&deleted.snapshot)
        .unwrap();
    drop(deleted);
    assert!(runtime.run_index_generation_reclamation_pass().is_some());
    assert!(
        weak.upgrade().is_none(),
        "settled historical journal body has no remaining reader"
    );
    let reads = bytes.reads.lock().unwrap().len();
    for _ in 0..2 {
        assert!(
            matches!(
                continuation.prepare_next_page(&runtime, execution, limits()),
                Err(RelationalAuthoredWriteDenial::IncompleteManagedSourceHistory)
            ),
            "page refusal cannot advance or infer an empty interval"
        );
    }
    assert!(
        bytes.reads.lock().unwrap().len() > reads,
        "Native reacquires actual canonical bytes before rejecting missing live metadata"
    );
    runtime
        .discard_authored_write_generation(execution, selected(&runtime, &target))
        .unwrap();
    runtime
        .rebuild_authored_write_generation(execution, selected(&runtime, &target))
        .unwrap();
    assert!(matches!(
        runtime.authored_managed_source_history_cursor(
            execution,
            &original,
            selected(&runtime, &target),
            &[selector]
        ),
        Err(RelationalAuthoredWriteDenial::IncompleteManagedSourceHistory)
    ));
    let ordinary = runtime
        .authored_write_history_cursor(
            execution,
            &original,
            selected(&runtime, &target),
            &[RelationalAuthoredWriteSelector::Record {
                record: RecordRef::Entity(scope),
                scope: RelationalAuthoredWriteScope::Lifecycle,
            }],
        )
        .unwrap();
    assert_eq!(collect(&runtime, execution, ordinary), [expected]);
}
