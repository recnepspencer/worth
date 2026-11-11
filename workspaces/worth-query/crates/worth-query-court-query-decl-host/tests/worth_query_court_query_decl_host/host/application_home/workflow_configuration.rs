//! Installed workflow configuration retains the authentic Home and its refusal posture.
use super::*;
use worth_query_host::facade::application_installation::WorthQueryWorkflowRuntimeBindingDenial as Denial;
use worth_query_testkit_document_retention::facade::document_retention_model as document;
mod memory_home;
mod publication;

#[test]
fn memory_home_workflow_configuration_survives_close_refusal_and_resumes_the_original_key() {
    use worth_query_host::facade::application_entry::WorkflowDefinitionPublicationOutcome as Outcome;
    let home = memory_home::bind(memory_home::open());
    assert!(!home.is_directory());
    let branch = home.runtime().runtime().current_world();
    assert!(matches!(
        publication::publish(&home, 0x716801),
        Outcome::Published(_)
    ));
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let Err(refused) = home.close(&execution) else {
        panic!("Memory has no Directory close lifecycle");
    };
    assert!(refused.reason().directory_denial().is_none());
    assert!(refused.reason().query_denial().is_none());
    let Err(refused) = refused.try_close(&execution) else {
        panic!("retry must preserve the same Memory close refusal");
    };
    let home = refused.resume().unwrap();
    assert_eq!(home.runtime().runtime().current_world(), branch);
    assert!(
        matches!(
            publication::publish(&home, 0x716801),
            Outcome::RecoveredCommitted(_)
        ),
        "the same actual configured Home replays its original publication key"
    );
}

#[test]
fn foreign_workflow_binding_returns_the_original_home_spec_and_signing_owner() {
    let home = memory_home::open();
    let other = memory_home::open();
    let original = home.runtime().runtime().current_world();
    let schema = home.runtime().installed_schema().binding_identity();
    let other_schema = other.runtime().installed_schema().binding_identity();
    assert_ne!(schema, other_schema);
    let workflow = document::workflow::install_workflow_spec(
        other.runtime().installed_schema(),
        other.runtime().installed_program(),
        document::workflow::workflow_resources(),
    );
    let (authentication, _) = document::workflow::install_certification_authentication_with_clock(
        home.runtime().installed_schema(),
    );
    let signing = authentication.signing_owner();
    let refused = home
        .retain_workflow_spec(workflow, signing)
        .err()
        .expect("foreign installed schema refuses");
    assert_eq!(refused.denial(), &Denial::ForeignSchema);
    let (home, workflow, signing, cause) = refused.into_parts();
    assert_eq!(cause, Denial::ForeignSchema);
    assert_eq!(home.runtime().runtime().current_world(), original);
    assert_eq!(*workflow.schema_binding(), other_schema);
    assert_eq!(*signing.binding_identity(), schema);
    let (other_authentication, _) =
        document::workflow::install_certification_authentication_with_clock(
            other.runtime().installed_schema(),
        );
    let _other = other
        .retain_workflow_spec(workflow, other_authentication.signing_owner())
        .unwrap();
    let workflow = document::workflow::install_workflow_spec(
        home.runtime().installed_schema(),
        home.runtime().installed_program(),
        document::workflow::workflow_resources(),
    );
    let home = home.retain_workflow_spec(workflow, signing).unwrap();
    assert!(matches!(publication::publish(&home, 0x716802),
        worth_query_host::facade::application_entry::WorkflowDefinitionPublicationOutcome::Published(_)),
        "refused binding installed neither a signer nor partial vocabulary");
}

#[test]
fn workflow_configuration_keeps_no_runtime_backing_and_refuses_a_foreign_receiver() {
    let mut runtime = document::host::publish_on_first_program();
    let workflow = document::workflow::install_workflow_spec(
        runtime.installed_schema(),
        runtime.installed_program(),
        document::workflow::workflow_resources(),
    );
    let (authentication, _) = document::workflow::install_certification_authentication_with_clock(
        runtime.installed_schema(),
    );
    let mut configuration = runtime
        .install_workflow_configuration(workflow, authentication.signing_owner())
        .unwrap_or_else(|(_, _, cause)| panic!("actual installed inputs: {cause:?}"));
    assert!(configuration.vocabulary_for(&runtime).is_ok());
    let workflow = document::workflow::install_workflow_spec(
        runtime.installed_schema(),
        runtime
            .supported_program::<document::programs::RetentionProgramP1>()
            .unwrap()
            .installed_program(),
        document::workflow::workflow_resources(),
    );
    configuration
        .support_workflow_spec(&mut runtime, workflow)
        .unwrap_or_else(|(_, cause)| {
            panic!("configuration must leave backing uniquely owned: {cause:?}")
        });
    let other = memory_home::open();
    assert!(matches!(
        configuration.vocabulary_for(other.runtime()),
        Err(Denial::ForeignRuntime)
    ));
}
