pub use worth_query::facade::domain;

#[derive(Clone, Copy, Debug)]
pub enum InvalidWorkflow {
    Cycle,
    MissingPredecessor,
    DuplicateStage,
    ExtraRoot,
    IncompleteTerminalPath,
    UndeclaredRequiredDomain,
    UnusedOperationGraphRead,
}

pub fn workflow_definition(
    invalid: Option<InvalidWorkflow>,
) -> domain::WorthQueryPortableWorkflowDefinition {
    let stages = match invalid {
        None => valid_stages(),
        Some(InvalidWorkflow::Cycle) => vec![
            stage("start", [], false, false, no_value(), text()),
            stage("left", ["right"], false, false, text(), text()),
            stage("right", ["left"], false, false, text(), text()),
            stage("publish", ["left"], true, true, text(), projection()),
        ],
        Some(InvalidWorkflow::MissingPredecessor) => vec![
            stage("start", [], false, false, no_value(), text()),
            stage("publish", ["missing"], true, true, text(), projection()),
        ],
        Some(InvalidWorkflow::DuplicateStage) => vec![
            stage("start", [], false, false, no_value(), text()),
            stage("publish", ["start"], true, true, text(), projection()),
            stage("publish", ["start"], true, true, text(), projection()),
        ],
        Some(InvalidWorkflow::ExtraRoot) => vec![
            stage("start", [], false, false, no_value(), text()),
            stage("orphan", [], false, false, no_value(), text()),
            stage("publish", ["start"], true, true, text(), projection()),
        ],
        Some(InvalidWorkflow::IncompleteTerminalPath) => vec![
            stage("start", [], false, false, no_value(), text()),
            stage("dead-end", ["start"], false, false, text(), text()),
            stage("publish", ["start"], true, true, text(), projection()),
        ],
        Some(InvalidWorkflow::UndeclaredRequiredDomain) => {
            let mut stages = valid_stages();
            stages[1] =
                stages[1]
                    .clone()
                    .with_semantics(domain::WorthQueryWorkflowStageSemantics {
                        input: text(),
                        output: text(),
                        required_domain_roles: vec![
                            domain::WorthQueryOperationRequiredDomainRole::new("auxiliary")
                                .unwrap(),
                        ],
                        graph_read_roles: vec!["model".into()],
                        cost_roles: standard_cost_roles(true),
                        failure_classes: vec![domain::WorthQueryOperationFailureClass::Dependency],
                        resources: super::super::execution_resource_contract(),
                        ..Default::default()
                    });
            stages
        }
        Some(InvalidWorkflow::UnusedOperationGraphRead) => {
            let mut stages = valid_stages();
            let publish = stages.pop().expect("valid workflow retains publication");
            stages.push(
                publish.with_semantics(domain::WorthQueryWorkflowStageSemantics {
                    input: text(),
                    output: projection(),
                    cost_roles: standard_cost_roles(false),
                    terminal_result_states: vec![domain::WorthQueryOperationResultState::Ready],
                    failure_classes: vec![domain::WorthQueryOperationFailureClass::Dependency],
                    resources: super::super::execution_resource_contract(),
                    ..Default::default()
                }),
            );
            stages
        }
    };
    domain::WorthQueryPortableWorkflowDefinition::new("start", stages)
}

pub fn valid_stages() -> Vec<domain::WorthQueryPortableWorkflowStage> {
    vec![
        stage("start", [], false, false, no_value(), text()),
        stage("left", ["start"], false, false, text(), text()),
        stage("right", ["start"], false, false, text(), text()),
        stage(
            "publish",
            ["left", "right"],
            true,
            true,
            text(),
            projection(),
        ),
    ]
}

pub fn stage(
    identity: &str,
    predecessors: impl IntoIterator<Item = &'static str>,
    terminal: bool,
    publishable: bool,
    input: domain::WorthQueryWorkflowValueContract,
    output: domain::WorthQueryWorkflowValueContract,
) -> domain::WorthQueryPortableWorkflowStage {
    let produces_projection = matches!(output, domain::WorthQueryWorkflowValueContract::Projection);
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
        graph_read_roles: produces_projection
            .then_some("model".into())
            .into_iter()
            .collect(),
        cost_roles: standard_cost_roles(produces_projection),
        resources: super::super::execution_resource_contract(),
        terminal_result_states: terminal
            .then_some(domain::WorthQueryOperationResultState::Ready)
            .into_iter()
            .collect(),
        failure_classes: vec![domain::WorthQueryOperationFailureClass::Dependency],
        ..Default::default()
    })
}

pub fn standard_cost_roles(graph_read: bool) -> Vec<domain::WorthQueryWorkflowCostRole> {
    use domain::WorthQueryWorkflowCostRole as Role;
    let mut roles = vec![Role::Admission, Role::Execution, Role::ResultValidation];
    if graph_read {
        roles.push(Role::GraphRead);
    }
    roles
}

pub const fn no_value() -> domain::WorthQueryWorkflowValueContract {
    domain::WorthQueryWorkflowValueContract::NotRequired
}

pub const fn text() -> domain::WorthQueryWorkflowValueContract {
    domain::WorthQueryWorkflowValueContract::Text
}

pub const fn projection() -> domain::WorthQueryWorkflowValueContract {
    domain::WorthQueryWorkflowValueContract::Projection
}
