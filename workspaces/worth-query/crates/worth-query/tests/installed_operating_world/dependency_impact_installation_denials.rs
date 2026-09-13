use worth_query::facade::consumer_kit::{
    in_memory_test_runtime, WorthQueryTestBackendError, WorthQueryTestBackendErrorKind,
    WorthQueryTestBackendSchema,
};
use worth_query::facade::{domain, runtime};
use worth_relational::facade::identity::KindId;

use super::installed_operation_fixture::{
    canonical_bundle, operation_identity_contract, semantic_closure, GeometryDomain, ReadFamily,
    WorkflowRead,
};

#[test]
fn cyclic_workflow_is_denied_by_installation_before_dependency_impact_work() {
    assert_installation_denial(
        workspace_for("dependency-impact-cycle", Scenario::Cycle),
        "cyclic-or-unreachable-workflow-stage",
    );
}

#[test]
fn multiple_root_unreachable_workflow_is_denied_by_installation() {
    assert_installation_denial(
        workspace_for("dependency-impact-multiple-root", Scenario::MultipleRoot),
        "workflow-non-entry-root",
    );
}

#[test]
fn missing_predecessor_is_denied_by_installation() {
    assert_installation_denial(
        workspace_for(
            "dependency-impact-missing-predecessor",
            Scenario::MissingPredecessor,
        ),
        "missing-workflow-predecessor",
    );
}

#[test]
fn graph_read_closure_mismatch_is_denied_by_installation() {
    assert_installation_denial(
        workspace_for(
            "dependency-impact-graph-read-closure",
            Scenario::GraphReadClosure,
        ),
        "workflow-graph-read-closure-mismatch",
    );
}

#[test]
fn touch_closure_mismatch_is_denied_by_installation() {
    assert_installation_denial(
        workspace_for("dependency-impact-touch-closure", Scenario::TouchClosure),
        "workflow-touch-closure-mismatch",
    );
}

#[test]
fn effect_closure_mismatch_is_denied_by_installation() {
    assert_installation_denial(
        workspace_for("dependency-impact-effect-closure", Scenario::EffectClosure),
        "workflow-effect-closure-mismatch",
    );
}

#[test]
fn invariant_closure_mismatch_is_denied_by_installation() {
    assert_installation_denial(
        workspace_for(
            "dependency-impact-invariant-closure",
            Scenario::InvariantClosure,
        ),
        "workflow-invariant-closure-mismatch",
    );
}

#[derive(Clone, Copy)]
enum Scenario {
    Cycle,
    MultipleRoot,
    MissingPredecessor,
    GraphReadClosure,
    TouchClosure,
    EffectClosure,
    InvariantClosure,
}

fn workspace_for(
    name: &str,
    scenario: Scenario,
) -> Result<runtime::WorthQueryWorkspace, WorthQueryTestBackendError> {
    let mut semantics = semantic_closure(
        canonical_bundle("Vertex"),
        domain::WorthQuerySupportRequirement::Required,
        true,
    );
    semantics.workflow = domain::WorthQueryOperationWorkflowContract::Declared(workflow(scenario));
    match scenario {
        Scenario::TouchClosure => {
            semantics.touches = domain::WorthQueryOperationTouchContract::Declared {
                graph_roles: vec!["model".into()],
                scopes: vec![domain::WorthQueryOperationTouchScope::DeclaredDomain(
                    domain::WorthQueryDeclaredDomainTouchScopeIdentity::new("vertex").unwrap(),
                )],
            };
        }
        Scenario::EffectClosure => {
            semantics.effects = domain::WorthQueryOperationEffectContract::Declared {
                effect_families: vec![domain::WorthQueryOperationEffectFamily::Mutation],
            };
        }
        Scenario::InvariantClosure => {
            semantics.invariants = domain::WorthQueryOperationInvariantContract::Declared {
                invariant_slots: vec!["dependency-impact-invariant:1".into()],
            };
        }
        Scenario::Cycle
        | Scenario::MultipleRoot
        | Scenario::MissingPredecessor
        | Scenario::GraphReadClosure => {}
    }
    let operation = domain::WorthQueryDomainOperationDefinition::<
        GeometryDomain,
        WorkflowRead,
        ReadFamily,
    >::new(
        domain::WorthQueryDomainOperationIdentity::new("workflow-read", 1),
        semantics,
    );
    let mut package = domain::WorthQueryDomainPackage::declare(
        GeometryDomain,
        domain::WorthQueryDomainIdentityDeclaration::new(
            domain::WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
            domain::WorthQueryDomainIdentityName::new("geometry").unwrap(),
            domain::WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    );
    if matches!(scenario, Scenario::InvariantClosure) {
        package = package.invariant(domain::WorthQueryDomainInvariantDefinition::new(
            domain::WorthQueryDomainIdentityName::new("dependency-impact-invariant").unwrap(),
            domain::WorthQueryDomainSemanticVersion::new(1, 0),
            domain::WorthQueryDomainInvariantPredicate::requires_outgoing_relations(
                vec![KindId::new(0x9140_1801)],
                vec![KindId::new(0x9140_1802)],
                1,
            ),
            std::num::NonZeroU64::new(4096).unwrap(),
        ));
    }
    let package = package.operation(operation);
    let schema = WorthQueryTestBackendSchema::single_collection("Vertex")
        .aspect_contract(operation_identity_contract(1))
        .unwrap()
        .aspect("identity.id", "identity.id")
        .unwrap();

    // No executor, impact provider, operating world, or live surface is installed here.
    // The exact package-validation denial therefore proves those downstream lanes never start.
    in_memory_test_runtime()
        .with_schema(schema)
        .domain_package(package)
        .workspace(name)
}

fn workflow(scenario: Scenario) -> domain::WorthQueryPortableWorkflowDefinition {
    let stages = match scenario {
        Scenario::Cycle => vec![
            stage("start", [], false, false, no_value(), text(), true),
            stage("left", ["right"], false, false, text(), text(), true),
            stage("right", ["left"], false, false, text(), text(), true),
            stage("publish", ["left"], true, true, text(), projection(), true),
        ],
        Scenario::MultipleRoot => vec![
            stage("start", [], false, false, no_value(), text(), true),
            stage("orphan", [], false, false, no_value(), text(), true),
            stage("publish", ["start"], true, true, text(), projection(), true),
        ],
        Scenario::MissingPredecessor => vec![
            stage("start", [], false, false, no_value(), text(), true),
            stage(
                "publish",
                ["missing"],
                true,
                true,
                text(),
                projection(),
                true,
            ),
        ],
        Scenario::GraphReadClosure => valid_stages(false),
        Scenario::TouchClosure | Scenario::EffectClosure | Scenario::InvariantClosure => {
            valid_stages(true)
        }
    };
    domain::WorthQueryPortableWorkflowDefinition::new("start", stages)
}

fn valid_stages(reads_graph: bool) -> Vec<domain::WorthQueryPortableWorkflowStage> {
    vec![
        stage("start", [], false, false, no_value(), text(), false),
        stage(
            "publish",
            ["start"],
            true,
            true,
            text(),
            projection(),
            reads_graph,
        ),
    ]
}

fn stage(
    identity: &str,
    predecessors: impl IntoIterator<Item = &'static str>,
    terminal: bool,
    publishable: bool,
    input: domain::WorthQueryWorkflowValueContract,
    output: domain::WorthQueryWorkflowValueContract,
    reads_graph: bool,
) -> domain::WorthQueryPortableWorkflowStage {
    domain::WorthQueryPortableWorkflowStage::new(
        identity,
        predecessors,
        terminal,
        publishable,
        std::iter::empty::<domain::WorthQueryOperationCapabilityRequirement>(),
    )
    .with_semantics(domain::WorthQueryWorkflowStageSemantics {
        input,
        output,
        graph_read_roles: reads_graph.then_some("model".into()).into_iter().collect(),
        resources: super::installed_operation_fixture::execution_resource_contract(),
        terminal_result_states: terminal
            .then_some(domain::WorthQueryOperationResultState::Ready)
            .into_iter()
            .collect(),
        failure_classes: vec![domain::WorthQueryOperationFailureClass::Dependency],
        ..Default::default()
    })
}

fn assert_installation_denial(
    outcome: Result<runtime::WorthQueryWorkspace, WorthQueryTestBackendError>,
    reason: &str,
) {
    let denial = match outcome {
        Ok(_) => panic!("invalid workflow reached downstream workspace construction"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        WorthQueryTestBackendErrorKind::DomainInstallationFailed
    );
    assert_eq!(
        denial.message(),
        format!(
            "failed to validate in-memory test domain: \
             InvalidDomainOperation: workflow-read:1:{reason}"
        )
    );
}

const fn no_value() -> domain::WorthQueryWorkflowValueContract {
    domain::WorthQueryWorkflowValueContract::NotRequired
}

const fn text() -> domain::WorthQueryWorkflowValueContract {
    domain::WorthQueryWorkflowValueContract::Text
}

const fn projection() -> domain::WorthQueryWorkflowValueContract {
    domain::WorthQueryWorkflowValueContract::Projection
}
