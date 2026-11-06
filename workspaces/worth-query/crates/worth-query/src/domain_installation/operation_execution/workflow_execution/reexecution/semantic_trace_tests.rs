use super::super::workflow_conditional_trace::WorthQueryConditionalObservationMeaning;
use super::*;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[test]
fn omitted_stage_and_typed_output_drift_localize_without_digest_oracles() {
    let original = trace(vec![stage("start"), stage("finish")]);
    let omitted = trace(vec![stage("start")]);
    assert_eq!(
        compare_exact_workflow_traces(&original, &omitted, Default::default()),
        WorthQueryReplayComparison::Diverged(WorthQueryReplayDivergence::StageSet)
    );

    let mut changed = trace(vec![stage("start"), stage("finish")]);
    changed.stages[1].output = WorthQueryWorkflowSemanticValue::Text("changed".into());
    assert_eq!(
        compare_exact_workflow_traces(&original, &changed, Default::default()),
        WorthQueryReplayComparison::Diverged(WorthQueryReplayDivergence::Output {
            stage: "finish".into()
        })
    );
}

#[test]
fn effect_drift_is_never_hidden_by_the_diagnostic_noise_allowance() {
    let original = trace(vec![stage("start")]);
    let mut changed = trace(vec![stage("start")]);
    changed.stages[0]
        .effects
        .push(WorthQueryEffectTraceMeaning {
            family: WorthQueryOperationEffectFamily::Mutation,
            mutation: None,
        });
    assert_eq!(
        compare_exact_workflow_traces(
            &original,
            &changed,
            WorthQueryReplayNoiseContract {
                diagnostic_warnings: true
            }
        ),
        WorthQueryReplayComparison::Diverged(WorthQueryReplayDivergence::Effect {
            stage: "start".into()
        })
    );
}

#[test]
fn invariant_identity_cannot_hide_different_effect_coverage() {
    let mut original = trace(vec![stage("mutate")]);
    original.stages[0].effects = vec![mutation_effect(1), mutation_effect(2)];
    original.stages[0].invariants = vec![WorthQueryInvariantTraceMeaning {
        invariant_role: "mutation-accepted".into(),
        installed_invariant_identity: "invariant:v1".into(),
        effect_indices: vec![0],
    }];
    let mut changed = original.clone();
    changed.stages[0].invariants[0].effect_indices = vec![1];

    assert_eq!(
        compare_exact_workflow_traces(&original, &changed, Default::default()),
        WorthQueryReplayComparison::Diverged(WorthQueryReplayDivergence::Invariant {
            stage: "mutate".into()
        })
    );
}

#[test]
fn same_effect_family_on_a_different_target_is_semantic_drift() {
    let mut original = trace(vec![stage("mutate")]);
    original.stages[0].effects.push(mutation_effect(1));
    let mut changed = trace(vec![stage("mutate")]);
    changed.stages[0].effects.push(mutation_effect(2));

    assert_eq!(
        compare_exact_workflow_traces(&original, &changed, Default::default()),
        WorthQueryReplayComparison::Diverged(WorthQueryReplayDivergence::Effect {
            stage: "mutate".into()
        })
    );
}

fn mutation_effect(local_slot: u64) -> WorthQueryEffectTraceMeaning {
    let target = crate::memory_workspace::WorthQueryEntityIdentity::from_relational_record(
        worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(1, local_slot, 0),
    );
    WorthQueryEffectTraceMeaning {
        family: WorthQueryOperationEffectFamily::Mutation,
        mutation: Some(WorthQueryMutationTraceMeaning {
            target_entity: Some(target),
            target_collection: Some("Vertex".into()),
            deltas: Vec::new(),
            declared_aspect_operations: Vec::new(),
            declared_aspect_value_digest: None,
            naming: None,
            continuity: None,
        }),
    }
}

#[test]
fn only_explicit_diagnostic_noise_can_converge() {
    let original = trace(vec![stage("start")]);
    let mut changed = trace(vec![stage("start")]);
    changed.stages[0]
        .warnings
        .push(WorthQueryWorkflowStageWarning::Advisory(
            "provider timing".into(),
        ));
    assert!(matches!(
        compare_exact_workflow_traces(&original, &changed, Default::default()),
        WorthQueryReplayComparison::Diverged(WorthQueryReplayDivergence::Diagnostic { .. })
    ));
    assert_eq!(
        compare_exact_workflow_traces(
            &original,
            &changed,
            WorthQueryReplayNoiseContract {
                diagnostic_warnings: true
            }
        ),
        WorthQueryReplayComparison::Equivalent
    );
}

#[test]
fn lineage_divergence_is_typed_and_localized_to_its_stage() -> Result<(), IdentityDerivationDenial>
{
    let original = trace(vec![stage("publish")]);
    let mut changed = trace(vec![stage("publish")]);
    changed.stages[0]
        .lineage
        .push(WorthQueryLineageTraceMeaning {
            outcome: lineage_outcome()?,
            effect_indices: vec![0],
        });
    assert_eq!(
        compare_exact_workflow_traces(&original, &changed, Default::default()),
        WorthQueryReplayComparison::Diverged(WorthQueryReplayDivergence::Lineage {
            stage: "publish".into()
        })
    );

    Ok(())
}

#[test]
fn operational_signal_identity_is_not_replay_semantics() {
    let original_meaning = conditional_meaning("signal:attempt-1", condition(true), "truth-a");
    let mut replay_meaning = original_meaning.clone();
    replay_meaning.signal_projection = "signal:attempt-2".into();
    let mut original = trace(Vec::new());
    original.conditional_path.push(original_meaning);
    let mut replay = trace(Vec::new());
    replay.conditional_path.push(replay_meaning);

    assert_eq!(
        compare_exact_workflow_traces(&original, &replay, Default::default()),
        WorthQueryReplayComparison::Equivalent
    );
    assert_ne!(
        original.conditional_path[0].signal_projection(),
        replay.conditional_path[0].signal_projection()
    );
}

#[test]
fn condition_outcome_and_observation_drift_are_semantic() {
    let original = conditional_meaning("signal:original", condition(true), "truth-a");

    let mut condition_drift = original.clone();
    condition_drift.declaration = condition(false);
    assert_conditional_drift(&original, condition_drift);

    let mut outcome_drift = original.clone();
    outcome_drift.outcome =
        crate::domain_installation::WorthQueryConditionalOutcomeClass::Suppressed;
    assert_conditional_drift(&original, outcome_drift);

    let mut observation_drift = original.clone();
    observation_drift.observations[0].current = Some(validated_truth("truth-b"));
    assert_conditional_drift(&original, observation_drift);
}

fn assert_conditional_drift(
    original: &WorthQueryConditionalTraceMeaning,
    candidate: WorthQueryConditionalTraceMeaning,
) {
    assert_ne!(original, &candidate);
    assert_ne!(
        super::super::workflow_conditional_trace::conditional_meaning_semantic_material(original),
        super::super::workflow_conditional_trace::conditional_meaning_semantic_material(&candidate),
    );
    let mut original_trace = trace(Vec::new());
    original_trace.conditional_path.push(original.clone());
    let mut candidate_trace = trace(Vec::new());
    candidate_trace.conditional_path.push(candidate);
    assert_eq!(
        compare_exact_workflow_traces(&original_trace, &candidate_trace, Default::default()),
        WorthQueryReplayComparison::Diverged(WorthQueryReplayDivergence::OperationConditionalPath)
    );
}

fn conditional_meaning(
    signal_identity: &str,
    declaration: worth_query_installation::facade::WorthQueryPortableConditionalNodeDeclaration,
    truth: &str,
) -> WorthQueryConditionalTraceMeaning {
    WorthQueryConditionalTraceMeaning {
        location: worth_query_installation::facade::WorthQueryConditionalNodeLocation::operation(
            "gate",
        )
        .unwrap(),
        declaration,
        outcome: crate::domain_installation::WorthQueryConditionalOutcomeClass::ComputedChanged,
        artifact_reuse_admitted: false,
        signal_projection: signal_identity.into(),
        observations: vec![WorthQueryConditionalObservationMeaning {
            dependency_ordinal: 0,
            previous: None,
            current: Some(validated_truth(truth)),
        }],
    }
}

fn condition(
    always: bool,
) -> worth_query_installation::facade::WorthQueryPortableConditionalNodeDeclaration {
    use worth_query_installation::facade as installation;
    let dependency = conditional_dependency();
    installation::WorthQueryPortableConditionalNodeDeclaration::declare(
        "gate",
        installation::WorthQueryConditionalNodeRole::OperationGate,
    )
    .dependencies([dependency.clone()])
    .outputs([
        installation::WorthQueryConditionalNodeOutput::OperationOutput {
            projection_role: installation::WorthQueryOperationProjectionRole::new("vertex")
                .unwrap(),
        },
    ])
    .required_context([installation::WorthQueryConditionalNodeContext::Snapshot])
    .evaluation(
        if always {
            installation::WorthQueryConditionalEvaluationCondition::always_eligible()
        } else {
            installation::WorthQueryConditionalEvaluationCondition::aspect_filtered([dependency])
                .unwrap()
        },
        installation::WorthQueryConditionalTrigger::DependencyChange,
    )
    .comparison(
        installation::WorthQueryComparatorRequirement::ExactCanonicalValue,
        installation::WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue,
    )
    .artifact_policy(
        installation::WorthQueryArtifactReuseEquivalence::NotReusable,
        installation::WorthQueryMaintenancePosture::LazyUntilObserved,
        installation::WorthQueryArtifactPosture::Ephemeral,
    )
    .output_relationship(installation::WorthQueryOutputRelationship::ContributesToOperationOutput)
    .finish()
    .unwrap()
}

fn conditional_dependency() -> worth_query_installation::facade::WorthQuerySemanticTruthDependency {
    use worth_foundational::facade::{
        AspectContract, AspectContractRevision, AspectIdentity, AspectKey, AspectMask, FieldKey,
        ScalarAspectType,
    };
    worth_query_installation::facade::WorthQuerySemanticTruthDependency::new(
        worth_query_installation::facade::WorthQueryConditionalGraphReadRole::new("model").unwrap(),
        AspectContract::scalar(
            AspectKey::new("truth").unwrap(),
            AspectIdentity(91),
            AspectContractRevision(1),
            ScalarAspectType::String,
        ),
        AspectMask::whole_aspect(),
        worth_relational::facade::schema::AspectBinding::EntityField {
            field: FieldKey::new("truth").unwrap(),
        },
        worth_query_installation::facade::WorthQuerySemanticLocality::SourceRecord,
        [worth_relational::facade::schema::RelationalAspectChangeKind::FieldSet],
    )
    .unwrap()
}

fn validated_truth(truth: &str) -> worth_foundational::facade::ContractValidatedAspectArtifact {
    let contract = conditional_dependency().contract().clone();
    worth_foundational::facade::validate_aspect_value(
        &contract,
        worth_foundational::facade::AspectValue::String(truth.into()).into(),
    )
    .into_result()
    .unwrap()
}

fn lineage_outcome() -> Result<InstalledIdentityEvolutionOutcome, IdentityDerivationDenial> {
    let continuity = crate::runtime::WorthQueryContinuityMutationEvidence::test_only(
        crate::runtime::WorthQueryContinuityMutationFamily::RebindExistingTarget,
        crate::runtime::WorthQueryContinuityOutcomeClass::ContinuesAsSingleSuccessor,
        "entity:1",
        vec!["entity:2".into()],
        None,
        Some("Vertex"),
    )?;
    let context =
        crate::identity_evolution::IdentityEvolutionQueryContext::lineage_traversal_for_test(
            crate::identity::CanonicalQueryDigest::from_parts(&["semantic-trace-query".into()]),
            crate::identity::BasisDigest::from_parts(&["semantic-trace-basis".into()]),
            crate::identity_evolution::LineageTraversalDescriptor::direct_successor_exact(
                continuity
                    .prior_authoritative_identity()
                    .evidence_identity()
                    .as_str(),
                continuity.successor_authoritative_identities()[0]
                    .evidence_identity()
                    .as_str(),
            ),
        );
    let admitted = crate::identity_evolution::admit_identity_evolution_query(context)
        .expect("test lineage admission");
    let artifact = crate::identity_evolution::execute_admitted_identity_evolution_query(&admitted)
        .expect("test lineage execution");
    Ok(InstalledIdentityEvolutionOutcome::from_execution(
        artifact,
        Some(continuity),
        None,
        crate::identity_evolution::InstalledIdentityEvolutionBinding {
            operation_identity: "operation",
            run_identity: "run",
            stage_identity: "publish",
            effect_receipt_identity: "effect:1".into(),
            establishing_entity_targets: Vec::new(),
        },
    )
    .expect("matching engine and mutation continuity evidence"))
}

fn trace(stages: Vec<WorthQueryWorkflowStageTraceSemantics>) -> WorthQueryWorkflowTraceSemantics {
    WorthQueryWorkflowTraceSemantics {
        operation_identity: "operation".into(),
        conditional_path: Vec::new(),
        stages,
        publication: None,
    }
}

fn stage(identity: &str) -> WorthQueryWorkflowStageTraceSemantics {
    WorthQueryWorkflowStageTraceSemantics {
        stage_identity: identity.into(),
        predecessor_stage_identities: Vec::new(),
        input: WorthQueryWorkflowSemanticValue::NotRequired,
        output: WorthQueryWorkflowSemanticValue::Text(identity.into()),
        result_state: None,
        warnings: Vec::new(),
        effects: Vec::new(),
        invariants: Vec::new(),
        conditional_path: Vec::new(),
        lineage: Vec::new(),
        domain_evidence: None,
    }
}
