use worth_execution::{
    CancellationSource, ExecutionAllocationPolicy, ExecutionRequest, SerialMemoryBudget,
    SerialRequest,
};
use worth_relational::facade::{
    durability::RelationalCheckpointArtifactPolicy, history::BranchId,
    runtime::RelationalRuntimeApi,
};

#[test]
fn a_late_capture_cannot_regress_the_selected_transition_frontier_or_membership() {
    let directory = tempfile::tempdir().unwrap();
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    use worth_relational::facade::{
        identity::{KindId, PartitionId},
        mvcc::RelationalTransactionIntent,
        schema::{
            EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry,
            SchemaId, SchemaVersionId,
        },
        symbols::ClientKey,
        transactions::{CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch},
    };
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "checkpoint.seed".into(),
            schema_id: SchemaId("checkpoint".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(Vec::new()),
        })
        .unwrap();
    let home = worth_store_testkit_application_oracle::facade::home_opening::open(
        worth_store_testkit_application_oracle::facade::home_opening::request(
            directory.path().join("home"),
            RelationalRuntimeApi::builder().schema_setup(move |schema| {
                schema.schema_registry(registry);
            }),
        ),
        &execution,
    )
    .unwrap();
    let (_, basis) = home
        .relational()
        .runtime()
        .observe_branch(&home.relational().runtime().main_branch_identity())
        .unwrap();
    let mut transaction = home
        .relational()
        .runtime()
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("seed").push(MutationIntent::Create(CreateIntent::Entity(
                EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw("seed"),
                    fields: Default::default(),
                },
            ))),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction
        .commit(&execution, home.relational().runtime())
        .unwrap();
    let earlier = home
        .relational()
        .runtime()
        .durability_authority()
        .native_checkpoint_with_policy(
            &execution,
            ExecutionAllocationPolicy::SystemAllocation,
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let (_, source) = home
        .relational()
        .runtime()
        .observe_fork_source(&BranchId("main".into()))
        .unwrap();
    home.relational()
        .runtime()
        .fork_branch(&execution, BranchId("later".into()), source)
        .unwrap();
    let selected = home.checkpoint(&execution).unwrap();
    assert!(selected.covered_transitions().0 > earlier.covered_branch_transitions().unwrap().0);
    let refused = home
        .publish_captured_relational_checkpoint_for_test(earlier, &execution)
        .unwrap_err();
    assert!(
        matches!(
            &refused,
            worth_store_application::ApplicationHomeCheckpointDenial::Native(
                worth_store_relational::RelationalSectionCheckpointDenial::StaleCapture
            )
        ),
        "stale native capture must keep its exact refusal: {refused:?}"
    );
    let records = home.physical_serving_for_test().anchored_records().unwrap();
    let record = records
        .read(
            worth_store::physical_runtime::AnchoredRecordName::new("home.checkpoint").unwrap(),
            1 << 30,
        )
        .unwrap()
        .unwrap();
    assert_eq!(record.version().generation(), selected.generation());
    drop(record);
    home.close(&execution);
}

#[test]
fn selected_checkpoint_survives_observer_drop_online_history_and_mutation_before_ack_recovery() {
    use worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint;
    use worth_store::physical_runtime::{AnchoredRecordStepKind, PhysicalMutationOutcome};
    use worth_store_application::{ApplicationHomeCheckpointDenial, ApplicationHomeCloseOutcome};
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let branch = BranchId("main".into());
    let home = crate::durable_run::open(directory.path(), &execution).unwrap();
    let created = crate::workload::commit(
        home.relational().runtime(),
        &execution,
        &branch,
        crate::batches::create(1),
    )
    .unwrap();
    let worth_relational::facade::transactions::RecordRef::Entity(entity) =
        created.changed_records[0]
    else {
        panic!("created entity")
    };
    home.relational()
        .runtime()
        .snapshots()
        .release_snapshot(&created.snapshot)
        .unwrap();
    for round in 2..=3 {
        let changed = crate::workload::commit(
            home.relational().runtime(),
            &execution,
            &branch,
            crate::batches::update(entity, round),
        )
        .unwrap();
        home.relational()
            .runtime()
            .snapshots()
            .release_snapshot(&changed.snapshot)
            .unwrap();
    }
    let expected_history = home
        .relational()
        .runtime()
        .history()
        .entity_aspect_history(&execution, &branch, entity, None)
        .unwrap();
    assert_eq!(expected_history.len(), 3);
    home.checkpoint(&execution).unwrap();
    assert!(matches!(
        home.close(&execution),
        ApplicationHomeCloseOutcome::Closed {
            checkpoint: Ok(_),
            ..
        }
    ));
    let home = crate::durable_run::open(directory.path(), &execution).unwrap();
    assert_eq!(home.relational().cold_storage_reads().member_reads, 0);
    let prior_generation = crate::selected_checkpoint::generation(&home);
    let mut unrelated = crate::batches::create(4);
    let worth_relational::facade::transactions::MutationIntent::Create(
        worth_relational::facade::transactions::CreateIntent::Entity(spec),
    ) = &mut unrelated.intents[0]
    else {
        panic!("entity create")
    };
    spec.partition_id = worth_relational::facade::identity::PartitionId(2);
    let changed =
        crate::workload::commit(home.relational().runtime(), &execution, &branch, unrelated)
            .unwrap();
    home.relational()
        .runtime()
        .snapshots()
        .release_snapshot(&changed.snapshot)
        .unwrap();

    let cancellation = CancellationSource::new();
    let interrupted =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let (result, gate) = std::thread::scope(|scope| {
        let control = scope.spawn(move || {
            let gate: worth_store::physical_runtime::certification::CertificationPhysicalMutationPauseGate =
                receiver.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
            assert!(gate.await_arrival(), "the actual final anchor reached root replacement");
            cancellation.cancel();
            gate
        });
        let result = home.checkpoint_with_step_observer_for_test(
            &ExecutionRequest::serial(&interrupted),
            |registration| {
                if registration.kind() == AnchoredRecordStepKind::Anchor {
                    let gate = home
                        .physical_serving_for_test()
                        .certification_pause_physical_mutation_at(
                            CertificationPhysicalMutationCheckpoint::AfterRootReplacement,
                        );
                    sender.send(gate).unwrap();
                }
            },
        );
        (result, control.join().unwrap())
    });
    let observer = match result {
        Err(ApplicationHomeCheckpointDenial::RecoveryRequired(observer)) => observer,
        other => panic!("interrupted actual anchor must retain the Home attempt: {other:?}"),
    };
    assert!(!observer.acknowledgement_complete);
    gate.release();
    let terminal_deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let observation = home
            .checkpoint_observation()
            .expect("the Home still owns this attempt");
        if let Some(terminal) = observation.physical_terminal() {
            assert!(
                matches!(terminal.outcome(), PhysicalMutationOutcome::Completed(done) if done.persisted_records().len() == 1)
            );
            assert_eq!(terminal.settlement_failure(), None);
            assert!(!observation.acknowledgement_complete);
            break;
        }
        assert!(
            std::time::Instant::now() < terminal_deadline,
            "actual terminal evidence was not delivered"
        );
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let selected_generation = crate::selected_checkpoint::generation(&home);
    assert_eq!(selected_generation, prior_generation + 1);
    drop(observer);
    assert_eq!(
        home.relational()
            .runtime()
            .history()
            .entity_aspect_history(&execution, &branch, entity, None)
            .unwrap(),
        expected_history
    );
    let changed = crate::workload::commit(
        home.relational().runtime(),
        &execution,
        &branch,
        crate::batches::update(entity, 5),
    )
    .unwrap();
    home.relational()
        .runtime()
        .snapshots()
        .release_snapshot(&changed.snapshot)
        .unwrap();
    assert!(
        !home
            .checkpoint_observation()
            .unwrap()
            .acknowledgement_complete
    );
    let recovered = home.recover_checkpoint(&execution).unwrap();
    assert_eq!(recovered.generation(), selected_generation);
    assert_eq!(
        crate::selected_checkpoint::generation(&home),
        selected_generation,
        "ack recovery must not republish the manifest"
    );
    let exact_history = home
        .relational()
        .runtime()
        .history()
        .entity_aspect_history(&execution, &branch, entity, None)
        .unwrap();
    assert_eq!(exact_history.len(), expected_history.len() + 1);
    let artifacts = crate::workload::artifacts_of(&execution, home.relational().runtime());
    assert!(matches!(
        home.close(&execution),
        ApplicationHomeCloseOutcome::Closed {
            checkpoint: Ok(_),
            ..
        }
    ));
    let home = crate::durable_run::open(directory.path(), &execution).unwrap();
    assert_eq!(
        crate::workload::artifacts_of(&execution, home.relational().runtime()),
        artifacts
    );
    assert_eq!(
        home.relational()
            .runtime()
            .history()
            .entity_aspect_history(&execution, &branch, entity, None)
            .unwrap(),
        exact_history
    );
    assert!(matches!(
        home.close(&execution),
        ApplicationHomeCloseOutcome::Closed { .. }
    ));
}

#[test]
fn known_checkpoint_failure_allows_close_and_reopen_preserves_the_journal_tail() {
    use worth_store_application::ApplicationHomeCloseOutcome;
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let home = crate::durable_run::open(directory.path(), &execution).unwrap();
    let committed = crate::workload::commit(
        home.relational().runtime(),
        &execution,
        &BranchId("main".into()),
        crate::batches::create(1),
    )
    .unwrap();
    home.relational()
        .runtime()
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    let expected = crate::workload::artifacts_of(&execution, home.relational().runtime());
    let closing = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(0)),
        CancellationSource::new().token(),
        None,
    );
    match home.close(&ExecutionRequest::serial(&closing)) {
        ApplicationHomeCloseOutcome::Closed { checkpoint, .. } => assert!(checkpoint.is_err()),
        ApplicationHomeCloseOutcome::CloseRefused { .. } => {
            panic!("a known checkpoint admission failure cannot refuse close")
        }
    }
    let home = crate::durable_run::open(directory.path(), &execution).unwrap();
    assert_eq!(
        crate::workload::artifacts_of(&execution, home.relational().runtime()),
        expected
    );
    assert!(matches!(
        home.close(&execution),
        ApplicationHomeCloseOutcome::Closed { .. }
    ));
}
