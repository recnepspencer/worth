use worth_query_declaration::facade::application_capability::ApplicationCapabilityMarkerIdentity;
use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowSpec, ApplicationWorkflowSpecIdentity,
};

use super::{
    WorthQueryApplicationWorkflowInstallationDenialKind,
    WorthQueryApplicationWorkflowResourceCeiling, WorthQueryApplicationWorkflowSpecInstallation,
};
use crate::application_program::{
    install_application_program,
    program_support_fixture::{
        installed_support_schema, validated, AdjustInput, CompleteProgram, SupportSchema,
        UnknownOperation,
    },
};

struct SupportWorkflow;
struct WorkflowCapability;

worth_query_declaration::worth_query_portable_type!(
    WorkflowCapability => "worth.query.tests.workflow-capability.v1"
);

impl ApplicationCapabilityMarkerIdentity for WorkflowCapability {
    type Schema = SupportSchema;
    const IDENTIFIER: &'static str = "worth.query.tests.workflow-capability.v1";
}

impl ApplicationWorkflowSpec for SupportWorkflow {
    type Schema = SupportSchema;
    const IDENTITY: ApplicationWorkflowSpecIdentity =
        ApplicationWorkflowSpecIdentity::new("worth.query.tests.support-workflow.v1");
}

fn resources() -> WorthQueryApplicationWorkflowResourceCeiling {
    WorthQueryApplicationWorkflowResourceCeiling::new(8, 16, 2, 4, 4096, 8, 32, 4096)
        .expect("the test ceilings are nonzero")
}

#[test]
fn workflow_vocabulary_cannot_relabel_a_foreign_program_installation() {
    let program_schema = installed_support_schema();
    let program = install_application_program(validated::<CompleteProgram>(), &program_schema)
        .expect("the complete fixture program installs");
    let foreign_schema = installed_support_schema();
    let result = WorthQueryApplicationWorkflowSpecInstallation::<
        SupportSchema,
        SupportWorkflow,
        CompleteProgram,
    >::begin(&foreign_schema, &program, resources())
    .finish();
    let denial = match result {
        Ok(_) => panic!("a workflow vocabulary relabeled a foreign program installation"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationWorkflowInstallationDenialKind::ForeignSchemaBinding
    );
}

#[test]
fn approval_capability_requires_its_operation_in_the_selected_program() {
    let schema = installed_support_schema();
    let program = install_application_program(validated::<CompleteProgram>(), &schema)
        .expect("the complete fixture program installs");
    let denial = match WorthQueryApplicationWorkflowSpecInstallation::<
        SupportSchema,
        SupportWorkflow,
        CompleteProgram,
    >::begin(&schema, &program, resources())
    .approval::<WorkflowCapability, UnknownOperation, AdjustInput>()
    {
        Ok(_) => panic!("approval accepted an operation outside the selected program"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationWorkflowInstallationDenialKind::OperationNotInProgram
    );
}

#[test]
fn authoring_capability_requires_its_operation_in_the_selected_program() {
    let schema = installed_support_schema();
    let program = install_application_program(validated::<CompleteProgram>(), &schema)
        .expect("the complete fixture program installs");
    let denial = match WorthQueryApplicationWorkflowSpecInstallation::<
        SupportSchema,
        SupportWorkflow,
        CompleteProgram,
    >::begin(&schema, &program, resources())
    .authoring_capability::<WorkflowCapability, UnknownOperation, AdjustInput>()
    {
        Ok(_) => panic!("authoring accepted an operation outside the selected program"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationWorkflowInstallationDenialKind::OperationNotInProgram
    );
}
