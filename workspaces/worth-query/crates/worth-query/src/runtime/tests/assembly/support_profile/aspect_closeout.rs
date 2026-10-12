use super::super::super::support::*;

#[test]
fn runtime_public_aspect_api_finalization_closeout_answers_substrate_handoff_questions() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("task.aspect-api-closeout")
        .expect("task runtime should open a named workspace");
    let closeout = workspace.public_aspect_api_finalization_closeout();
    let report = workspace.public_mutation_surface_report();
    let matrix = workspace.public_support_matrix();
    let naming = WorthQueryRuntime::public_api_naming_contract();

    assert_eq!(
        closeout.backend_posture(),
        WorthQueryRuntimeBackendPosture::Primary
    );
    assert_eq!(
        closeout.support_matrix_digest(),
        matrix.matrix_digest().as_str()
    );
    assert_eq!(closeout.mutation_surface_digest(), report.report_digest());
    assert_eq!(closeout.naming_contract_digest(), naming.contract_digest());
    assert!(closeout
        .preferred_stable_surfaces()
        .iter()
        .any(|row| row == "workspace.insert(...)"));
    assert!(closeout
        .lower_level_stable_surfaces()
        .iter()
        .any(|row| row == "WorthQueryWriteCommand::InsertAspects=>workspace.insert(...)"));
    assert!(closeout
        .support_gated_surfaces()
        .iter()
        .any(|row| row == "workspace.intent(...)"));
}
