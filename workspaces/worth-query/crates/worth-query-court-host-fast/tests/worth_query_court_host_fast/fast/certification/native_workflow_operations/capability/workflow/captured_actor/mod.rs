pub(crate) use worth_query_worlds::facade::certification::native_workflow_operations::capability::workflow::captured_actor::*;

#[test]
fn captured_target_actor_rule_uses_original_proposal_not_target_input() {
    let installed_host = host::install_program::<Program>(true);
    let installed = WorthQueryApplicationWorkflowSpecInstallation::<CapabilitySchema, Spec, Program>::begin(
        installed_host.application.installed_schema(),
        installed_host.application.installed_program(),
        workflow_resources(),
    )
    .operation::<ControlBinding>().unwrap()
    .operation::<ActorBinding>().unwrap()
    .authoring_capability::<RetentionCapability, SetRetention, SetRetentionInput>().unwrap()
    .instance_start_capability::<WorkflowInstanceStartCapability, SetRetention, SetRetentionInput>().unwrap()
    .advance_capability::<WorkflowAdvanceCapability, SetRetention, SetRetentionInput>().unwrap()
    .assessment::<super::super::readback::ReadBinding>().unwrap()
    .approval::<RetentionCapability, SetRetention, SetRetentionInput>().unwrap()
    .finish().unwrap();
    let (authentication, _clock) = install_certification_authentication_with_clock(
        installed_host.application.installed_schema(),
    );
    let workflow = installed_host
        .application
        .retain_workflow_spec(installed, authentication.signing_owner())
        .unwrap();
    let host = host::Host {
        application: workflow.program_runtime().clone(),
        contacts: installed_host.contacts,
        projection: installed_host.projection,
    };
    let (instance, inputs, required) = propose(&workflow, &host);
    assert_actor_execution(&workflow, &host, &instance, inputs, &required);
}
