//! Closing and reopening a host carrying a genuinely installed workflow vocabulary.
use super::super::*;
use worth_query_host::facade::application_installation::{program, WorthQueryHomeOpening};

#[test]
fn workflow_application_home_close_reopens_with_its_installed_vocabulary() {
    let application = publish_workflow_on_first_program();
    let revision = *application.workflow.installed_program().revision();
    let home = application
        .workflow
        .close()
        .expect("the idle workflow owner closes");
    let reopened = program(
        validated_first_program(),
        DocumentRetentionSchema::declaration().expect("the fixture schema is valid"),
        ((),),
        host_limits(),
    )
    .roster(WorthQueryApplicationProgramRoster::new().support(validated_second_program()))
    .open(home)
    .expect("the workflow application's home resumes");
    assert_eq!(
        reopened.opening(),
        &WorthQueryHomeOpening::Resumed {
            installed: revision
        }
    );
    let reopened = super::super::super::workflow::retain_workflow(reopened);
    assert_eq!(
        reopened.workflow.workflow_spec().program_revision(),
        &revision
    );
    reopened
        .workflow
        .close()
        .expect("the reopened workflow owner closes again");
}
