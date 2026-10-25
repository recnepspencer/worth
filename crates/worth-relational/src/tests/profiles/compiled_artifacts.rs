use crate::facade::config::AdjacencyBackend;
use crate::facade::runtime::CompiledArtifactAuthorityStatus;
use crate::tests::support::*;

#[test]
fn chip_profile_emits_dense_patch_surface_details() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::ChipSimulation);
    let left = create_entity_in_partition(execution, &runtime, "left", PartitionId(7));
    let right = create_entity_in_partition(execution, &runtime, "right", PartitionId(11));
    let _ =
        create_relation_in_partition(execution, &runtime, left, right, "bridge", PartitionId(29));
    let publication = runtime.publication().artifacts();
    let patch = publication.latest_patch().unwrap();

    assert!(patch
        .authoritative_record_patches
        .iter()
        .all(|record| matches!(record.detail, PatchDetail::DenseBitset(_))));
}

#[test]
fn chip_profile_preserves_relation_traversal_with_compressed_adjacency_backend() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::ChipSimulation);
    let source = create_entity_in_partition(execution, &runtime, "source", PartitionId(7));
    let target_a = create_entity_in_partition(execution, &runtime, "target-a", PartitionId(7));
    let target_b = create_entity_in_partition(execution, &runtime, "target-b", PartitionId(9));
    let relation_a =
        create_relation_in_partition(execution, &runtime, source, target_a, "r-a", PartitionId(7));
    let relation_b = create_relation_in_partition(
        execution,
        &runtime,
        source,
        target_b,
        "r-b",
        PartitionId(12),
    );
    let version_id = runtime.history().latest_commit().unwrap().version_id;

    assert_eq!(
        runtime.config().storage.adjacency_policy.backend,
        AdjacencyBackend::CompressedFanoutAdjacency
    );
    assert_eq!(
        runtime
            .storage_access()
            .outgoing_relations_for_entity(source, version_id),
        vec![relation_a, relation_b]
    );
    assert_eq!(
        runtime
            .storage_access()
            .incoming_relations_for_entity(target_b, version_id),
        vec![relation_b]
    );
}

#[test]
fn chip_profile_compiled_artifacts_are_derived_from_committed_truth() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::ChipSimulation);
    let left = create_entity_in_partition(execution, &runtime, "left", PartitionId(7));
    let right = create_entity_in_partition(execution, &runtime, "right", PartitionId(11));
    let _ =
        create_relation_in_partition(execution, &runtime, left, right, "bridge", PartitionId(29));
    let commit = runtime.history().latest_commit().unwrap().clone();

    let artifact = runtime
        .compiled_artifacts_authority()
        .compile_execution_artifact(
            execution,
            commit.commit_id,
            vec![PartitionId(7), PartitionId(11), PartitionId(29)],
        )
        .unwrap();

    assert_eq!(artifact.source_commit_id, commit.commit_id);
    assert_eq!(artifact.source_version_id, commit.version_id);
    assert_eq!(artifact.source_branch_id, BranchId("main".to_string()));
    assert_eq!(
        runtime
            .compiled_artifacts()
            .compiled_artifact_authority_status(artifact.artifact_id),
        CompiledArtifactAuthorityStatus::Authoritative
    );
}

#[test]
fn compiled_artifact_rejects_stale_topology_after_later_commit() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema_profile(RelationalRuntimeProfile::ChipSimulation);
    let original = create_entity_outcome(execution, &runtime, "seed");
    let artifact = runtime
        .compiled_artifacts_authority()
        .compile_execution_artifact(
            execution,
            original.commit.commit_id,
            vec![PartitionId::main()],
        )
        .unwrap();
    let _later = create_entity_outcome(execution, &runtime, "later");

    assert_eq!(
        runtime
            .compiled_artifacts()
            .compiled_artifact_authority_status(artifact.artifact_id),
        CompiledArtifactAuthorityStatus::StaleVersion
    );
}

#[test]
fn chip_profile_declared_aspect_fanout_preserves_endpoint_history_for_netlist_like_shapes() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema_profile(
        RelationalRuntimeProfile::ChipSimulation,
        CascadeDeletePolicy::RetainDanglingForAudit,
    );
    let source = create_entity_in_partition(execution, &runtime, "net-src", PartitionId(7));
    let targets = (0..3)
        .map(|index| {
            create_entity_in_partition(
                execution,
                &runtime,
                &format!("net-target-{index}"),
                PartitionId((index + 11) as u32),
            )
        })
        .collect::<Vec<_>>();
    let relations = targets
        .iter()
        .enumerate()
        .map(|(index, target)| {
            create_relation_in_partition(
                execution,
                &runtime,
                source,
                *target,
                &format!("net-edge-{index}"),
                PartitionId((index + 21) as u32),
            )
        })
        .collect::<Vec<_>>();
    let live_version = runtime.history().latest_commit().unwrap().version_id;
    let live_commit_id = runtime.history().latest_commit().unwrap().commit_id;
    let compiled = runtime
        .compiled_artifacts_authority()
        .compile_execution_artifact(
            execution,
            live_commit_id,
            vec![
                PartitionId(7),
                PartitionId(11),
                PartitionId(12),
                PartitionId(13),
            ],
        )
        .unwrap();
    let deleted = delete_entity(execution, &runtime, source);

    assert_eq!(
        runtime
            .storage_access()
            .outgoing_relations_for_entity(source, live_version),
        relations
    );
    assert_eq!(
        runtime
            .compiled_artifacts()
            .compiled_artifact_authority_status(compiled.artifact_id),
        CompiledArtifactAuthorityStatus::StaleVersion
    );
    assert_eq!(deleted.changed_records.len(), 4);
    for relation in relations {
        let history = runtime.history().relation_aspect_history(
            &BranchId("main".to_string()),
            relation,
            None,
        );
        assert_eq!(history.len(), 2);
        assert_direct_history_origin_invariants(&history, RecordRef::Relation(relation));
        assert_eq!(
            history[0].origin.changed_aspects,
            ordered_aspect_keys([
                aspect_key("label"),
                aspect_key("lifecycle"),
                aspect_key("source"),
                aspect_key("target"),
            ])
        );
        assert_eq!(
            history[1].origin.changed_aspects,
            ordered_aspect_keys([aspect_key("lifecycle")])
        );
    }
}

#[test]
fn chip_profile_branch_local_topology_pressure_preserves_relation_history_isolation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema_profile(
        RelationalRuntimeProfile::ChipSimulation,
        CascadeDeletePolicy::RetainDanglingForAudit,
    );
    let source = create_entity_in_partition(execution, &runtime, "topo-src", PartitionId(7));
    let target = create_entity_in_partition(execution, &runtime, "topo-target", PartitionId(11));
    let relation = create_relation_in_partition(
        execution,
        &runtime,
        source,
        target,
        "topo-edge",
        PartitionId(21),
    );
    let main_commit = runtime.history().latest_commit().unwrap().clone();
    let main_artifact = runtime
        .compiled_artifacts_authority()
        .compile_execution_artifact(
            execution,
            main_commit.commit_id,
            vec![PartitionId(7), PartitionId(11), PartitionId(21)],
        )
        .unwrap();
    runtime
        .history_authority()
        .fork_branch_from(
            execution,
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let feature_target = create_entity_in_partition_on_branch(
        execution,
        &runtime,
        "topo-target-feature",
        PartitionId(13),
        BranchId("feature".to_string()),
    );
    let feature_relation = create_relation_in_partition_on_branch(
        execution,
        &runtime,
        source,
        feature_target,
        "topo-edge-feature",
        "topo-edge-feature",
        PartitionId(22),
        BranchId("feature".to_string()),
    );
    let main_view = runtime
        .read_truth()
        .project_historical_version(main_commit.version_id)
        .all_authoritative_relation_records(execution)
        .expect("read request remains live");
    let feature_commit = runtime
        .history()
        .branch_head(&BranchId("feature".to_string()))
        .unwrap()
        .clone();
    let feature_view = runtime
        .read_truth()
        .project_historical_version(feature_commit.version_id)
        .all_authoritative_relation_records(execution)
        .expect("read request remains live");
    let feature_artifact = runtime
        .compiled_artifacts_authority()
        .compile_execution_artifact(
            execution,
            feature_commit.commit_id,
            vec![
                PartitionId(7),
                PartitionId(11),
                PartitionId(13),
                PartitionId(21),
                PartitionId(22),
            ],
        )
        .unwrap();
    let main_history =
        runtime
            .history()
            .relation_aspect_history(&BranchId("main".to_string()), relation, None);
    let feature_history =
        runtime
            .history()
            .relation_aspect_history(&BranchId("feature".to_string()), relation, None);
    let main_digest = relation_aspect_history_digest_on_branch(
        &runtime,
        &BranchId("main".to_string()),
        relation,
        None,
    );
    let feature_digest = relation_aspect_history_digest_on_branch(
        &runtime,
        &BranchId("feature".to_string()),
        relation,
        Some(&any_aspect_filter(["lifecycle"])),
    );

    assert_eq!(main_artifact.source_branch_id, BranchId("main".to_string()));
    assert_eq!(
        feature_artifact.source_branch_id,
        BranchId("feature".to_string())
    );
    assert_eq!(main_artifact.source_commit_id, main_commit.commit_id);
    assert_eq!(feature_artifact.source_commit_id, feature_commit.commit_id);
    assert_eq!(
        main_view
            .iter()
            .map(|record| record.relation_id)
            .collect::<Vec<_>>(),
        vec![relation]
    );
    assert_eq!(
        feature_view
            .iter()
            .map(|record| record.relation_id)
            .collect::<Vec<_>>(),
        vec![relation, feature_relation]
    );
    assert_eq!(main_history.len(), 1);
    assert_eq!(feature_history.len(), 0);
    assert_direct_history_origin_invariants(&main_history, RecordRef::Relation(relation));
    assert_eq!(
        main_history[0].origin.changed_aspects,
        ordered_aspect_keys([
            aspect_key("label"),
            aspect_key("lifecycle"),
            aspect_key("source"),
            aspect_key("target"),
        ])
    );
    assert_eq!(main_digest.entry_count, 1);
    assert_eq!(feature_digest.entry_count, 0);
    assert_eq!(
        runtime
            .compiled_artifacts()
            .compiled_artifact_authority_status(main_artifact.artifact_id),
        CompiledArtifactAuthorityStatus::StaleVersion
    );
    assert_eq!(
        runtime
            .compiled_artifacts()
            .compiled_artifact_authority_status(feature_artifact.artifact_id),
        CompiledArtifactAuthorityStatus::Authoritative
    );
}
