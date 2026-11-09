//! Warm public point/projection reads share one record's untouched aspect state.
use crate::leased_preparation::ordinary_context::support::{aspect_key, field_key};
use worth_execution::{
    CancellationToken, ExecutionAllocationPolicy, ExecutionRequest, ExecutionResourceLease,
    ExecutionWorkCeiling, LeaseRequest,
};
use worth_foundational::facade::AspectValue;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    config::CascadeDeletePolicy,
    identity::{KindId, PartitionId},
    schema::DeclaredAspectContractBinding,
    symbols::ClientKey,
    transactions::{
        AspectFieldPatch, CreateIntent, EntitySpec, MutationIntent, RecordRef, WorkerIntentBatch,
    },
};
use worth_testkit_relational::aspect_storage::{
    aspect_field_locator, definition::entity_field_aspect, AspectSchemaFixture,
};
fn exercise(aspects: usize) -> (u64, u64) {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let setup = ExecutionRequest::serial(&host);
    let mut schema = AspectSchemaFixture::with_default_declared_aspects(
        CascadeDeletePolicy::CascadeDeleteRelations,
    );
    for ordinal in 0..aspects {
        let name = format!("other-{ordinal:05}");
        let (binding, contract) = entity_field_aspect(aspect_key(&name), field_key(&name));
        schema
            .entity_aspects
            .push(DeclaredAspectContractBinding { binding, contract });
    }
    let runtime = schema.build_runtime();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    let locators = std::iter::once("name".to_owned())
        .chain((0..aspects).map(|ordinal| format!("other-{ordinal:05}")))
        .map(|name| aspect_field_locator(aspect_key(&name), field_key(&name)));
    transaction
        .push_batch(
            WorkerIntentBatch::new("immutable-record-read").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw("row"),
                    fields: AspectFieldPatch::new(
                        locators
                            .map(|locator| {
                                (locator, AspectValue::String("immutable-payload".into()))
                            })
                            .collect(),
                    ),
                }),
            )),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let committed = transaction.commit(&setup, &runtime).unwrap();
    let entity = committed
        .changed_records
        .iter()
        .find_map(|record| match record {
            RecordRef::Entity(id) => Some(*id),
            _ => None,
        })
        .unwrap();
    let pin = runtime
        .read_truth()
        .pin_read_capability(&committed.snapshot, &[], &setup)
        .unwrap();
    let read = |execution: &ExecutionRequest<'_, '_>| {
        let point = runtime
            .read_truth()
            .visible_entity_at_version(execution, entity, committed.commit.version_id)
            .unwrap()
            .unwrap();
        assert!(
            point
                .authoritative_aspect_state
                .as_ref()
                .unwrap()
                .aspects()
                .len()
                >= aspects
        );
        let projected = pin
            .project_read()
            .bounded_entities_of_kind(execution, KindId(1), 2)
            .unwrap()
            .unwrap()
            .into_records();
        assert_eq!(projected.len(), 1);
        assert_eq!(point.entity_id, projected[0].entity_id);
        (point, projected)
    };
    // Cold admission is outside this warm clone boundary, under its real setup request.
    drop(read(&setup));
    let lease = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                Some(1 << 30),
                Some(100_000_000),
            ),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    })
    .unwrap();
    let execution = ExecutionRequest::leased(&lease);
    let ((point, projected), report) = execution
        .run(ExecutionWorkCeiling::new(None), |_| read(&execution))
        .unwrap();
    assert_eq!(
        point.authoritative_aspect_state,
        projected[0].authoritative_aspect_state
    );
    runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    (
        report.charged_work(),
        report.physical().peak_charged_memory_bytes(),
    )
}
#[test]
fn fixed_record_reads_do_not_requote_the_unchanged_aspect_population() {
    let small = exercise(512);
    let large = exercise(1024);
    eprintln!(
        "SDK shared record reads: work {}->{}, peak clone memory {}->{}",
        small.0, large.0, small.1, large.1
    );
    for (phase, before, after) in [
        ("shared record read work", small.0, large.0),
        ("shared record clone memory", small.1, large.1),
    ] {
        worth_testkit_relational::maintenance::assert_change_bounded(
            phase,
            "unchanged aspects",
            before.max(1),
            after.max(1),
        );
    }
}
