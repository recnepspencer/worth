pub(crate) use worth_query_bank_worlds::facade::external_rail::exit_proof::separate_process::*;

#[test]
fn rail_process_pid_differs_from_test_process_pid() {
    let rail = spawn_rail();

    let rail_pid = rail.pid();
    let test_process_pid = std::process::id();

    assert_ne!(
        rail_pid, test_process_pid,
        "the external rail must run in its own OS process, not inside the test process"
    );
    assert_ne!(rail_pid, 0, "a spawned child always has a nonzero PID");
}
