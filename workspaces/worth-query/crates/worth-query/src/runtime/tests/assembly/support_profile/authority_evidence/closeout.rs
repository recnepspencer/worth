use super::super::super::super::support::*;

#[test]
fn runtime_public_authority_evidence_closeout_matches_certified_support_surface() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("task.authority-evidence-closeout")
        .expect("task runtime should open a named workspace");
    let support = workspace.public_authoritative_mutation_evidence_support();
    let closeout = workspace.public_authoritative_mutation_evidence_closeout();

    assert_eq!(
        closeout.backend_posture(),
        WorthQueryRuntimeBackendPosture::Primary
    );
    assert_eq!(closeout.query_support_digest(), support.support_digest());
    assert_eq!(
        closeout.support_matrix_digest(),
        workspace.public_support_matrix().matrix_digest().as_str()
    );
    assert!(closeout
        .safe_to_build_now()
        .iter()
        .any(|line| line.contains("declared-versus-resolved target evidence")));
    assert!(closeout
        .safe_to_build_now()
        .iter()
        .any(|line| line.contains("graph composition support is now machine-readable")));
    assert!(closeout
        .must_not_assume_yet()
        .iter()
        .any(|line| line.contains("unsupported existing-truth binding")));
    assert!(closeout.migration_guidance().iter().any(
        |line| line.contains("read graph-composition capability rows and extension-hook rows")
    ));
    assert!(closeout
        .required_verification_commands()
        .iter()
        .any(|line| line == "cargo test -p worth-query"));
    assert!(!closeout.closeout_digest().is_empty());
}
