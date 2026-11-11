//! Real cold endpoint publication preserves deferred history in its exact root.
use super::*;

#[test]
fn cold_endpoint_publication_carries_omitted_history_into_the_exact_observation_root() {
    let setup = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&setup);
    let (runtime, medium) = initialized(request);
    let mut entities = Vec::new();
    for key in [
        "prefix-source",
        "prefix-first-target",
        "prefix-other-target",
    ] {
        let created = create_entity_outcome(request, &runtime, key);
        entities.push(created.semantic_result().created_entities()[0].assigned);
        release_test_commit_snapshot(&runtime, &created);
    }
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("prefix-relation").push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: entities[0].partition_id,
                    kind_id: KindId(2),
                    client_key: "prefix-edge".into(),
                    source: EntityReference::Existing(entities[0]),
                    target: EntityReference::Existing(entities[1]),
                    fields: field_patch("label", "label", "prefix-edge"),
                }),
            )),
            request.allocation_policy(),
        )
        .unwrap();
    let created = transaction.commit(request, &runtime).unwrap();
    let relation = created.semantic_result().created_relations()[0].assigned;
    let original_version = created.version_id;
    release_test_commit_snapshot(&runtime, &created);
    drop(created);
    let changed = endpoints(request, &runtime, relation, entities[0], entities[2]);
    release_test_commit_snapshot(&runtime, &changed);
    drop(changed);
    let (_, old_basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let old_observation = old_basis.observation();
    let oracle = runtime
        .read_truth()
        .read_observation(request, &old_observation)
        .unwrap()
        .unwrap();
    let expected_entities = oracle.entities().to_vec();
    let expected_relations = oracle.relations().to_vec();
    assert_eq!(expected_relations.len(), 1);
    drop((oracle, old_observation, old_basis));
    // Preserve the complete ordinary Full result, including the validated
    // endpoint aspects. Revert before capture, then repeat that edit cold.
    let reverted = endpoints(request, &runtime, relation, entities[0], entities[1]);
    release_test_commit_snapshot(&runtime, &reverted);
    drop(reverted);
    let image = capture_external(&runtime, &medium, request);
    medium.file_checkpoint(image.checkpoint().clone());
    drop(runtime);
    let history = member(&image, entities[0].partition_id, true);
    let source = SelectedSource::new(medium.clone());
    let runtime = recover(&image, source.clone(), receiver(&medium), request);
    let history_reads = || {
        source
            .member_reads
            .lock()
            .unwrap()
            .iter()
            .filter(|&&member| member == history)
            .count()
    };
    assert_eq!(history_reads(), 0);
    read_head(request, &runtime, entities[0]).unwrap();
    assert_eq!(history_reads(), 0, "Head must not demand omitted history");
    let changed = endpoints(request, &runtime, relation, entities[0], entities[2]);
    let commit_id = changed.commit.commit_id;
    let current_version = changed.version_id;
    release_test_commit_snapshot(&runtime, &changed);
    drop(changed);
    assert_eq!(
        history_reads(),
        0,
        "endpoint publication keeps the omitted prefix cold"
    );
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let observation = basis.observation();
    assert_eq!(observation.commit_id(), Some(commit_id));
    assert!(observation.issued_from(&basis));
    let descriptor = basis.descriptor().clone();
    assert_eq!(
        descriptor
            .reference()
            .target()
            .as_basis()
            .unwrap()
            .selected_commit_id(),
        commit_id.0
    );
    let view = runtime
        .read_truth()
        .read_observation(request, &observation)
        .unwrap()
        .unwrap();
    assert_eq!(
        history_reads(),
        1,
        "this exact observation's Full demand reads its one prior member"
    );
    assert_eq!(basis.descriptor(), &descriptor);
    assert_eq!(view.entities(), expected_entities);
    assert_eq!(view.relations(), expected_relations);
    for (version, target) in [
        (original_version, entities[1]),
        (current_version, entities[2]),
    ] {
        let rows = runtime
            .read_truth()
            .outgoing_relations_of_kind_at_version(request, entities[0], KindId(2), version)
            .unwrap();
        assert_eq!(
            rows.iter()
                .map(|row| (row.relation_id, row.source, row.target))
                .collect::<Vec<_>>(),
            vec![(relation, entities[0], target)]
        );
    }
}

fn endpoints(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    relation_id: worth_relational::facade::identity::RelationId,
    source: EntityId,
    target: EntityId,
) -> CommitResult {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("cold-prefix-endpoint").push(MutationIntent::Relation(
                RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                    relation_id,
                    kind_id: KindId(2),
                    source: EntityReference::Existing(source),
                    target: EntityReference::Existing(target),
                }),
            )),
            request.allocation_policy(),
        )
        .unwrap();
    transaction.commit(request, runtime).unwrap()
}
