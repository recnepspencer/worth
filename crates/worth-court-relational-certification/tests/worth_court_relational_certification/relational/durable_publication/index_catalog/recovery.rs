//! A selected catalog can contain registrations newer than a frozen tail.
use super::*;
use std::sync::Arc;

#[test]
fn checkpoint_catalog_ahead_of_frozen_tail_preserves_original_body_and_refuses_false_coverage() {
    use worth_relational::facade::{
        history::{
            RelationalAuthoredEntityIndexDefinition, RelationalAuthoredWriteDenial,
            RelationalAuthoredWriteSelector,
        },
        storage::AuthoritativeFieldComparisonKey,
    };
    for touching in [false, true] {
        let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
        let execution = &ExecutionRequest::serial(&request);
        let medium = TestDurableMedium::new();
        let runtime = durable_runtime(&medium);
        create_entity(execution, &runtime, "prefix-predecessor");
        let predecessor_branch = BranchId("retained-prefix-predecessor".into());
        let (_, fork) = runtime
            .observe_fork_source(&BranchId("main".into()))
            .unwrap();
        runtime
            .fork_branch(execution, predecessor_branch.clone(), fork)
            .unwrap();
        let original = runtime
            .index_authority()
            .register(definition("original-prefix"))
            .unwrap();
        let frozen = prepare(
            execution,
            &runtime,
            &BranchId("main".into()),
            "old-frozen-tail",
        );
        let mut untouched = definition("later-untouched");
        if !touching {
            untouched.kind = DerivedIndexKind::EntityField {
                field_locator: worth_relational::facade::transactions::planned_single_field_locator(
                    worth_foundational::facade::AspectKey::new("label").unwrap(),
                    worth_foundational::facade::FieldKey::new("label").unwrap(),
                ),
            };
        }
        let untouched = runtime.index_authority().register(untouched).unwrap();
        medium.file_checkpoint(capture(execution, &runtime));
        let committed = publish(execution, &runtime, frozen);
        assert_eq!(delta(&committed)["next_index_id"], original.index_id.0 + 1);
        let (_, live_before) = runtime
            .observe_branch(&runtime.branch_identity(&predecessor_branch).unwrap())
            .unwrap();
        let (_, live_after) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let DerivedIndexKind::EntityField { field_locator } = &original.kind else {
            unreachable!()
        };
        let original_key = RelationalAuthoredWriteSelector::EntityFieldIndex {
            definition: RelationalAuthoredEntityIndexDefinition {
                index_id: original.index_id,
                name: original.name.clone(),
                branch_scoped: original.branch_scoped,
                locator: field_locator.clone(),
            },
            kind: KindId(1),
            value: AuthoritativeFieldComparisonKey::from_aspect_value(
                &worth_foundational::facade::AspectValue::String("original-prefix-witness".into()),
            ),
        };
        assert!(
            runtime
                .authored_write_history_cursor(
                    *execution,
                    &live_before,
                    runtime
                        .select_exact_commit(&live_after.observation(), committed.commit.commit_id)
                        .into_outcome()
                        .unwrap(),
                    &[original_key]
                )
                .is_ok(),
            "original LIVE prefix was genuinely issued before recovery"
        );
        let original_body =
            serde_json::from_str::<Value>(&serde_json::to_string(committed.envelope()).unwrap())
                .unwrap();
        let actual_artifact = medium.offered().pop().unwrap();
        release_test_commit_snapshot(&runtime, &committed);
        drop((committed, runtime));
        let (recovered, _) = recover(execution, &medium);
        let (_, predecessor) = recovered
            .observe_branch(&recovered.branch_identity(&predecessor_branch).unwrap())
            .unwrap();
        let (_, basis) = recovered
            .observe_branch(&recovered.main_branch_identity())
            .unwrap();
        recovered
            .rebuild_authored_write_generation(
                *execution,
                recovered
                    .select_exact_commit(&basis.observation(), actual_artifact.commit_id())
                    .into_outcome()
                    .unwrap(),
            )
            .unwrap();
        let artifacts = recovered
            .durability()
            .canonical_commit_artifacts(&execution)
            .map(|artifact| artifact.unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            artifacts.last().unwrap().as_bytes(),
            actual_artifact.as_bytes()
        );
        let restored = Artifact::readmit(artifacts.last().unwrap().as_bytes(), execution).unwrap();
        let restored_body =
            serde_json::from_str::<Value>(&serde_json::to_string(restored.envelope()).unwrap())
                .unwrap();
        assert_eq!(
            restored_body["index_definition_delta"],
            original_body["index_definition_delta"]
        );
        assert_eq!(
            restored_body["authored_index_writes"],
            original_body["authored_index_writes"]
        );
        assert_eq!(
            recovered
                .index_access()
                .definition_lookup_snapshot()
                .definition_count(),
            2
        );
        for (definition, expected_complete) in [(original, true), (untouched, false)] {
            let DerivedIndexKind::EntityField { field_locator } = definition.kind else {
                unreachable!()
            };
            let key = RelationalAuthoredWriteSelector::EntityFieldIndex {
                definition: RelationalAuthoredEntityIndexDefinition {
                    index_id: definition.index_id,
                    name: definition.name,
                    branch_scoped: definition.branch_scoped,
                    locator: field_locator,
                },
                kind: KindId(1),
                value: AuthoritativeFieldComparisonKey::from_aspect_value(
                    &worth_foundational::facade::AspectValue::String(
                        "never-a-proven-absence".into(),
                    ),
                ),
            };
            let selected_history = recovered.authored_write_history_cursor(
                *execution,
                &predecessor,
                recovered
                    .select_exact_commit(&basis.observation(), actual_artifact.commit_id())
                    .into_outcome()
                    .unwrap(),
                &[key],
            );
            if expected_complete {
                assert!(
                    selected_history.is_ok(),
                    "original prefix remains actually complete: {selected_history:?}"
                );
            } else {
                assert!(
                    matches!(
                        selected_history,
                        Err(RelationalAuthoredWriteDenial::IncompleteIndexHistory)
                    ),
                    "later definition has no coverage; touching={touching}"
                );
            }
        }
    }
}

#[test]
fn same_position_checkpoint_catalog_cannot_regress_selection_or_ack_later_registration() {
    use worth_relational::facade::durability::{
        RelationalCheckpointAcknowledgementDenial, RelationalCheckpointArtifactPolicy,
        RelationalDurableColdSource,
    };
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity(execution, &runtime, "checkpoint-position");
    let source: Arc<dyn RelationalDurableColdSource> = medium.clone();
    let capture = || {
        runtime
            .durability_authority()
            .native_checkpoint_with_policy(
                execution,
                execution.allocation_policy(),
                RelationalCheckpointArtifactPolicy::ExternalMembers {
                    current_members: &[],
                    source: Some(source.clone()),
                },
            )
            .unwrap()
    };
    let earlier = capture();
    let installed = runtime
        .index_authority()
        .register(definition("selected-only-catalog"))
        .unwrap();
    let selected = capture();
    assert_eq!(earlier.covered_position(), selected.covered_position());
    medium.file_checkpoint(selected.clone());
    let (ack, _) = execution
        .run_native_with_checkpoint(|context| selected.acknowledge_selected(execution, context))
        .unwrap();
    ack.unwrap();
    assert!(matches!(
        earlier.validate_publication_source(&source),
        Err(RelationalCheckpointAcknowledgementDenial::StaleCapture)
    ));
    let later = runtime
        .index_authority()
        .register(definition("after-selected-capture"))
        .unwrap();
    let result = create_entity_outcome(execution, &runtime, "tail-after-selection");
    assert_eq!(
        delta(&result)["base_next_index_id"],
        installed.index_id.0 + 1
    );
    assert_eq!(
        delta(&result)["definitions"][0]["index_id"],
        later.index_id.0
    );
    assert_eq!(delta(&result)["definitions"].as_array().unwrap().len(), 1);
    release_test_commit_snapshot(&runtime, &result);
}

#[test]
fn expanded_restore_reemits_checkpoint_only_definitions_without_fabricating_selection() {
    for seeded in [false, true] {
        let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
        let execution = &ExecutionRequest::serial(&request);
        let medium = TestDurableMedium::new();
        let source = durable_runtime(&medium);
        if seeded {
            create_entity(execution, &source, "expanded-seed");
        }
        let installed = source
            .index_authority()
            .register(definition("expanded-checkpoint-only"))
            .unwrap();
        let checkpoint = source
            .durability_authority()
            .native_checkpoint(execution, execution.allocation_policy())
            .unwrap();
        let destination = TestDurableMedium::new();
        let mut restored = durable_runtime(&destination);
        restored
            .durability_recovery()
            .restore_native_checkpoint(execution, &checkpoint)
            .unwrap();
        assert_eq!(
            restored.index_access().matching_definition(&installed),
            Some(installed.clone())
        );
        // The newly attached medium has no original journal prefix, so use a
        // reconstructive memory owner for the safe original-payload comparison.
        let mut reconstructed = memory_runtime();
        reconstructed
            .durability_recovery()
            .restore_native_checkpoint(execution, &checkpoint)
            .unwrap();
        let result = create_entity_outcome(execution, &reconstructed, "expanded-next");
        assert_eq!(delta(&result)["base_next_index_id"], 1);
        assert_eq!(
            delta(&result)["definitions"][0]["index_id"],
            installed.index_id.0
        );
        release_test_commit_snapshot(&reconstructed, &result);
    }
}

#[test]
fn unselected_original_capture_cannot_acknowledge_definition_introduction() {
    use worth_relational::facade::durability::{
        RelationalCheckpointAcknowledgementDenial, RelationalCheckpointArtifactPolicy,
        RelationalDurableColdSource,
    };
    let request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    create_entity(execution, &runtime, "unselected-seed");
    let installed = runtime
        .index_authority()
        .register(definition("unselected-pending-definition"))
        .unwrap();
    let source: Arc<dyn RelationalDurableColdSource> = medium.clone();
    let capture = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: Some(source),
            },
        )
        .unwrap();
    assert!(
        capture.publication_subject().is_some(),
        "actual source-bound original capture"
    );
    let (ack, _) = execution
        .run_native_with_checkpoint(|context| capture.acknowledge_selected(execution, context))
        .unwrap();
    assert!(
        matches!(
            ack,
            Err(RelationalCheckpointAcknowledgementDenial::SelectionNotKnown)
        ),
        "the actual attached medium never selected this original capture: {ack:?}"
    );
    let result = create_entity_outcome(execution, &runtime, "after-unselected-capture");
    assert_eq!(delta(&result)["base_next_index_id"], 1);
    assert_eq!(
        delta(&result)["definitions"][0]["index_id"],
        installed.index_id.0
    );
    let actual = medium.offered().last().unwrap().as_bytes().to_vec();
    release_test_commit_snapshot(&runtime, &result);
    drop((result, capture, runtime));
    let (recovered, _) = recover(execution, &medium);
    assert_eq!(
        recovered.index_access().matching_definition(&installed),
        Some(installed)
    );
    let replayed = recovered
        .durability()
        .canonical_commit_artifacts(&execution)
        .last()
        .unwrap()
        .unwrap();
    assert_eq!(replayed.as_bytes(), actual);
}
