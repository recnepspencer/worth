use super::*;

pub(super) fn materialize_decision<Schema, Operation, Input, Scope>(
    layout: &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
    program_revision: worth_query_declaration::facade::application_program::ApplicationProgramRevision,
    admitted: crate::domain_computation::primary_graph::workflow::instance::AdmittedWorkflowTransition<Schema, Operation, Input, Scope>,
    meaning: crate::domain_computation::primary_graph::workflow::WorkflowApprovalMeaning,
    evidence_currentness: std::sync::Arc<
        [crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact],
    >,
    approval_projection: publication::PreparedWorkflowApprovalProjection,
) -> Result<
    PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
    WorthQueryApplicationAttemptDenial,
>
where
    Schema: ApplicationSchema,
{
    let transition_identity = admitted.identity().to_owned();
    let transition_identity_bytes = *admitted.identity_bytes();
    let instance = admitted.instance();
    let node_path = admitted.node_path().to_owned();
    let mut demand = PlatformEffectDemand::default();
    crate::domain_computation::primary_graph::workflow::visit_workflow_approval_facts(
        layout,
        &admitted,
        &meaning,
        |effect| demand.observe(&effect),
    )?;
    let reservation = admit_platform_effects(admitted.read_set(), demand)?;
    let mut effects = Vec::new();
    crate::domain_computation::primary_graph::workflow::visit_workflow_approval_facts(
        layout,
        &admitted,
        &meaning,
        |effect| {
            effects.push(effect);
            Ok::<(), WorthQueryApplicationAttemptDenial>(())
        },
    )?;
    let validator_work_admission = reservation.materialize(&effects)?;
    let progress_update = admitted.prepare_progress_update(meaning.decision.outcome(), None)?;
    Ok(PreparedWorkflowAdvance::Transition {
        program: WorthQueryApplicationEffectProgram {
            read_set: admitted.into_read_set(),
            effects,
            emission_retained_bytes: 0,
            emission_retained_bytes_ceiling: 0,
            conditional_definition: None,
            effect_posture: crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture::Platform,
            validator_work_admission,
            output_correspondence: Default::default(),
            retain_output_demand_observation: false,
            retain_client_observation: false,
            producer_required_invariants: &[],
            output_currentness_facts: Some(evidence_currentness),
        },
        program_revision,
        transition_identity,
        transition_identity_bytes,
        transition_identity_locator: layout.transition.identity.clone(),
        assessment_identity_locator: layout.assessment_evidence.identity.clone(),
        instance,
        node_path,
        assessment: None,
        supporting_identity: None,
        operation_receipt_identity: None,
        progress_update: Some(progress_update),
        terminal: false,
        approval: Some(approval_projection),
        approval_identity: Some(meaning.identity),
        approval_authentication: None,
        replays: Default::default(),
    })
}
