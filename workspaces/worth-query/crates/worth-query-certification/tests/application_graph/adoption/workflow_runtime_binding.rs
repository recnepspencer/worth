//! Workflow configuration refusal preserves runtime, vocabulary and signing custody.
use crate::document_retention_model::{
    host::publish_on_first_program,
    operator_identity::{authenticate_operator, request_scope},
    programs::{DocumentRetentionFeature, RetentionCompletion, RetentionProgramP0},
    retention_entry::{SetRetentionBinding, SetRetentionIntent, DOCUMENT_IDENTITY},
    schema::{DocumentRetentionSchema, SetRetentionInput},
    workflow::{
        install_certification_authentication_with_clock, install_workflow_spec, workflow_resources,
    },
};
use worth_query_decl::facade::application_program::{
    ApplicationActionReference, ApplicationRootComposition,
};
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationRequestExt,
    application_installation::WorthQueryWorkflowRuntimeBindingDenial,
};

type Action = ApplicationActionReference<
    DocumentRetentionSchema,
    ApplicationRootComposition,
    DocumentRetentionFeature,
    SetRetentionBinding,
    RetentionCompletion,
>;

#[test]
fn workflow_binding_keepalive_refusal_returns_inputs_for_authentic_retry() {
    let mut runtime = publish_on_first_program();
    let workflow = install_workflow_spec(
        runtime.installed_schema(),
        runtime.installed_program(),
        workflow_resources(),
    );
    let expected_revision = *workflow.program_revision();
    let expected_schema = workflow.schema_binding().clone();
    let (authentication, _clock) =
        install_certification_authentication_with_clock(runtime.installed_schema());
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let operation = {
        let request = runtime.request(&principal, &scope);
        request
            .mutate(SetRetentionIntent {
                input: SetRetentionInput {
                    identity: DOCUMENT_IDENTITY.to_owned(),
                    retention_days: 8,
                },
            })
            .without_source()
            .idempotency_owned(0x9176_6601)
            .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
            .retain_operation::<RetentionProgramP0, Action>(&runtime)
            .unwrap_or_else(|(_, denial)| panic!("real SourceOnly construction: {denial:?}"))
    };
    // No caller clone remains: the genuine pending operation holds the extra allocation.
    let refusal = match runtime.retain_workflow_spec(workflow, authentication.signing_owner()) {
        Err(refusal) => refusal,
        Ok(_) => panic!("live operation must prevent unique-runtime configuration"),
    };
    assert_eq!(
        refusal.denial(),
        &WorthQueryWorkflowRuntimeBindingDenial::RuntimeKeepaliveRetained
    );
    let (returned_runtime, workflow, signing_owner, denial) = refusal.into_parts();
    assert_eq!(
        denial,
        WorthQueryWorkflowRuntimeBindingDenial::RuntimeKeepaliveRetained
    );
    assert_eq!(workflow.program_revision(), &expected_revision);
    assert_eq!(workflow.schema_binding(), &expected_schema);
    runtime = returned_runtime;
    operation
        .close(&runtime.request(&principal, &scope))
        .unwrap_or_else(|(_, denial)| panic!("initial owner close: {denial:?}"));
    let mut bound = runtime
        .retain_workflow_spec(workflow, signing_owner)
        .unwrap_or_else(|refusal| {
            panic!(
                "returned inputs bind after real owner close: {:?}",
                refusal.denial()
            )
        });
    assert_eq!(bound.workflow_spec().program_revision(), &expected_revision);
    assert_eq!(bound.installed_schema().binding_identity(), expected_schema);
    let operation = bound
        .request(&principal, &scope)
        .mutate(SetRetentionIntent {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: 9,
            },
        })
        .without_source()
        .idempotency_owned(0x9176_6602)
        .unwrap_or_else(|(_, _, denial)| panic!("configured input capture: {denial:?}"))
        .retain_operation::<RetentionProgramP0, Action>(bound.program_runtime())
        .unwrap_or_else(|(_, denial)| panic!("configured original owner: {denial:?}"));
    let stop = bound.close_conditional_runtime().unwrap_err();
    assert_eq!(
        stop.kind(),
        worth_query_host::facade::primary_graph::WorthQueryConditionalRuntimeInstallationDenialKind::RuntimeKeepaliveRetained
    );
    assert_eq!(bound.workflow_spec().program_revision(), &expected_revision);
    assert_eq!(bound.workflow_spec().schema_binding(), &expected_schema);
    assert_eq!(
        bound.authentication_owner().binding_identity(),
        &expected_schema
    );
    operation
        .close(&bound.request(&principal, &scope))
        .unwrap_or_else(|(_, denial)| panic!("configured original owner close: {denial:?}"));
    bound
        .close_conditional_runtime()
        .expect("configured runtime closes after its owner");
    assert_eq!(bound.workflow_spec().program_revision(), &expected_revision);
}
