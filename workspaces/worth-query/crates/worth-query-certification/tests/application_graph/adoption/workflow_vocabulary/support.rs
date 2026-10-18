//! Refused support retains exact vocabulary for correction on its owning host.
use super::*;

#[test]
fn a_supported_vocabulary_names_one_rostered_program_once() {
    let mut application = publish_workflow_on_first_program();
    support_workflow_program::<RetentionProgramP1>(&mut application);

    let duplicate = install_workflow_spec(
        application.installed_schema(),
        application
            .supported_program::<RetentionProgramP1>()
            .expect("P1 is rostered")
            .installed_program(),
        application.workflow_spec().resources(),
    );
    let expected_revision = *duplicate.program_revision();
    let (duplicate, denial) = application
        .workflow_mut()
        .support_workflow_spec(duplicate)
        .unwrap_err();
    assert_eq!(
        denial,
        WorthQueryWorkflowRuntimeBindingDenial::AlreadySupported
    );
    assert_eq!(duplicate.program_revision(), &expected_revision);

    let initial = install_workflow_spec(
        application.installed_schema(),
        application.installed_program(),
        application.workflow_spec().resources(),
    );
    let expected_revision = *initial.program_revision();
    let (initial, denial) = application
        .workflow_mut()
        .support_workflow_spec(initial)
        .unwrap_err();
    assert_eq!(
        denial,
        WorthQueryWorkflowRuntimeBindingDenial::AlreadySupported
    );
    assert_eq!(initial.program_revision(), &expected_revision);

    let mut other = publish_workflow_on_first_program();
    let foreign_schema = install_workflow_spec(
        other.installed_schema(),
        other
            .supported_program::<RetentionProgramP1>()
            .unwrap()
            .installed_program(),
        application.workflow_spec().resources(),
    );
    let expected_binding = foreign_schema.schema_binding().clone();
    let (foreign_schema, denial) = application
        .workflow_mut()
        .support_workflow_spec(foreign_schema)
        .unwrap_err();
    assert_eq!(
        denial,
        WorthQueryWorkflowRuntimeBindingDenial::ForeignSchema
    );
    assert_eq!(foreign_schema.schema_binding(), &expected_binding);
    other
        .workflow_mut()
        .support_workflow_spec(foreign_schema)
        .unwrap_or_else(|(_, denial)| {
            panic!("same returned vocabulary on its authentic host: {denial:?}")
        });
}

#[test]
fn a_retired_program_accepts_no_workflow_vocabulary() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut application = publish_workflow_on_first_program();
    let late = install_workflow_spec(
        application.installed_schema(),
        application
            .supported_program::<RetentionProgramP1>()
            .expect("P1 is rostered")
            .installed_program(),
        application.workflow_spec().resources(),
    );
    retire_second_program(request_execution, &application);
    let expected_revision = *late.program_revision();
    let (late, denial) = application
        .workflow_mut()
        .support_workflow_spec(late)
        .unwrap_err();
    assert_eq!(
        denial,
        WorthQueryWorkflowRuntimeBindingDenial::ForeignProgram
    );
    assert_eq!(late.program_revision(), &expected_revision);
}
