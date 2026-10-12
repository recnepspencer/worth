//! every downstream semantic role participates in atomic installation.
use super::*;

#[test]
fn every_downstream_semantic_role_participates_in_atomic_installation() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    type Mutation = fn(&mut domain::WorthQueryDomainOperationSemanticClosure);
    let mutations: [(&str, Mutation); 20] = [
        ("parameters", |value| {
            value.parameters = domain::WorthQueryOperationParameterContract::Declared {
                fields: vec![domain::WorthQueryOperationParameterField {
                    name: "entity".into(),
                    value_family: domain::WorthQueryOperationValueFamily::EntityIdentity,
                    required: true,
                }],
            }
        }),
        ("native-projection", |value| {
            value.native_projection = domain::WorthQueryOperationNativeProjectionContract::new(
                operation_identity_contract(2),
                worth_foundational::facade::AspectMask::whole_aspect(),
            )
            .unwrap()
        }),
        ("canonical-query", |value| {
            value.canonical_query = canonical_bundle("AlternateVertex")
        }),
        ("collection", |value| {
            value.collection = domain::WorthQueryOperationCollectionContract::Collection {
                row_identity_field: domain::WorthQueryOperationCollectionField::from_dotted(
                    "identity.id",
                )
                .expect("valid collection field"),
                ordering_fields: vec![domain::WorthQueryOperationCollectionField::from_dotted(
                    "identity.id",
                )
                .expect("valid collection field")],
                grouping: domain::WorthQueryOperationGroupingContract::Ungrouped,
                window: domain::WorthQueryOperationWindowPolicy::ContinuationBounded,
                continuation: domain::WorthQueryOperationContinuationPosture::SnapshotCursor,
            }
        }),
        ("required-capability", |value| {
            value.required_capabilities =
                vec![domain::WorthQueryOperationCapabilityRequirement::QueryRead]
        }),
        ("invariant", |value| {
            value.invariants = domain::WorthQueryOperationInvariantContract::Declared {
                invariant_slots: vec!["semantic-invariant:1".into()],
            }
        }),
        ("support", |value| {
            value.support.live = domain::WorthQuerySupportRequirement::Required
        }),
        ("workflow", |value| {
            value.workflow = domain::WorthQueryOperationWorkflowContract::Declared(
                domain::WorthQueryPortableWorkflowDefinition::new(
                    "read",
                    [domain::WorthQueryPortableWorkflowStage::new(
                        "read",
                        std::iter::empty::<&str>(),
                        true,
                        true,
                        std::iter::empty::<domain::WorthQueryOperationCapabilityRequirement>(),
                    )
                    .with_semantics(domain::WorthQueryWorkflowStageSemantics {
                        input: domain::WorthQueryWorkflowValueContract::NotRequired,
                        output: domain::WorthQueryWorkflowValueContract::Projection,
                        graph_read_roles: vec!["model".into()],
                        terminal_result_states: vec![domain::WorthQueryOperationResultState::Ready],
                        failure_classes: vec![domain::WorthQueryOperationFailureClass::Dependency],
                        resources: super::super::installed_operation_fixture::execution_resource_contract(
                        ),
                        ..Default::default()
                    })],
                ),
            )
        }),
        ("conditional-node", |value| {
            value.conditional_nodes = vec![
                domain::WorthQueryPortableConditionalNodeDeclaration::declare(
                    "drift-node",
                    domain::WorthQueryConditionalNodeRole::Computed,
                )
                .dependencies(std::iter::empty())
                .outputs([domain::WorthQueryConditionalNodeOutput::OperationOutput {
                    projection_role: domain::WorthQueryOperationProjectionRole::new("vertex")
                        .unwrap(),
                }])
                .required_context([domain::WorthQueryConditionalNodeContext::Basis])
                .evaluation(
                    domain::WorthQueryConditionalEvaluationCondition::on_demand(),
                    domain::WorthQueryConditionalTrigger::on_demand::<DriftTrigger>(),
                )
                .comparison(
                    domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
                    domain::WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue,
                )
                .artifact_policy(
                    domain::WorthQueryArtifactReuseEquivalence::NotReusable,
                    domain::WorthQueryMaintenancePosture::OnDemandOnly,
                    domain::WorthQueryArtifactPosture::Ephemeral,
                )
                .output_relationship(domain::WorthQueryOutputRelationship::IsOperationOutput)
                .finish()
                .unwrap(),
            ];
        }),
        ("graph-read", |value| match &mut value.graph_reads {
            domain::WorthQueryOperationGraphReadContract::DeclaredDomain { roles } => {
                roles[0].role = "source".into()
            }
            domain::WorthQueryOperationGraphReadContract::NotRequired => unreachable!(),
            domain::WorthQueryOperationGraphReadContract::Declared { .. } => {
                unreachable!("portable domain operations cannot carry application read scopes")
            }
        }),
        ("touch", |value| {
            value.touches = domain::WorthQueryOperationTouchContract::Declared {
                graph_roles: vec!["model".into()],
                scopes: vec![domain::WorthQueryOperationTouchScope::DeclaredDomain(
                    domain::WorthQueryDeclaredDomainTouchScopeIdentity::new("vertex").unwrap(),
                )],
            }
        }),
        ("effect", |value| {
            value.effects = domain::WorthQueryOperationEffectContract::Declared {
                effect_families: vec![domain::WorthQueryOperationEffectFamily::Mutation],
            }
        }),
        ("replay", |value| {
            value.replay = domain::WorthQueryOperationReplayContract::NotSupported
        }),
        ("lineage", |value| {
            value.lineage = domain::WorthQueryOperationLineageContract::Preserve
        }),
        ("promotion", |value| {
            value.promotion = domain::WorthQueryOperationPromotionContract::OnDurableReference
        }),
        ("publication", |value| {
            value.publication = domain::WorthQueryOperationPublicationContract::NotRequired;
            value.projection_consumption =
                domain::WorthQueryOperationProjectionConsumptionContract::NotRequired;
            value.support.projection_consumption =
                domain::WorthQuerySupportRequirement::NotRequired;
        }),
        ("result", |value| {
            value
                .terminal
                .result_states
                .retain(|state| *state != domain::WorthQueryOperationResultState::Violation)
        }),
        ("failure", |value| {
            value
                .terminal
                .failure_classes
                .push(domain::WorthQueryOperationFailureClass::Conflict)
        }),
        ("cost", |value| {
            value.cost.execution = domain::WorthQueryOperationCostClass::GraphBreadth
        }),
        ("lowering", |value| {
            value.lowering.family = "other-lowering".into()
        }),
    ];
    for (role, mutate) in mutations {
        let denial = match semantic_drift_workspace(
            request_execution,
            &format!("operation-{role}-drift"),
            mutate,
        ) {
            Ok(_) => panic!("{role} drift installed beside the original"),
            Err(denial) => denial,
        };
        assert!(
            denial.message().contains("ConflictingDomainOperation"),
            "{role} failed for the wrong reason: {}",
            denial.message()
        );
    }
}
