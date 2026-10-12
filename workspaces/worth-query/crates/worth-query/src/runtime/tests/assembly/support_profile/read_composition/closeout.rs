use super::super::super::super::support::*;

#[test]
fn runtime_public_read_composition_phase_one_closeout_answers_kernel_gate() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("task.read-composition-closeout")
        .expect("task runtime should open a named workspace");
    let support = workspace.public_read_composition_support_report();
    let closeout = workspace.public_read_composition_phase_one_closeout();

    assert_eq!(
        closeout.backend_posture(),
        WorthQueryRuntimeBackendPosture::Primary
    );
    assert_eq!(closeout.read_support_digest(), support.support_digest());
    assert_eq!(
        closeout.support_matrix_digest(),
        workspace.public_support_matrix().matrix_digest().as_str()
    );
    assert!(closeout
        .safe_to_build_now()
        .iter()
        .any(|line| line.contains("compose_read, define_read_family")));
    assert!(closeout
        .safe_to_build_now()
        .iter()
        .any(|line| line.contains("scope classes are kernel-owned")));
    assert!(closeout
        .safe_to_build_now()
        .iter()
        .any(|line| line.contains("operator-owned graph lanes now cover direct_edge")));
    assert!(closeout
        .safe_to_build_now()
        .iter()
        .any(|line| line.contains("query_runtime_current")));
    assert!(closeout
        .safe_to_build_now()
        .iter()
        .any(|line| line.contains("query_runtime_historical")));
    assert!(closeout
        .safe_to_build_now()
        .iter()
        .any(|line| line.contains("descriptor-backed synthetic runtime relationship proof")));
    assert!(closeout
        .safe_to_build_now()
        .iter()
        .any(|line| line.contains("domain_read_family_lowering")));
    assert!(closeout
        .safe_to_build_now()
        .iter()
        .any(|line| line.contains("does not accept a caller-authored invariant callback")));
    assert!(closeout
        .must_not_assume_yet()
        .iter()
        .any(|line| line.contains("by itself certifies Worth topology migration")));
    assert!(closeout
        .must_not_assume_yet()
        .iter()
        .any(|line| line.contains("future non-topology Worth domains")));
    assert!(closeout
        .migration_guidance()
        .iter()
        .any(|line| line.contains("one bounded read family onto compose_read")));
    assert!(closeout
        .migration_guidance()
        .iter()
        .any(|line| line.contains("lowering, decoder, and certification hook boundaries")));
    assert!(!closeout
        .migration_guidance()
        .iter()
        .any(|line| line.contains("invariant-pack")));
    assert!(closeout
        .migration_guidance()
        .iter()
        .any(|line| line.contains("prefer an operator-owned read lane")));
    assert!(closeout
        .migration_guidance()
        .iter()
        .any(|line| line.contains("historical basis-aware read-family path")));
    assert!(closeout.required_verification_commands().iter().any(
        |line| line == "cargo test -p worth-query --test phase_boundaries_compile_fail --quiet"
    ));
    assert!(!closeout.closeout_digest().is_empty());
}
