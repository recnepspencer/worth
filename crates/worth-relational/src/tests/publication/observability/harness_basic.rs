use super::fixtures::*;

#[test]
fn harness_runner_captures_snapshot_diagnostics_and_replay() {
    let adapter = RelationalHarnessAdapter;
    let fixture = ScenarioPlan::new(
        "fixture",
        crate::presentation::harness::RelationalFixture {
            entities: Vec::new(),
            relations: Vec::new(),
        },
    )
    .compile();
    let batch = MutationBatch::new("mutate").push(batch_create("from-harness"));
    let request = ExecutionRequest::target("inspect", "entity:0:1".to_string());
    let profile = ExecutionProfile::forensic("forensic");
    let mut runtime = adapter.create_runtime().unwrap();
    adapter.load_fixture(&mut runtime, &fixture).unwrap();
    adapter.apply_mutation_batch(&mut runtime, &batch).unwrap();
    let run = adapter
        .execute(&mut runtime, &fixture, &request, &profile)
        .unwrap();
    let snapshot = adapter
        .capture_snapshot(&runtime, &fixture, &request, &profile)
        .unwrap();
    let diagnostics = adapter
        .capture_diagnostics(&runtime, &fixture, &profile)
        .unwrap();
    let replay_request = ReplayRequest {
        name: "replay".to_string(),
        source_run: run.clone(),
        request: request.clone(),
        profile: profile.clone(),
    };
    let replay = adapter
        .capture_replay(&runtime, &fixture, &replay_request)
        .unwrap();

    assert_eq!(snapshot.observations.len(), 1);
    assert_eq!(
        snapshot.observations[0].status,
        worth_harness::facade::ObservationStatus::Clean
    );
    let Some(worth_harness::facade::SnapshotPayload::Binary(snapshot_binary)) =
        &snapshot.observations[0].value
    else {
        panic!(
            "expected aspect-native binary snapshot value, got {:?}",
            snapshot.observations[0].value
        );
    };
    assert_eq!(
        snapshot_binary.media_type,
        "application/vnd.worth.relational.harness.aspect-snapshot.v1+octet-stream"
    );
    assert!(snapshot_binary
        .bytes
        .starts_with(b"worth.relational.harness.aspect-snapshot.v1"));
    assert_eq!(
        snapshot_binary.size_bytes,
        Some(snapshot_binary.bytes.len() as u64)
    );
    assert!(snapshot_binary
        .content_hash
        .as_deref()
        .is_some_and(|hash| { hash.starts_with("sha256:") }));
    assert!(diagnostics.summary.is_object());
    assert!(replay.summary.is_object());
}

#[test]
fn harness_snapshot_marks_missing_targets_unknown() {
    let adapter = RelationalHarnessAdapter;
    let fixture = ScenarioPlan::new(
        "fixture",
        crate::presentation::harness::RelationalFixture {
            entities: Vec::new(),
            relations: Vec::new(),
        },
    )
    .compile();
    let profile = ExecutionProfile::forensic("forensic");
    let request = ExecutionRequest::target("missing", "entity:0:999".to_string());
    let runtime = adapter.create_runtime().unwrap();
    let snapshot = adapter
        .capture_snapshot(&runtime, &fixture, &request, &profile)
        .unwrap();

    assert_eq!(snapshot.observations.len(), 1);
    assert_eq!(
        snapshot.observations[0].status,
        worth_harness::facade::ObservationStatus::Unknown
    );
    assert_eq!(
        snapshot.observations[0].detail.as_deref(),
        Some("target not visible at captured snapshot")
    );
    assert!(snapshot.observations[0].value.is_none());
}

#[test]
fn harness_fixture_loads_declared_aspect_field_patches() {
    let adapter = RelationalHarnessAdapter;
    let fixture = ScenarioPlan::new(
        "fixture",
        crate::presentation::harness::RelationalFixture {
            entities: vec![crate::presentation::harness::FixtureEntity {
                kind_id: KindId(1),
                client_key: "from-fixture".to_string(),
                fields: name_field_patch("from-fixture"),
            }],
            relations: Vec::new(),
        },
    )
    .compile();
    let mut runtime = RelationalRuntimeApi::builder()
        .schema_registry(test_schema_registry())
        .build();

    adapter.load_fixture(&mut runtime, &fixture).unwrap();

    let snapshot = runtime.visibility_authority().snapshot();
    let read = runtime.read_truth().read_version(snapshot.version_id);
    let entity_id = crate::facade::identity::EntityId::new(PartitionId::main(), 0, 1);
    let entity = read.get_entity(entity_id).expect("fixture entity visible");

    assert_eq!(read_entity_name(entity), Some("from-fixture".into()));
}

#[test]
fn runtime_packet_execution_and_storage_stats_are_readable() {
    let runtime = runtime_with_test_schema();
    let entity = create_entity(&runtime, "first");
    let snapshot = runtime.visibility_authority().snapshot();
    let result = execute_explicit_query(
        &runtime,
        &snapshot,
        "entities",
        vec![RecordRef::Entity(entity)],
    )
    .result;
    let stats = runtime.storage_access().storage_stats();

    assert_eq!(result.entities.len(), 1);
    assert_eq!(stats.live_entities, 1);
    assert!(stats.snapshot_count >= 1);
    assert!(stats.entity_chunks >= 1);
}

#[test]
fn repeated_serial_runs_are_harness_comparable() {
    let adapter = RelationalHarnessAdapter;
    let runner = worth_harness::facade::HarnessRunner::new(adapter);
    let fixture = ScenarioPlan::new(
        "fixture",
        crate::presentation::harness::RelationalFixture {
            entities: Vec::new(),
            relations: Vec::new(),
        },
    )
    .compile();
    let batch = MutationBatch::new("mutate").push(batch_create("stable"));
    let request = ExecutionRequest::target("inspect", "entity:0:1".to_string());
    let profile = ExecutionProfile::forensic("forensic");
    let run_a = runner
        .execute_core(&fixture, Some(&batch), &request, &profile)
        .unwrap();
    let run_b = runner
        .execute_core(&fixture, Some(&batch), &request, &profile)
        .unwrap();
    let comparison = runner
        .compare_runs(
            &run_a.run,
            &run_b.run,
            &worth_harness::facade::ComparisonProfile::default(),
        )
        .unwrap();

    assert!(comparison.mismatches.is_empty());
}

#[test]
fn harness_publication_artifact_extension_excludes_diagnostic_volume_from_run_parity() {
    let adapter = RelationalHarnessAdapter;
    let runner = worth_harness::facade::HarnessRunner::new(adapter);
    let (fixture, batch, request) = harness_phase8_fixture_batch_request();
    let serial = runner
        .execute_core(
            &fixture,
            Some(&batch),
            &request,
            &ExecutionProfile::serial("serial"),
        )
        .unwrap();
    let staged = runner
        .execute_core(
            &fixture,
            Some(&batch),
            &request,
            &ExecutionProfile::staged_parallel("staged"),
        )
        .unwrap();
    let serial_publication_artifacts = &serial.run.extensions["publication_artifacts"];
    let staged_publication_artifacts = &staged.run.extensions["publication_artifacts"];
    let comparison = runner
        .compare_runs(
            &serial.run,
            &staged.run,
            &worth_harness::facade::ComparisonProfile::default(),
        )
        .unwrap();

    assert!(serial_publication_artifacts["observation"]
        .get("diagnostics_artifact_count")
        .is_none());
    assert!(staged_publication_artifacts["observation"]
        .get("diagnostics_artifact_count")
        .is_none());
    assert!(comparison.mismatches.is_empty());
}
