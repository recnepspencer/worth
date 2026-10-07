use worth_query::facade::domain;

use super::world::{canonical_bundle, execution_resource_contract, semantic_closure};
use super::{PhaseDomain, PhaseFamily, PhaseOperation};

pub(super) fn package() -> domain::WorthQueryDomainPackage<PhaseDomain> {
    let mut semantics = semantic_closure(
        canonical_bundle("Vertex"),
        domain::WorthQuerySupportRequirement::Required,
        true,
    );
    semantics.effects = domain::WorthQueryOperationEffectContract::Declared {
        effect_families: vec![domain::WorthQueryOperationEffectFamily::Mutation],
    };
    semantics.touches = domain::WorthQueryOperationTouchContract::Declared {
        graph_roles: vec!["model".into()],
        scopes: vec![domain::WorthQueryOperationTouchScope::DeclaredDomain(
            domain::WorthQueryDeclaredDomainTouchScopeIdentity::new("vertex").unwrap(),
        )],
    };
    semantics.lowering.family = "workflow-fold-v1".into();
    let mut stages = vec![stage("start", [], false)];
    stages.extend(super::MEMBERS.map(|name| stage(name, ["start"], false)));
    stages.push(stage("publish", super::MEMBERS, true));
    semantics.workflow = domain::WorthQueryOperationWorkflowContract::Declared(
        domain::WorthQueryPortableWorkflowDefinition::new("start", stages),
    );
    let operation = domain::WorthQueryDomainOperationDefinition::<
        PhaseDomain,
        PhaseOperation,
        PhaseFamily,
    >::new(
        domain::WorthQueryDomainOperationIdentity::new("workflow-fold", 1),
        semantics,
    );
    domain::WorthQueryDomainPackage::declare(
        PhaseDomain,
        domain::WorthQueryDomainIdentityDeclaration::new(
            domain::WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
            domain::WorthQueryDomainIdentityName::new("geometry").unwrap(),
            domain::WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
    .operation(operation)
}

fn stage(
    name: &str,
    predecessors: impl IntoIterator<Item = &'static str>,
    terminal: bool,
) -> domain::WorthQueryPortableWorkflowStage {
    use domain::{WorthQueryWorkflowCostRole as Cost, WorthQueryWorkflowValueContract as Value};
    let member = super::MEMBERS.contains(&name);
    let reads = member || terminal;
    let mut costs = vec![Cost::Admission, Cost::Execution, Cost::ResultValidation];
    if reads {
        costs.push(Cost::GraphRead);
    }
    if member {
        costs.push(Cost::Effect);
    }
    domain::WorthQueryPortableWorkflowStage::new(
        name,
        predecessors,
        terminal,
        terminal,
        std::iter::empty::<domain::WorthQueryOperationCapabilityRequirement>(),
    )
    .with_semantics(domain::WorthQueryWorkflowStageSemantics {
        input: if name == "start" {
            Value::NotRequired
        } else {
            Value::Text
        },
        output: if terminal {
            Value::Projection
        } else {
            Value::Text
        },
        graph_read_roles: reads.then_some("model".into()).into_iter().collect(),
        touch_roles: member.then_some("model".into()).into_iter().collect(),
        effect_roles: member
            .then_some(domain::WorthQueryOperationEffectFamily::Mutation)
            .into_iter()
            .collect(),
        cost_roles: costs,
        resources: execution_resource_contract(),
        terminal_result_states: terminal
            .then_some(domain::WorthQueryOperationResultState::Ready)
            .into_iter()
            .collect(),
        failure_classes: vec![domain::WorthQueryOperationFailureClass::Dependency],
        ..Default::default()
    })
}
