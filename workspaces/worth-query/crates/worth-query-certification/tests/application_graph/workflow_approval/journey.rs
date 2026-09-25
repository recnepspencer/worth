use super::*;

pub(super) fn approval_journey(
    completion: &str,
    key: u64,
) -> (
    BoundedDimensionWorkflowRuntime,
    PublishedWorkflowDefinitionRef,
    PublishedWorkflowInstanceRef,
    PublishedWorkflowProposalRef,
    RequiredWorkflowApproval,
    Vec<worth_relational::facade::identity::EntityId>,
) {
    let application =
        super::super::bounded_dimension_model::host::publish_workflow_on_first_program();
    let definition = match publish_definition(
        &application,
        reviewed_geometry_definition(completion),
        WorkflowDefinitionExpectedPredecessor::Absent,
        key,
    )
    .expect("approval definition publication must prepare")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => performed,
        other => panic!("expected a published approval definition, got {other:?}"),
    };
    let instance = match start_instance(&application, definition.definition().clone(), key + 1)
        .expect("approval instance start must prepare")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("expected a started approval instance, got {other:?}"),
    };
    let proposal = proposal::published_proposal(&application, instance.clone(), key + 2);
    let mut evidence = Vec::new();
    for offset in [3, 5] {
        let settlement = settle_assessment(&application, instance.clone(), key + offset);
        match accept_assessment(
            &application,
            instance.clone(),
            &settlement,
            key + offset + 1,
        ) {
            Ok(WorkflowProgressOutcome::Completed(performed)) => evidence.push(
                performed
                    .assessment_evidence()
                    .expect("accepted assessment must expose its evidence entity")
                    .evidence(),
            ),
            other => panic!("expected accepted assessment, got {other:?}"),
        }
    }
    match advance_instance(&application, instance.clone(), key + 7) {
        Ok(WorkflowProgressOutcome::Completed(_)) => {}
        other => panic!("expected completed evidence join, got {other:?}"),
    }
    let required = match advance_instance(&application, instance.clone(), key + 8)
        .expect("approval requirement must prepare")
    {
        WorkflowProgressOutcome::AwaitingApproval(required) => required,
        other => panic!("expected an approval requirement, got {other:?}"),
    };
    (
        application,
        definition.definition().clone(),
        instance,
        proposal,
        required,
        evidence,
    )
}
