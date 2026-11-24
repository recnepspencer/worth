use super::*;
use worth_relational::facade::durability::RelationalRecordedComponentAdmissionCause as Cause;

#[test]
fn recorded_component_recovery_admits_original_empty_and_historical_tail_subjects() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let empty = current(execution, &runtime);
    assert!(matches!(
        empty.target(),
        RecordedRelationalComponentTarget::Empty
    ));
    create_entity(execution, &runtime, "original-product-predecessor");
    let predecessor = current(execution, &runtime);
    create_entity(execution, &runtime, "unpublished-native-successor");
    let successor = current(execution, &runtime);
    create_entity(execution, &runtime, "original-world-genesis");
    let genesis = current(execution, &runtime);
    create_entity(execution, &runtime, "later-writer-head");
    let later = current(execution, &runtime);
    let offers = medium.offered().len();
    drop(runtime);
    let mut recovered = durable_runtime(&medium);
    let plan = recovered
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let (_, authority) = recovered
        .durability_recovery()
        .recover_with_authority(execution, plan)
        .unwrap();
    for recorded in [&empty, &predecessor, &successor, &genesis, &later] {
        let basis = authority
            .admit_recorded_component(execution, &recovered, recorded)
            .unwrap()
            .into_basis();
        assert_recorded_basis(&recovered, &basis, recorded);
        assert_eq!(
            &recovered
                .owner_component_services()
                .basis_port()
                .export_recorded_retained_component(execution, &basis)
                .unwrap(),
            recorded,
            "recovery re-exports the original historical subject without replacing it with the head"
        );
        assert_ne!(
            basis.descriptor().runtime_instance_id(),
            recorded.descriptor().runtime_instance_id()
        );
        let lease = recovered.retain_component_basis(&basis).unwrap();
        let retained = recovered
            .readmit_retained_branch_basis(basis.descriptor(), &lease)
            .unwrap();
        assert_eq!(retained, basis);
        let snapshot = recovered
            .snapshots()
            .snapshot_for_observation(&basis.observation())
            .unwrap();
        recovered.snapshots().release_snapshot(&snapshot).unwrap();
        recovered.release_component_basis(lease).unwrap();
    }
    let predecessor_basis = authority
        .admit_recorded_component(execution, &recovered, &predecessor)
        .unwrap()
        .into_basis();
    assert!(
        recovered
            .owner_component_services()
            .basis_port()
            .compare_current_exact(&predecessor_basis)
            .is_err(),
        "historical admission does not substitute the latest root"
    );
    let foreign_medium = TestDurableMedium::new();
    let foreign = durable_runtime(&foreign_medium);
    assert!(matches!(
        authority
            .admit_recorded_component(execution, &foreign, &successor)
            .unwrap_err()
            .cause(),
        Cause::ForeignRecoveryRuntime
    ));
    let mut changed = serde_json::to_value(&successor).unwrap();
    let digest: worth_foundational::CanonicalDigestId =
        serde_json::from_value(changed["target"]["Canonical"]["artifact_digest"].clone()).unwrap();
    let mut bytes = *digest.bytes();
    bytes[0] ^= 1;
    changed["target"]["Canonical"]["artifact_digest"] =
        serde_json::to_value(worth_foundational::CanonicalDigestId::new(bytes)).unwrap();
    let changed: RecordedRelationalComponentReference = serde_json::from_value(changed).unwrap();
    assert_ne!(changed, successor);
    assert!(matches!(
        authority
            .admit_recorded_component(execution, &recovered, &changed)
            .unwrap_err()
            .cause(),
        Cause::RecordedTargetMismatch
    ));
    assert_eq!(
        medium.offered().len(),
        offers,
        "readmission never republishes"
    );
}

#[test]
fn recorded_component_recovery_never_qualifies_memory_as_a_canonical_durable_source() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let source = memory_runtime();
    create_entity(execution, &source, "memory-canonical");
    let recorded = current(execution, &source);
    let plan = source
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = memory_runtime();
    let (_, authority) = recovered
        .durability_recovery()
        .recover_with_authority(execution, plan)
        .unwrap();
    assert!(matches!(
        authority
            .admit_recorded_component(execution, &recovered, &recorded)
            .unwrap_err()
            .cause(),
        Cause::MemoryOnly
    ));
}

fn prepare_main_candidate(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    key: &str,
) -> worth_relational::facade::mvcc::PreparedRelationalCommitCandidate {
    let services = runtime.owner_component_services();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut txn = services
        .transaction_admission_port()
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    txn.push_batch(
        worth_testkit_relational::durable_publication::batch_create(key),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    services
        .preparation_port()
        .prepare_branch_transaction(execution, txn)
        .unwrap()
}

#[test]
fn successor_memory_component_admission_binds_owner_and_exact_candidate_successor() {
    use worth_execution::{SerialMemoryBudget, SerialRequest};
    use worth_relational::facade::{
        branch::{RelationalBranchBasisDenial, RelationalRecordedComponentRefusal as Refusal},
        transactions::RelationalExecutionDenialCause,
    };
    use worth_testkit_relational::durable_publication::{create_entity_outcome, memory_runtime};

    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = memory_runtime();
    let _ = create_entity_outcome(&execution, &runtime, "seed");
    let foreign = memory_runtime();
    let _ = create_entity_outcome(&execution, &foreign, "foreign-seed");

    let foreign_candidate = prepare_main_candidate(&foreign, &execution, "foreign-commit");
    assert!(matches!(
        runtime
            .owner_component_services()
            .basis_port()
            .admit_successor_memory_component(&execution, &foreign_candidate),
        Err(Refusal::Basis(
            RelationalBranchBasisDenial::ForeignRuntime { .. }
        ))
    ));

    let candidate_a = prepare_main_candidate(&runtime, &execution, "candidate-a");
    let admitted_a = runtime
        .owner_component_services()
        .basis_port()
        .admit_successor_memory_component(&execution, &candidate_a)
        .unwrap();
    // Publish a different equal-sized commit on the same branch ("main", 1 parent).
    let _ = create_entity_outcome(&execution, &runtime, "other-commit");
    let (_, basis_b) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    assert!(matches!(
        runtime
            .owner_component_services()
            .basis_port()
            .observe_admitted_memory_component(admitted_a, &basis_b),
        Err(Refusal::Basis(
            RelationalBranchBasisDenial::WrongImmutableTarget
        ))
    ));

    let candidate_c = prepare_main_candidate(&runtime, &execution, "candidate-c");
    let measure_budget = SerialMemoryBudget::new(None);
    let measure_host = SerialRequest::from_memory(
        measure_budget.clone(),
        worth_execution::CancellationToken::new(),
        None,
    );
    let admitted_c = runtime
        .owner_component_services()
        .basis_port()
        .admit_successor_memory_component(&ExecutionRequest::serial(&measure_host), &candidate_c)
        .unwrap();
    let admitted_bytes = measure_budget.charged_memory_bytes();
    assert!(admitted_bytes > 0);
    // One byte short must refuse with typed PolicyMemoryExhausted and zero retained charge.
    let tight_budget = SerialMemoryBudget::new(Some(admitted_bytes - 1));
    let tight_host = SerialRequest::from_memory(
        tight_budget.clone(),
        worth_execution::CancellationToken::new(),
        None,
    );
    assert!(matches!(
        runtime
            .owner_component_services()
            .basis_port()
            .admit_successor_memory_component(&ExecutionRequest::serial(&tight_host), &candidate_c),
        Err(Refusal::Request(
            RelationalExecutionDenialCause::PolicyMemoryExhausted { .. }
        ))
    ));
    assert_eq!(tight_budget.charged_memory_bytes(), 0);
    let outcome = runtime
        .owner_component_services()
        .publication_port()
        .compare_and_publish(&execution, candidate_c);
    let worth_relational::facade::mvcc::RelationalPublicationOutcome::Performed(performed) =
        outcome
    else {
        panic!("expected performed publication: {outcome:?}");
    };
    let _ = runtime
        .owner_component_services()
        .settlement_port()
        .settle_performed_publication(performed)
        .unwrap();
    let (_, successor_c) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let observed = runtime
        .owner_component_services()
        .basis_port()
        .observe_admitted_memory_component(admitted_c, &successor_c)
        .unwrap();
    assert_eq!(measure_budget.charged_memory_bytes(), admitted_bytes);
    drop(observed);
    assert_eq!(measure_budget.charged_memory_bytes(), 0);
}
