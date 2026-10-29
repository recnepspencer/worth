#[allow(unused_imports)]
pub(crate) use {worth_execution::ExecutionAllocationPolicy as AllocationPolicy, worth_relational::facade::transactions::AspectFieldPatch, worth_relational::facade::history::BranchId, worth_execution::CancellationToken, worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause, worth_relational::facade::symbols::ClientKey, worth_relational::facade::transactions::CommitExecutionDenialKind, worth_relational::facade::transactions::CreateIntent, worth_relational::facade::indexes::DerivedIndexBuildRequest, worth_relational::facade::indexes::DerivedIndexDefinition, worth_relational::facade::indexes::DerivedIndexExecutionDenialKind, worth_relational::facade::indexes::DerivedIndexId, worth_relational::facade::indexes::DerivedIndexKind, worth_foundational::DeterminismContract, worth_relational::facade::transactions::EntitySpec, worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy, worth_relational::facade::identity::KindId, worth_relational::facade::transactions::MutationIntent, std::num::NonZeroUsize, worth_relational::facade::identity::PartitionId, worth_relational::facade::runtime::RelationalRuntimeApi, worth_relational::facade::mvcc::RelationalTransactionIntent, worth_relational::facade::transactions::TransactionCommitError, worth_relational::facade::transactions::WorkerIntentBatch, crate::leased_preparation::ordinary_context::authority, crate::leased_preparation::ordinary_context::lease_request, crate::leased_preparation::ordinary_context::lease_request_with_work, crate::leased_preparation::ordinary_context::leased_preparation_index_budget, crate::leased_preparation::ordinary_context::serial, crate::leased_preparation::ordinary_context::support};







#[test]
fn commit_preparation_spends_validation_and_later_packet_work_from_one_lease() {
    if worth_testkit_relational::test_process::reexecute_isolated_test() {
        return;
    }

    let _serial = serial();
    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(crate::leased_preparation::ordinary_context::support::demo_schema_registry())
        .build()
        .expect("valid native runtime configuration");

    let attempt = |work_ceiling: u64, validation_only: bool, publish: bool| {
        let lease = authority()
            .request_lease(lease_request_with_work(
                4 * 1024 * 1024,
                worth_execution::CancellationToken::new(),
                work_ceiling,
            ))
            .expect("work ceiling is within the authority's limits");
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .expect("main basis remains unchanged");
        let mut transaction = runtime
            .begin_branch_transaction(&basis, worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary())
            .expect("basis admits transaction");
        transaction
            .push_batch(
                worth_relational::facade::transactions::WorkerIntentBatch::new("cumulative-work").push(worth_relational::facade::transactions::MutationIntent::Create(
                    worth_relational::facade::transactions::CreateIntent::Entity(EntitySpec {
                        partition_id: worth_relational::facade::identity::PartitionId::main(),
                        kind_id: KindId(1),
                        client_key: worth_relational::facade::symbols::ClientKey::raw("cumulative-work"),
                        fields: worth_relational::facade::transactions::AspectFieldPatch::default(),
                    }),
                )),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("entity intent stages");
        if validation_only {
            runtime
                .validate_branch_transaction(
                    &worth_execution::ExecutionRequest::leased(&lease),
                    transaction,
                )
                .map(|_| ())
        } else if publish {
            runtime
                .commit_branch_transaction(
                    &worth_execution::ExecutionRequest::leased(&lease),
                    transaction,
                )
                .map(|_| ())
        } else {
            runtime
                .prepare_branch_transaction(
                    &worth_execution::ExecutionRequest::leased(&lease),
                    transaction,
                )
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
        attempt(admitted, validation_only, false).expect("generous lease succeeds");
        while denied + 1 < admitted {
            let trial = denied + (admitted - denied) / 2;
            match attempt(trial, validation_only, false) {
                Ok(()) => admitted = trial,
                Err(worth_relational::facade::transactions::TransactionCommitError::Execution { denial, .. })
                    if denial.kind == worth_relational::facade::transactions::CommitExecutionDenialKind::Cause(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted) =>
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
    attempt(validation_work, true, false).expect("validation fits its minimum work lease");
    assert!(matches!(
        attempt(validation_work, false, false),
        Err(worth_relational::facade::transactions::TransactionCommitError::Execution {
            denial: worth_relational::facade::transactions::CommitExecutionDenial {
                kind: worth_relational::facade::transactions::CommitExecutionDenialKind::Cause(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted),
                ..
            },
            ..
        })
    ));
    let head_before = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .0;
    // The same one-entity validation ceiling cannot fund later packet work
    // when the ordinary commit also attempts publication.
    assert!(matches!(
        attempt(validation_work, false, true),
        Err(worth_relational::facade::transactions::TransactionCommitError::Execution {
            denial: worth_relational::facade::transactions::CommitExecutionDenial {
                kind: worth_relational::facade::transactions::CommitExecutionDenialKind::Cause(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted),
                ..
            },
            ..
        })
    ));
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .0,
        head_before
    );
}

#[test]
fn leased_index_and_commit_preparation_preserve_parity_and_stop_before_publication() {
    if worth_testkit_relational::test_process::reexecute_isolated_test() {
        return;
    }

    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let _serial = serial();
    let authority = authority();
    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(crate::leased_preparation::ordinary_context::support::demo_schema_registry())
        .build()
        .expect("valid native runtime configuration");
    let (created, _) = crate::leased_preparation::ordinary_context::support::create_entity(execution, &runtime, "leased-seed");
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "leased.entity.name".to_owned(),
            kind: worth_relational::facade::indexes::DerivedIndexKind::EntityField {
                field_locator: crate::leased_preparation::ordinary_context::support::aspect_field_locator("name"),
            },
            branch_scoped: true,
        })
        .expect("index fixture has identity capacity");
    let request = DerivedIndexBuildRequest {
        source_commit_id: created.commit.commit_id,
        branch_id: BranchId("main".to_owned()),
        index_ids: vec![index.index_id],
    };
    let serial = runtime
        .index_authority()
        .build_for_commit(execution, request.clone());
    assert!(serial.execution_denial.is_none());
    let lease = authority
        .request_lease(lease_request(32 * 1024 * 1024, worth_execution::CancellationToken::new()))
        .expect("generous lease is admitted");
    let parallel = runtime.index_authority().build_for_commit(
        &worth_execution::ExecutionRequest::leased(&lease),
        request.clone(),
    );
    assert!(parallel.execution_denial.is_none());
    assert_eq!(serial.generations.len(), 1);
    assert_eq!(
        serial.generations[0].entries,
        parallel.generations[0].entries
    );

    crate::leased_preparation::ordinary_context::leased_preparation_index_budget::assert_prep_and_map_share_work(&runtime, &request, authority);

    let latest_generation = parallel.generations[0].generation_id;
    let cancelled = worth_execution::CancellationSource::new();
    cancelled.cancel();
    let cancelled_lease = authority
        .request_lease(lease_request(32 * 1024 * 1024, cancelled.token()))
        .expect("cancelled request still receives a lease for typed execution stop");
    let stopped_index = runtime.index_authority().build_for_commit(
        &worth_execution::ExecutionRequest::leased(&cancelled_lease),
        request.clone(),
    );
    assert_eq!(
        stopped_index.execution_denial.map(|denial| denial.kind),
        Some(worth_relational::facade::indexes::DerivedIndexExecutionDenialKind::Cause(worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled))
    );
    assert!(stopped_index.generations.is_empty());
    assert_eq!(
        runtime
            .index_access()
            .read_latest_generation(execution, index.index_id, &request.branch_id)
            .unwrap()
            .expect("earlier successful generation remains")
            .generation_id,
        latest_generation
    );

    // A zero-memory, single-worker policy refuses the first reservation;
    // packet construction and its scratch/result ceilings cannot run first.
    let mut small_request = lease_request(0, worth_execution::CancellationToken::new());
    small_request.policy = worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Automatic,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(0), Some(1_000_000)),
    );
    let small_lease = authority
        .request_lease(small_request)
        .expect("zero-memory policy is admitted");
    let exhausted_index = runtime.index_authority().build_for_commit(
        &worth_execution::ExecutionRequest::leased(&small_lease),
        request,
    );
    let Some(worth_relational::facade::indexes::DerivedIndexExecutionDenialKind::Cause(worth_relational::facade::transactions::RelationalExecutionDenialCause::PolicyMemoryExhausted {
        requested,
        admitted: 0,
        ancestor: 0,
    })) = exhausted_index.execution_denial.map(|denial| denial.kind)
    else {
        panic!("the first reservation must be refused by this request's policy");
    };
    assert!(requested > 0);
    assert!(exhausted_index.generations.is_empty());

    let identity = runtime.main_branch_identity();
    let (before, basis) = runtime
        .observe_branch(&identity)
        .expect("main branch is current");
    let mut transaction = runtime
        .begin_branch_transaction(&basis, worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary())
        .expect("current basis admits a transaction");
    transaction
        .push_batch(
            worth_relational::facade::transactions::WorkerIntentBatch::new("cancelled-prepare").push(worth_relational::facade::transactions::MutationIntent::Create(
                worth_relational::facade::transactions::CreateIntent::Entity(EntitySpec {
                    partition_id: worth_relational::facade::identity::PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: worth_relational::facade::symbols::ClientKey::raw("cancelled-prepare"),
                    fields: worth_relational::facade::transactions::AspectFieldPatch::default(),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("entity intent stages");
    let error = runtime
        .prepare_branch_transaction(
            &worth_execution::ExecutionRequest::leased(&cancelled_lease),
            transaction,
        )
        .expect_err("cancelled lease prevents preparation");
    assert!(matches!(
        error,
        worth_relational::facade::transactions::TransactionCommitError::Execution {
            denial: worth_relational::facade::transactions::CommitExecutionDenial {
                kind: worth_relational::facade::transactions::CommitExecutionDenialKind::Cause(worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled),
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

#[path = "bulk_creation/mod.rs"]
pub(crate) mod bulk_creation;
#[path = "leased_preparation_denial_causes/mod.rs"]
pub(crate) mod leased_preparation_denial_causes;
#[path = "leased_preparation_index_request_memory/mod.rs"]
pub(crate) mod leased_preparation_index_request_memory;
#[path = "preimage/mod.rs"]
pub(crate) mod preimage;
