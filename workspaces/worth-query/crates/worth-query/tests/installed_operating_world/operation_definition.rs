use worth_query::facade::domain;

use super::installed_operation_fixture::{
    canonical_bundle, conflicting_workspace, lowering_mismatch_workspace,
    mismatched_cost_workspace, mismatched_determinism_workspace, mismatched_read_plan_workspace,
    operation_identity_contract, semantic_drift_workspace, unsupported_direct_effect_workspace,
    workspace, GeometryDomain, ReadFamily, ReadVertex, ReadVertexLookalike,
};

struct DriftTrigger;
impl domain::WorthQueryOnDemandTriggerFamily for DriftTrigger {
    const PORTABLE_IDENTITY: &'static str = "worth.tests.triggers.operation-drift";
}

#[test]
fn declaration_order_converges_to_one_installed_operation_meaning() {
    let direct = workspace("operation-order-direct", false).unwrap();
    let reversed = workspace("operation-order-reversed", true).unwrap();
    let direct_receipt = direct.domain_installation_receipt(GeometryDomain).unwrap();
    let reversed_receipt = reversed
        .domain_installation_receipt(GeometryDomain)
        .unwrap();
    assert_eq!(
        direct_receipt.package_identity(),
        reversed_receipt.package_identity()
    );

    let direct_domain = direct.domain(GeometryDomain).unwrap();
    let reversed_domain = reversed.domain(GeometryDomain).unwrap();
    let direct_world = direct
        .observe_operating_world(direct.current_world())
        .unwrap();
    let reversed_world = reversed
        .observe_operating_world(reversed.current_world())
        .unwrap();
    let direct_operation = direct_world
        .family(ReadFamily)
        .bind(&direct_domain, ReadVertex)
        .unwrap();
    let reversed_operation = reversed_world
        .family(ReadFamily)
        .bind(&reversed_domain, ReadVertex)
        .unwrap();
    assert_eq!(
        direct_operation.definition(),
        reversed_operation.definition()
    );
    assert_eq!(
        (
            direct_operation.binding_counters().authority_checks,
            direct_operation.binding_counters().operation_lookups,
            direct_operation.binding_counters().graph_binding_lookups,
        ),
        (1, 1, 1)
    );
}

#[test]
fn executor_lowering_family_must_match_installed_semantics() {
    let denial = match lowering_mismatch_workspace("operation-lowering-mismatch") {
        Ok(_) => panic!("foreign lowering family must not install"),
        Err(denial) => denial,
    };
    assert!(
        denial
            .message()
            .contains("executor lowering family disagrees with installed semantics"),
        "unexpected denial: {}",
        denial.message()
    );
}

#[test]
fn executor_precompiled_read_must_match_installed_canonical_semantics() {
    let denial = match mismatched_read_plan_workspace("operation-read-plan-mismatch") {
        Ok(_) => panic!("executor-authored semantic plan drift must not install"),
        Err(denial) => denial,
    };
    assert!(
        denial
            .message()
            .contains("executor read declaration disagrees with installed canonical semantics"),
        "unexpected denial: {}",
        denial.message()
    );
}

#[test]
fn executor_cost_must_match_installed_semantics() {
    let denial = match mismatched_cost_workspace("operation-cost-mismatch") {
        Ok(_) => panic!("executor cost drift must not install"),
        Err(denial) => denial,
    };
    assert!(
        denial
            .message()
            .contains("executor cost contract disagrees with installed semantics"),
        "unexpected denial: {}",
        denial.message()
    );
}

#[test]
fn executor_determinism_must_match_installed_semantics() {
    let denial = match mismatched_determinism_workspace("operation-determinism-mismatch") {
        Ok(_) => panic!("executor determinism drift must not install"),
        Err(denial) => denial,
    };
    assert!(
        denial
            .message()
            .contains("executor determinism disagrees with installed semantics"),
        "unexpected denial: {}",
        denial.message()
    );
}

#[test]
fn direct_primary_effect_requires_a_query_owned_execution_door() {
    let denial = match unsupported_direct_effect_workspace("unsupported-direct-effect") {
        Ok(_) => panic!("direct primary effect installed without an execution door"),
        Err(denial) => denial,
    };
    assert!(denial
        .message()
        .contains("direct operation declares effects without a Query-owned execution door"));
}

#[test]
fn rebuilt_execution_index_preserves_operation_authority_and_exact_shape() {
    let world = workspace("operation-index-rebuild", false).unwrap();
    let report = world.verify_domain_execution_index_rebuild();
    assert!(report.is_equivalent());
    assert_eq!(report.active_identity(), report.rebuilt_identity());
    assert_eq!(report.domain_operation_count(), 2);
    assert_eq!(report.operation_graph_participation_count(), 0);
    assert_eq!(report.operation_required_domain_count(), 0);

    let installed_domain = world.domain(GeometryDomain).unwrap();
    let operating_world = world
        .observe_operating_world(world.current_world())
        .unwrap();
    let operation = operating_world
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    assert_eq!(operation.definition().identity().name(), "read-vertex");
}

#[test]
fn one_field_semantic_drift_rejects_the_package_atomically() {
    let denial = match conflicting_workspace("operation-semantic-conflict") {
        Ok(_) => panic!("conflicting operation meaning must reject the package"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        worth_query::facade::consumer_kit::WorthQueryTestBackendErrorKind::DomainInstallationFailed
    );
    assert!(denial.message().contains("ConflictingDomainOperation"));
}

#[test]
fn marker_lookalike_misses_one_index_without_later_work() {
    let world = workspace("operation-marker-lookalike", false).unwrap();
    let domain = world.domain(GeometryDomain).unwrap();
    let operating_world = world
        .observe_operating_world(world.current_world())
        .unwrap();
    let denial = match operating_world
        .family(ReadFamily)
        .bind(&domain, ReadVertexLookalike)
    {
        Ok(_) => panic!("lookalike operation marker must not bind"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        domain::WorthQueryOperationBindingDenialKind::OperationNotInstalled
    );
    assert_eq!(
        (
            denial.counters().authority_checks,
            denial.counters().operation_lookups,
            denial.counters().planning_steps,
        ),
        (1, 1, 0)
    );
}

#[test]
fn foreign_domain_authority_denies_before_operation_lookup() {
    let owner = workspace("operation-owner", false).unwrap();
    let foreign = workspace("operation-foreign", false).unwrap();
    let foreign_domain = foreign.domain(GeometryDomain).unwrap();
    let operating_world = owner
        .observe_operating_world(owner.current_world())
        .unwrap();
    let denial = match operating_world
        .family(ReadFamily)
        .bind(&foreign_domain, ReadVertex)
    {
        Ok(_) => panic!("foreign domain authority must not bind"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        domain::WorthQueryOperationBindingDenialKind::DomainAuthority
    );
    assert_eq!(denial.counters().operation_lookups, 0);
    assert_eq!(denial.counters().planning_steps, 0);
}

#[test]
fn every_downstream_semantic_role_participates_in_atomic_installation() {
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
                        resources: super::installed_operation_fixture::execution_resource_contract(
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
        let denial = match semantic_drift_workspace(&format!("operation-{role}-drift"), mutate) {
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
