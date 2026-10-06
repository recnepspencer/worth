#[path = "leased_preparation/index_budget.rs"]
mod leased_preparation_index_budget;
#[path = "../examples/support.rs"]
mod support;

use std::{
    num::NonZeroUsize,
    sync::{Mutex, OnceLock},
};

use worth_execution::{
    CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig, LeaseRequest,
};
use worth_foundational::facade::AspectValue;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    history::BranchId,
    identity::{KindId, PartitionId},
    indexes::{
        DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexExecutionDenialKind,
        DerivedIndexId, DerivedIndexKind,
    },
    mvcc::RelationalTransactionIntent,
    runtime::RelationalRuntimeApi,
    symbols::ClientKey,
    transactions::{
        AspectFieldPatch, BulkEntityCreateIntent, CommitExecutionDenialKind, CreateIntent,
        EntitySpec, MutationIntent, TransactionCommitError, WorkerIntentBatch,
    },
};

static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
static TEST_SERIAL: Mutex<()> = Mutex::new(());

fn authority() -> &'static ExecutionAuthority {
    AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: 64 * 1024 * 1024,
        })
        .expect("one authority for this integration binary")
    })
}

fn lease_request(memory: u64, cancellation: CancellationToken) -> LeaseRequest {
    lease_request_with_work(memory, cancellation, 1_000_000)
}

fn lease_request_with_work(
    memory: u64,
    cancellation: CancellationToken,
    work_ceiling: u64,
) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(4).unwrap(), memory, work_ceiling),
        ),
        deadline: None,
        cancellation,
    }
}

#[test]
fn commit_preparation_spends_validation_and_later_packet_work_from_one_lease() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(support::demo_schema_registry())
        .build();

    let attempt = |work_ceiling: u64, validation_only: bool| {
        let lease = authority()
            .request_lease(lease_request_with_work(
                4 * 1024 * 1024,
                CancellationToken::new(),
                work_ceiling,
            ))
            .expect("work ceiling is within the authority's limits");
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .expect("main basis remains unchanged");
        let mut transaction = runtime
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .expect("basis admits transaction");
        transaction
            .push_batch(
                WorkerIntentBatch::new("cumulative-work").push(MutationIntent::Create(
                    CreateIntent::Entity(EntitySpec {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(1),
                        client_key: ClientKey::raw("cumulative-work"),
                        fields: AspectFieldPatch::default(),
                    }),
                )),
            )
            .expect("entity intent stages");
        if validation_only {
            runtime
                .validate_branch_transaction_with_lease(transaction, &lease)
                .map(|_| ())
        } else {
            runtime
                .prepare_branch_transaction_with_lease(transaction, &lease)
                .map(|candidate| {
                    runtime
                        .preparation_port()
                        .discard_prepared_candidate(candidate)
                        .expect("unused successful candidate is discarded");
                })
        }
    };

    let minimum_work = |validation_only| {
        let mut denied = 0;
        let mut admitted = 1_000_000;
        attempt(admitted, validation_only).expect("generous lease succeeds");
        while denied + 1 < admitted {
            let trial = denied + (admitted - denied) / 2;
            match attempt(trial, validation_only) {
                Ok(()) => admitted = trial,
                Err(TransactionCommitError::Execution { denial, .. })
                    if denial.kind == CommitExecutionDenialKind::WorkExhausted =>
                {
                    denied = trial;
                }
                Err(error) => panic!("unexpected preparation stop (validation_only={validation_only}) at work {trial}: {error:?}"),
            }
        }
        admitted
    };

    let validation_work = minimum_work(true);
    let full_preparation_work = minimum_work(false);
    assert!(
        full_preparation_work > validation_work,
        "later packet maps must spend work beyond validation"
    );
    attempt(validation_work, true).expect("validation fits its minimum work lease");
    assert!(matches!(
        attempt(validation_work, false),
        Err(TransactionCommitError::Execution {
            denial: worth_relational::facade::transactions::CommitExecutionDenial {
                kind: CommitExecutionDenialKind::WorkExhausted,
                ..
            },
            ..
        })
    ));
}

#[test]
fn leased_index_and_commit_preparation_preserve_parity_and_stop_before_publication() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let authority = authority();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(support::demo_schema_registry())
        .build();
    let (created, _) = support::create_entity(&runtime, "leased-seed");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "leased.entity.name".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: support::aspect_field_locator("name"),
        },
        branch_scoped: true,
    });
    let request = DerivedIndexBuildRequest {
        source_commit_id: created.commit.commit_id,
        branch_id: BranchId("main".to_owned()),
        index_ids: vec![index.index_id],
    };
    let serial = runtime.index_authority().build_for_commit(request.clone());
    assert!(serial.execution_denial.is_none());
    let lease = authority
        .request_lease(lease_request(32 * 1024 * 1024, CancellationToken::new()))
        .expect("generous lease is admitted");
    let parallel = runtime
        .index_authority()
        .build_for_commit_with_lease(request.clone(), &lease);
    assert!(parallel.execution_denial.is_none());
    assert_eq!(serial.generations.len(), 1);
    assert_eq!(
        serial.generations[0].entries,
        parallel.generations[0].entries
    );

    leased_preparation_index_budget::assert_prep_and_map_share_work(&runtime, &request, authority);

    let latest_generation = parallel.generations[0].generation_id;
    let cancelled = worth_execution::CancellationSource::new();
    cancelled.cancel();
    let cancelled_lease = authority
        .request_lease(lease_request(32 * 1024 * 1024, cancelled.token()))
        .expect("cancelled request still receives a lease for typed execution stop");
    let stopped_index = runtime
        .index_authority()
        .build_for_commit_with_lease(request.clone(), &cancelled_lease);
    assert_eq!(
        stopped_index.execution_denial.map(|denial| denial.kind),
        Some(DerivedIndexExecutionDenialKind::Cancelled)
    );
    assert!(stopped_index.generations.is_empty());
    assert_eq!(
        runtime
            .index_access()
            .latest_generation(index.index_id, &request.branch_id)
            .expect("earlier successful generation remains")
            .generation_id,
        latest_generation
    );

    let small_lease = authority
        .request_lease(lease_request(256, CancellationToken::new()))
        .expect("small lease is within authority cap");
    let exhausted_index = runtime
        .index_authority()
        .build_for_commit_with_lease(request, &small_lease);
    assert!(matches!(
        exhausted_index.execution_denial.map(|denial| denial.kind),
        Some(
            DerivedIndexExecutionDenialKind::ResourceExhausted
                | DerivedIndexExecutionDenialKind::ResultCapacityExceeded
        )
    ));
    assert!(exhausted_index.generations.is_empty());

    let identity = runtime.main_branch_identity();
    let (before, basis) = runtime
        .observe_branch(&identity)
        .expect("main branch is current");
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .expect("current basis admits a transaction");
    transaction
        .push_batch(
            WorkerIntentBatch::new("cancelled-prepare").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw("cancelled-prepare"),
                    fields: AspectFieldPatch::default(),
                }),
            )),
        )
        .expect("entity intent stages");
    let error = runtime
        .prepare_branch_transaction_with_lease(transaction, &cancelled_lease)
        .expect_err("cancelled lease prevents preparation");
    assert!(matches!(
        error,
        TransactionCommitError::Execution {
            denial: worth_relational::facade::transactions::CommitExecutionDenial {
                kind: CommitExecutionDenialKind::Cancelled,
                ..
            },
            ..
        }
    ));
    assert_eq!(
        runtime
            .observe_branch(&identity)
            .expect("main remains observable")
            .0,
        before
    );
}

#[test]
fn leased_bulk_creation_matches_serial_canonical_patch() {
    let _serial = TEST_SERIAL.lock().unwrap();
    fn commit_bulk(
        lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    ) -> worth_relational::facade::transactions::CommitResult {
        let runtime = RelationalRuntimeApi::builder()
            .schema_registry(support::demo_schema_registry())
            .build();
        let identity = runtime.main_branch_identity();
        let (_, basis) = runtime.observe_branch(&identity).expect("main basis");
        let mut transaction = runtime
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .expect("admitted basis");
        let keys = (0..64)
            .map(|index| ClientKey::raw(format!("bulk-{index}")))
            .collect();
        let fields = (0..64)
            .map(|index| {
                AspectFieldPatch::from_locator(
                    support::aspect_field_locator("name"),
                    AspectValue::String(format!("bulk-{index}").into()),
                )
            })
            .collect();
        transaction
            .push_batch(
                WorkerIntentBatch::new("leased-bulk").push(MutationIntent::Create(
                    CreateIntent::BulkEntities(BulkEntityCreateIntent {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(1),
                        client_keys: keys,
                        field_patches: fields,
                    }),
                )),
            )
            .expect("bulk transaction stages");
        match lease {
            Some(lease) => runtime.commit_branch_transaction_with_lease(transaction, lease),
            None => runtime.commit_branch_transaction(transaction),
        }
        .expect("bulk commit succeeds")
    }

    let serial = commit_bulk(None);
    let lease = authority()
        .request_lease(lease_request(32 * 1024 * 1024, CancellationToken::new()))
        .expect("bulk lease admitted");
    let leased = commit_bulk(Some(&lease));
    assert_eq!(serial.changed_records.len(), 64);
    assert_eq!(serial.changed_records, leased.changed_records);
    assert_eq!(serial.patch(), leased.patch());
}

#[test]
fn one_large_index_packet_stops_at_its_declared_ceiling_without_publication() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(support::demo_schema_registry())
        .build();
    let identity = runtime.main_branch_identity();
    let (_, basis) = runtime.observe_branch(&identity).expect("main basis");
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .expect("admitted basis");
    let client_keys = (0..128)
        .map(|index| ClientKey::raw(format!("index-heavy-{index}")))
        .collect();
    let field_patches = (0..128)
        .map(|index| {
            AspectFieldPatch::from_locator(
                support::aspect_field_locator("name"),
                AspectValue::String(format!("index-heavy-{index}").into()),
            )
        })
        .collect();
    transaction
        .push_batch(
            WorkerIntentBatch::new("large-index-source").push(MutationIntent::Create(
                CreateIntent::BulkEntities(BulkEntityCreateIntent {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_keys,
                    field_patches,
                }),
            )),
        )
        .expect("bulk source stages");
    let committed = runtime
        .commit_branch_transaction(transaction)
        .expect("source commits");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(0),
        name: "large.index.name".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: support::aspect_field_locator("name"),
        },
        branch_scoped: true,
    });
    let request = DerivedIndexBuildRequest {
        source_commit_id: committed.commit.commit_id,
        branch_id: BranchId("main".to_owned()),
        index_ids: vec![index.index_id],
    };
    let serial = runtime.index_authority().build_for_commit(request.clone());
    assert!(serial.execution_denial.is_none());
    let published = serial.generations[0].generation_id;
    let lease = authority()
        .request_lease(lease_request(64 * 1024, CancellationToken::new()))
        .expect("tight lease is admitted");
    let stopped = runtime
        .index_authority()
        .build_for_commit_with_lease(request.clone(), &lease);
    assert!(matches!(
        stopped.execution_denial.map(|denial| denial.kind),
        Some(
            DerivedIndexExecutionDenialKind::ResourceExhausted
                | DerivedIndexExecutionDenialKind::ResultCapacityExceeded
        )
    ));
    assert!(stopped.generations.is_empty());
    assert_eq!(
        runtime
            .index_access()
            .latest_generation(index.index_id, &request.branch_id)
            .expect("prior index remains")
            .generation_id,
        published
    );
}
