//! Actual installed publication denial exposes metadata, never proposal strings.
use super::*;
#[test]
fn empty_optional_struct_refusal_carries_numeric_member_and_redacts_proposal_strings() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let mut runtime = schema::runtime();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(crate::schema::data::RelationalSchemaRegistry::new())
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    // Establish real committed payload and incidence before the refused attempt.
    let baseline_targets = ["baseline-left", "baseline-right"].map(|key| CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(6),
        client_key: crate::symbols::data::ClientKey::raw(key),
    });
    let edge = CreatedRelationRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(2),
        client_key: crate::symbols::data::ClientKey::raw("baseline-incidence"),
        source: EntityReference::Created(baseline_targets[0].clone()),
        target: EntityReference::Created(baseline_targets[1].clone()),
    };
    let mut baseline_batch = WorkerIntentBatch::new("committed-refusal-baseline");
    for target in &baseline_targets {
        baseline_batch =
            baseline_batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: target.partition_id,
                kind_id: target.kind_id,
                client_key: target.client_key.clone(),
                fields: string_aspect_field_patch([(
                    aspect_key("value"),
                    field_key("value"),
                    "retained baseline payload",
                )]),
            })));
    }
    baseline_batch = baseline_batch.push(MutationIntent::Create(CreateIntent::Relation(
        RelationSpec {
            partition_id: edge.partition_id,
            kind_id: edge.kind_id,
            client_key: edge.client_key.clone(),
            source: edge.source.clone(),
            target: edge.target.clone(),
            fields: AspectFieldPatch::default(),
        },
    )));
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(baseline_batch, execution.allocation_policy())
        .unwrap();
    let baseline = transaction.commit(execution, &runtime).unwrap();
    let baseline_ids = baseline_targets.map(|target| baseline.created_entity(&target).unwrap());
    let edge_id = baseline.created_relation(&edge).unwrap();
    release_test_commit_snapshot(&runtime, &baseline);
    let before = test_owner_main_basis(&runtime).unwrap();
    let before_partition = before
        .inner
        .root
        .partition_state(PartitionId::main())
        .unwrap();
    assert_eq!(
        before_partition
            .adjacency
            .get(baseline_ids[0].slot_index())
            .unwrap()
            .current_ids(),
        [edge_id]
    );
    assert_eq!(
        before_partition
            .reverse_adjacency
            .get(baseline_ids[1].slot_index())
            .unwrap()
            .current_ids(),
        [edge_id]
    );
    for id in baseline_ids {
        assert!(before_partition
            .entity_arena
            .get(&id)
            .unwrap()
            .extra()
            .authoritative_aspect_state
            .is_some());
    }
    let before_history = runtime.history().commit_envelopes_snapshot();
    assert!(before_history
        .iter()
        .any(|envelope| envelope.commit == baseline.commit));
    let before_stream = runtime.history().latest_patch_stream_position();
    let targets = [
        (6, "private-client-key-sentinel"),
        (4, "empty-client-key-sentinel"),
    ]
    .map(|(kind, key)| CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(kind),
        client_key: crate::symbols::data::ClientKey::raw(key),
    });
    let mut batch = WorkerIntentBatch::new("real-refused-proposal");
    for (target, fields) in targets.iter().zip([
        string_aspect_field_patch([(
            aspect_key("value"),
            field_key("value"),
            "private-payload-sentinel",
        )]),
        AspectFieldPatch::default(),
    ]) {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: target.partition_id,
            kind_id: target.kind_id,
            client_key: target.client_key.clone(),
            fields,
        })));
    }
    let mut publication = native_birth_fixture(
        owner.admit_required_output("generic-creator", [1; 32]),
        targets.to_vec(),
    );
    let RequiredOutputPublication::Birth { roles, .. } = &mut publication else {
        panic!("birth")
    };
    for (ordinal, role) in roles.iter_mut().enumerate() {
        role.role = format!("authored-role-sentinel-{ordinal}");
    }
    let error = publish(&runtime, execution, &installed, batch, publication).unwrap_err();
    let detail = error.detail();
    assert!(detail.contains("reason=empty-payload-with-executable-bindings"));
    assert!(detail.contains("member_ordinal=1"));
    assert!(detail.contains("kind=4 schema_version=1"));
    assert!(detail.contains("executable_bindings=1"));
    let [RecordRef::Entity(id)] = error.context().affected_records.as_slice() else {
        panic!("one actual failed member")
    };
    assert!(detail.contains(&format!(
        "record=entity:{}:{}:{}",
        id.partition_value(),
        id.local_slot_value(),
        id.generation_value()
    )));
    for secret in [
        "private-client-key-sentinel",
        "empty-client-key-sentinel",
        "private-payload-sentinel",
        "authored-role-sentinel",
    ] {
        assert!(!detail.contains(secret));
    }
    // Symbol interning and burned allocation slots are deliberately not rewound.
    // Compare the committed owner image and its actual history, not checkpoint bytes.
    let after = test_owner_main_basis(&runtime).unwrap();
    assert_eq!(after.reference(), before.reference());
    assert_eq!(after.truth_version(), before.truth_version());
    assert_eq!(
        after.inner.root.canonical_envelope(),
        before.inner.root.canonical_envelope()
    );
    let after_partition = after
        .inner
        .root
        .partition_state(PartitionId::main())
        .unwrap();
    assert_eq!(
        after_partition.entity_arena.occupied_slots(),
        before_partition.entity_arena.occupied_slots()
    );
    assert_eq!(
        after_partition.relation_arena.occupied_slots(),
        before_partition.relation_arena.occupied_slots()
    );
    assert!(after_partition.entity_arena.get(id).is_none());
    for id in baseline_ids {
        let old = before_partition.entity_arena.get(&id).unwrap();
        let current = after_partition.entity_arena.get(&id).unwrap();
        assert!(old.is_live() && current.is_live());
        assert_eq!(current.kind_id(), old.kind_id());
        assert_eq!(current.generation(), old.generation());
        assert_eq!(
            current.extra().authoritative_aspect_state,
            old.extra().authoritative_aspect_state
        );
        assert_eq!(
            after_partition
                .adjacency
                .get(id.slot_index())
                .map(|edges| edges.current_ids().to_vec()),
            before_partition
                .adjacency
                .get(id.slot_index())
                .map(|edges| edges.current_ids().to_vec())
        );
        assert_eq!(
            after_partition
                .reverse_adjacency
                .get(id.slot_index())
                .map(|edges| edges.current_ids().to_vec()),
            before_partition
                .reverse_adjacency
                .get(id.slot_index())
                .map(|edges| edges.current_ids().to_vec())
        );
    }
    let old = before_partition.relation_arena.get(&edge_id).unwrap();
    let current = after_partition.relation_arena.get(&edge_id).unwrap();
    assert!(old.is_live() && current.is_live());
    assert_eq!(current.kind_id(), old.kind_id());
    assert_eq!(current.generation(), old.generation());
    assert_eq!(current.extra().endpoints, old.extra().endpoints);
    assert_eq!(
        current.extra().authoritative_aspect_state,
        old.extra().authoritative_aspect_state
    );
    assert_eq!(
        runtime.history().commit_envelopes_snapshot(),
        before_history
    );
    assert_eq!(
        runtime.history().latest_patch_stream_position(),
        before_stream
    );
}
