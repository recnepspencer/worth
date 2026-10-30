pub use worth_query::facade::domain;

#[derive(Clone, Copy)]
pub enum ArtifactWorkflowKind {
    Move,
    Lease,
    Integrated,
}

pub fn workflow_definition(
    contract: &domain::WorthQueryPortableArtifactContract,
    kind: ArtifactWorkflowKind,
) -> domain::WorthQueryPortableWorkflowDefinition {
    let artifact =
        || domain::WorthQueryWorkflowValueContract::installed_artifact(contract.reference());
    let stages = match kind {
        ArtifactWorkflowKind::Move => move_stages(artifact()),
        ArtifactWorkflowKind::Integrated => {
            let mut stages = move_stages(artifact());
            let consumer = stages
                .pop()
                .expect("the move workflow ends with its consumer");
            let mut semantics = consumer.semantics().clone();
            semantics.evidence =
                domain::WorthQueryDomainEvidenceContract::installed_artifact(contract.reference());
            stages.push(consumer.with_semantics(semantics));
            stages
        }
        ArtifactWorkflowKind::Lease => vec![
            stage(
                "produce",
                [],
                false,
                domain::WorthQueryWorkflowValueContract::Text,
                artifact(),
            ),
            stage(
                "observe-a",
                ["produce"],
                true,
                artifact(),
                domain::WorthQueryWorkflowValueContract::Text,
            ),
            stage(
                "observe-b",
                ["produce"],
                true,
                artifact(),
                domain::WorthQueryWorkflowValueContract::Text,
            ),
        ],
    };
    domain::WorthQueryPortableWorkflowDefinition::new("produce", stages)
}

pub fn move_stages(
    artifact: domain::WorthQueryWorkflowValueContract,
) -> Vec<domain::WorthQueryPortableWorkflowStage> {
    vec![
        stage(
            "produce",
            [],
            false,
            domain::WorthQueryWorkflowValueContract::Text,
            artifact.clone(),
        ),
        stage(
            "consume",
            ["produce"],
            true,
            artifact,
            domain::WorthQueryWorkflowValueContract::Text,
        ),
    ]
}

pub fn stage(
    identity: &str,
    predecessors: impl IntoIterator<Item = &'static str>,
    terminal: bool,
    input: domain::WorthQueryWorkflowValueContract,
    output: domain::WorthQueryWorkflowValueContract,
) -> domain::WorthQueryPortableWorkflowStage {
    domain::WorthQueryPortableWorkflowStage::new(
        identity,
        predecessors,
        terminal,
        false,
        std::iter::empty::<domain::WorthQueryOperationCapabilityRequirement>(),
    )
    .with_semantics(domain::WorthQueryWorkflowStageSemantics {
        input,
        output,
        cost_roles: vec![
            domain::WorthQueryWorkflowCostRole::Admission,
            domain::WorthQueryWorkflowCostRole::Execution,
            domain::WorthQueryWorkflowCostRole::ResultValidation,
        ],
        resources: super::super::execution_resource_contract(),
        terminal_result_states: terminal
            .then_some(domain::WorthQueryOperationResultState::Ready)
            .into_iter()
            .collect(),
        failure_classes: vec![domain::WorthQueryOperationFailureClass::Dependency],
        ..Default::default()
    })
}
