use super::super::support::*;

fn loop_successor_verified_profile() -> WorthQueryRuntimeSupportProfile {
    bridge_verified_direct_relation_profile("update_existing_verified")
}

#[test]
fn compose_graph_supports_verified_loop_successor_rewire_with_assumption_summary() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let binding = geometry_relation_binding(
        "authority:loop-next-rel",
        "HalfEdgeNextRelation:1",
        "HalfEdgeNextRelation",
    );
    let runtime = bridge_runtime_with_support_and_existing_truth_verification(
        request_execution,
        loop_successor_verified_profile(),
        TestExistingTruthVerificationAdapter::default()
            .with_value(
                &binding,
                "source.id",
                test_native_entity_ref_value(&test_entity_identity("he-1")),
            )
            .with_value(
                &binding,
                "target.id",
                test_native_entity_ref_value(&test_entity_identity("he-2")),
            ),
    );
    let mut workspace = runtime
        .workspace("topology.graph-composition-loop-successor-rewire")
        .expect("workspace should open");

    let receipt = workspace
        .compose_graph(request_execution, |graph| {
            let successor = graph.insert_entity("draft-half-edge", "HalfEdge", |half_edge| {
                half_edge
                    .set_aspect(test_aspect_touch("identity.id"), test_authored_string_aspect_value("he-3"))
                    .set_aspect(test_aspect_touch("kind.value"), test_authored_string_aspect_value("half_edge"))
            })?;
            graph.retarget_existing_verified(
                binding,
                |verify| {
                    verify
                        .existing_entity_identity(test_aspect_touch("source.id"), test_entity_identity("he-1"))
                        .existing_entity_identity(test_aspect_touch("target.id"), test_entity_identity("he-2"))
                },
                |update| {
                    update
                        .existing_entity_identity(test_aspect_touch("source.id"), test_entity_identity("he-1"))
                        .continuity_rebind_existing_target(crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_prior_authority(crate::runtime::WorthQueryContinuityPriorAuthorityLabel::new("authority:loop-next-rel").expect("continuity prior authority label")).expect("continuity prior authority identity"), crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_successor_authority(crate::runtime::WorthQueryContinuitySuccessorAuthorityLabel::new("authority:loop-next-rel-successor").expect("continuity successor authority label")).expect("continuity successor authority identity"),
                        )
                        .symbolic_entity_identity(test_aspect_touch("target.id"), successor.reference().clone())
                },
            )?;
            Ok(())
        })
        .expect("verified loop successor rewire should execute");
    let successor_identity = receipt.write_receipts()[0].deltas()[0]
        .entity_identity
        .clone();

    let program = receipt
        .graph_composition_program()
        .expect("graph composition receipt should expose program");
    let lifecycle = receipt
        .graph_composition_lifecycle_outcomes()
        .expect("graph composition receipt should expose lifecycle");
    let evidence = receipt
        .graph_composition_evidence()
        .expect("graph composition receipt should expose evidence");
    let assumptions = receipt
        .graph_composition_assumption_summary()
        .expect("verified graph composition should expose assumption summary");
    let resolution_map = receipt.graph_composition_resolution_map();

    assert_eq!(program.component_count(), 2);
    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .symbolic_resolution_count(),
        1
    );
    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .symbolic_target_reference_count(),
        0
    );
    assert_eq!(
        program.steps()[1].kind(),
        WorthQueryGraphCompositionProgramStepKind::ExistingTargetVerifiedRetarget
    );
    assert_eq!(
        lifecycle.entries()[1].outcome_kind(),
        WorthQueryGraphCompositionLifecycleOutcomeKind::RetargetedIdentityPreserved
    );
    assert_eq!(
        lifecycle.counter_snapshot(),
        "created=1;updated_identity_preserved=0;retargeted_identity_preserved=1;retired_current_truth=0;superseded_with_lineage=0;deleted_if_uncommitted=0;denied_before_execution=0"
    );
    assert_eq!(
        evidence.counter_snapshot(),
        "components=2;symbolic_entities=1;symbolic_relations=0;symbolic_resolutions=1;affected_live_views=0;affected_derived_views=0;considered_computed_views=0;created=1;updated_identity_preserved=0;retargeted_identity_preserved=1;retired_current_truth=0;superseded_with_lineage=0;deleted_if_uncommitted=0;denied_before_execution=0"
    );
    assert_eq!(evidence.symbolic_resolution_count(), 1);
    assert_eq!(
        assumptions.counter_snapshot(),
        "verified_steps=1;target_bindings=1;asserted_aspects=2;distinct_asserted_aspect_touches=2;cleared_assertions=0"
    );
    assert_eq!(assumptions.verified_step_count(), 1);
    assert_eq!(assumptions.assumption_snapshot_digests().len(), 1);
    assert_eq!(assumptions.verified_precondition_digests().len(), 1);
    assert_eq!(
        evidence
            .assumption_summary()
            .expect("graph composition evidence should retain assumption summary")
            .assumption_summary_digest(),
        assumptions.assumption_summary_digest()
    );
    assert_eq!(resolution_map.len(), 1);
    assert_eq!(resolution_map.entries()[0].component_index(), 1);
    assert_eq!(
        resolution_map.entries()[0].aspect_touch(),
        Some(&test_aspect_touch("target.id"))
    );
    assert_eq!(
        resolution_map.entries()[0].symbol().as_str(),
        "draft-half-edge"
    );
    assert_eq!(
        resolution_map.entries()[0].resolved_entity_identity(),
        &successor_identity
    );

    match workspace
        .inspect(request_execution, &receipt)
        .expect("receipt should inspect")
    {
        WorthQueryInspection::BatchWriteReceipt(inspection) => {
            let component = &inspection.component_operations()[1];
            let inspection_assumptions = inspection
                .graph_composition_assumption_summary()
                .expect("inspection should expose assumption summary");
            assert_eq!(
                inspection
                    .graph_composition_program()
                    .expect("inspection should expose program")
                    .steps()[1]
                    .kind(),
                WorthQueryGraphCompositionProgramStepKind::ExistingTargetVerifiedRetarget
            );
            assert_eq!(
                inspection
                    .graph_composition_lifecycle_outcomes()
                    .expect("inspection should expose lifecycle")
                    .entries()[1]
                    .outcome_kind(),
                WorthQueryGraphCompositionLifecycleOutcomeKind::RetargetedIdentityPreserved
            );
            assert_eq!(
                inspection
                    .batch_mutation_evidence()
                    .symbolic_resolution_count(),
                1
            );
            assert_eq!(
                inspection
                    .graph_composition_evidence()
                    .expect("inspection should expose composition evidence")
                    .symbolic_resolution_count(),
                1
            );
            assert_eq!(
                inspection_assumptions.assumption_summary_digest(),
                assumptions.assumption_summary_digest()
            );
            assert_eq!(inspection.graph_composition_resolution_map().len(), 1);
            assert_eq!(
                inspection.graph_composition_resolution_map().entries()[0].aspect_touch(),
                Some(&test_aspect_touch("target.id"))
            );
            assert_eq!(
                component
                    .existing_truth_assertion_evidence()
                    .expect("verified rewire should retain assertion evidence")
                    .verified_assumption_set()
                    .expect("verified rewire should retain assumption set")
                    .assumption_snapshot_digest(),
                assumptions.assumption_snapshot_digests()[0]
            );
            assert_eq!(
                component
                    .continuity_mutation_evidence()
                    .expect("verified rewire should retain continuity evidence")
                    .outcome_class(),
                WorthQueryContinuityOutcomeClass::ContinuesAsSingleSuccessor
            );
            assert_eq!(component.symbolic_aspect_resolution_evidence().len(), 1);
            assert_eq!(
                component.symbolic_aspect_resolution_evidence()[0].aspect_touch(),
                &test_aspect_touch("target.id")
            );
            assert_eq!(
                component.symbolic_aspect_resolution_evidence()[0].resolved_entity_identity(),
                &successor_identity
            );
        }
        other => panic!("expected batch receipt inspection, got {other:?}"),
    }
}

#[test]
fn compose_graph_denies_loop_successor_rewire_when_identity_preservation_is_unavailable() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace =
        bridge_runtime_with_support(request_execution, loop_successor_verified_profile())
            .workspace("topology.graph-composition-loop-successor-identity-denial")
            .expect("workspace should open");
    let binding = workspace
        .bind_existing_relation(
            WorthQueryExistingRelationTarget::new(
                crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:loop-next-rel").expect("existing-truth authority label")).expect("existing-truth authority identity"),
                test_entity_identity("HalfEdgeNextRelation:1"),
            )
            .expect("existing relation target should build")
            .in_target_collection("HalfEdgeNextRelation")
            .expect("existing relation target collection should build"),
        )
        .expect("relation binding should build");

    let error = workspace
        .compose_graph(request_execution, |graph| {
            graph.retarget_existing_verified(
                binding,
                |verify| {
                    verify
                        .existing_entity_identity(test_aspect_touch("source.id"), test_entity_identity("he-1"))
                        .existing_entity_identity(test_aspect_touch("target.id"), test_entity_identity("he-2"))
                },
                |update| {
                    update
                        .continuity_split_successors(
                            crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_prior_authority(crate::runtime::WorthQueryContinuityPriorAuthorityLabel::new(
                                "authority:loop-next-rel",
                            )
                            .expect("continuity prior authority label")).expect("continuity prior authority identity"),
                            [
                                crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_successor_authority(crate::runtime::WorthQueryContinuitySuccessorAuthorityLabel::new(
                                    "authority:loop-next-left",
                                )
                                .expect("continuity successor authority label")).expect("continuity successor authority identity"),
                                crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_successor_authority(crate::runtime::WorthQueryContinuitySuccessorAuthorityLabel::new(
                                    "authority:loop-next-right",
                                )
                                .expect("continuity successor authority label")).expect("continuity successor authority identity"),
                            ],
                        )
                        .existing_entity_identity(test_aspect_touch("target.id"), test_entity_identity("he-3"))
                },
            )?;
            Ok(())
        })
        .expect_err("split successor continuity should deny verified retarget");

    match error {
        WorthQueryRuntimeError::GraphCompositionDenied(denial) => {
            assert_eq!(
                denial.kind(),
                WorthQueryGraphCompositionDenialKind::ExistingTargetIdentityPreservationUnavailable
            );
            assert_eq!(
                denial.failure_stage(),
                WorthQueryGraphCompositionAdmissionTraceStage::LoweringValidated
            );
            assert_eq!(denial.target_collection(), Some("HalfEdgeNextRelation"));
        }
        other => panic!("expected graph composition denial, got {other:?}"),
    }
}
