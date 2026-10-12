use super::super::support::*;

fn task_relation_runtime(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryRuntime {
    stateful_bridge_task_relation_runtime(request_execution)
}

fn verified_profile() -> WorthQueryRuntimeSupportProfile {
    WorthQueryRuntimeSupportProfile::bridge_backed(
        "test-subscription-activation",
        "test-preview-basis",
        "test-inspector-evidence",
    )
    .with_bridge_backed_verification_support(
        "update_existing_verified",
        "direct_relation_identity",
        true,
        true,
        None,
    )
}

#[path = "graph_composition_supersede_existing/compose_graph_supports_existing_target_supersession_lifecycle.rs"]
mod compose_graph_supports_existing_target_supersession_lifecycle;

#[test]
fn compose_graph_supports_verified_existing_target_supersession_lifecycle() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let binding = WorthQueryExistingRelationTarget::new(
        crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(
            crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:rel-edge")
                .expect("existing-truth authority label"),
        )
        .expect("existing-truth authority identity"),
        test_entity_identity("TaskRelation:1"),
    )
    .expect("existing relation target should build")
    .in_target_collection("TaskRelation")
    .expect("existing relation target collection should build");
    let binding = WorthQueryExistingTruthTargetBinding::from_relation_target(binding)
        .expect("relation binding should build");
    let runtime = bridge_runtime_with_support_and_existing_truth_verification(
        request_execution,
        verified_profile(),
        TestExistingTruthVerificationAdapter::default()
            .with_value(
                &binding,
                "source.id",
                test_native_entity_ref_value(&test_entity_identity("vertex-a")),
            )
            .with_value(
                &binding,
                "target.id",
                test_native_entity_ref_value(&test_entity_identity("vertex-b")),
            ),
    );
    let mut workspace = runtime
        .workspace("tasks.graph-composition-verified-supersede")
        .expect("workspace should open");

    let receipt = workspace
        .compose_graph(request_execution, |graph| {
            graph.supersede_existing_verified(
                binding,
                |verify| {
                    verify
                        .existing_entity_identity(test_aspect_touch("source.id"), test_entity_identity("vertex-a"))
                        .existing_entity_identity(test_aspect_touch("target.id"), test_entity_identity("vertex-b"))
                },
                |update| {
                    update
                        .continuity_rebind_merge_successor(crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_prior_authority(crate::runtime::WorthQueryContinuityPriorAuthorityLabel::new("authority:rel-edge").expect("continuity prior authority label")).expect("continuity prior authority identity"), crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_successor_authority(crate::runtime::WorthQueryContinuitySuccessorAuthorityLabel::new("authority:rel-edge-merged").expect("continuity successor authority label")).expect("continuity successor authority identity"),
                        )
                        .existing_entity_identity(test_aspect_touch("target.id"), test_entity_identity("vertex-merged"))
                },
            )?;
            Ok(())
        })
        .expect("verified supersession program should execute");

    let program = receipt
        .graph_composition_program()
        .expect("graph composition receipt should expose program");
    let lifecycle = receipt
        .graph_composition_lifecycle_outcomes()
        .expect("graph composition receipt should expose lifecycle");

    assert_eq!(
        program.steps()[0].kind(),
        WorthQueryGraphCompositionProgramStepKind::ExistingTargetVerifiedSupersession
    );
    assert_eq!(
        lifecycle.entries()[0].outcome_kind(),
        WorthQueryGraphCompositionLifecycleOutcomeKind::SupersededWithLineage
    );
    assert_eq!(
        lifecycle.counter_snapshot(),
        "created=0;updated_identity_preserved=0;retargeted_identity_preserved=0;retired_current_truth=0;superseded_with_lineage=1;deleted_if_uncommitted=0;denied_before_execution=0"
    );
    assert_eq!(
        receipt
            .graph_composition_evidence()
            .expect("graph composition receipt should expose evidence")
            .counter_snapshot(),
        "components=1;symbolic_entities=0;symbolic_relations=0;symbolic_resolutions=0;affected_live_views=0;affected_derived_views=0;considered_computed_views=0;created=0;updated_identity_preserved=0;retargeted_identity_preserved=0;retired_current_truth=0;superseded_with_lineage=1;deleted_if_uncommitted=0;denied_before_execution=0"
    );

    match workspace
        .inspect(request_execution, &receipt)
        .expect("receipt should inspect")
    {
        WorthQueryInspection::BatchWriteReceipt(inspection) => {
            let component = &inspection.component_operations()[0];
            assert_eq!(
                component
                    .existing_truth_assertion_evidence()
                    .expect("verified supersession should retain assertion evidence")
                    .mode(),
                WorthQueryExistingTruthAssertionMode::BackendVerifiedAssertion
            );
            assert_eq!(
                component
                    .continuity_mutation_evidence()
                    .expect("verified supersession should retain continuity evidence")
                    .outcome_class(),
                WorthQueryContinuityOutcomeClass::ContinuesViaTruthLoweredCanonicalMergeSuccessor
            );
        }
        other => panic!("expected batch receipt inspection, got {other:?}"),
    }
}

#[test]
fn compose_graph_denies_existing_target_supersession_without_lineage_intent() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = task_relation_runtime(request_execution)
        .workspace("tasks.graph-composition-supersede-denial")
        .expect("runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "tasks.graph-composition-supersede-denial-relations",
            |q| {
                q.from("TaskRelation")
                    .select([
                        crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                            .unwrap(),
                        crate::authoring::AspectFieldKey::from_authoring_parts("kind", "value")
                            .unwrap(),
                        crate::authoring::AspectFieldKey::from_authoring_parts("source", "id")
                            .unwrap(),
                        crate::authoring::AspectFieldKey::from_authoring_parts("target", "id")
                            .unwrap(),
                    ])
                    .order_by(
                        crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                            .unwrap(),
                    )
                    .schema_basis("tasks-graph-composition-supersede-denial-relations")
            },
        )
        .expect("relation live view should declare");
    let seed = workspace
        .insert(request_execution, "TaskRelation", |relation| {
            relation
                .set_aspect(
                    test_aspect_touch("identity.id"),
                    test_authored_string_aspect_value("rel-edge"),
                )
                .set_aspect(
                    test_aspect_touch("kind.value"),
                    test_authored_string_aspect_value("edge"),
                )
                .existing_entity_identity(
                    test_aspect_touch("source.id"),
                    test_entity_identity("vertex-a"),
                )
                .existing_entity_identity(
                    test_aspect_touch("target.id"),
                    test_entity_identity("vertex-b"),
                )
        })
        .expect("seed insert should execute");
    let binding = workspace
        .bind_existing_relation(
            WorthQueryExistingRelationTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:rel-edge").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                seed.deltas()[0].entity_identity.clone(),
            )
            .expect("existing relation target should build")
            .in_target_collection("TaskRelation")
            .expect("existing relation target collection should build"),
        )
        .expect("relation binding should build");

    let error = workspace
        .compose_graph(request_execution, |graph| {
            graph.supersede_existing(binding, |relation| {
                relation
                    .continuity_rebind_existing_target(crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_prior_authority(crate::runtime::WorthQueryContinuityPriorAuthorityLabel::new("authority:rel-edge").expect("continuity prior authority label")).expect("continuity prior authority identity"), crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_successor_authority(crate::runtime::WorthQueryContinuitySuccessorAuthorityLabel::new("authority:rel-edge-successor").expect("continuity successor authority label")).expect("continuity successor authority identity"),
                    )
                    .existing_entity_identity(test_aspect_touch("target.id"), test_entity_identity("vertex-c"))
            })?;
            Ok(())
        })
        .expect_err("single-successor continuity should not impersonate supersession");

    match error {
        WorthQueryRuntimeError::GraphCompositionDenied(denial) => {
            assert_eq!(
                denial.kind(),
                WorthQueryGraphCompositionDenialKind::ExistingTargetSupersessionUnsupported
            );
            assert_eq!(
                denial.failure_stage(),
                WorthQueryGraphCompositionAdmissionTraceStage::LoweringValidated
            );
            assert_eq!(denial.target_collection(), Some("TaskRelation"));
        }
        other => panic!("expected graph composition denial, got {other:?}"),
    }
}
