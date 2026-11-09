//! Fresh processes share only the real Home path and public status observations.
use super::*;
const MODE: &str = "WORTH_QUERY_SUPPORT_STATUS_PROOF_MODE";
const PATH: &str = "WORTH_QUERY_SUPPORT_STATUS_PROOF_PATH";
#[test]
fn retired_support_is_readmitted_in_a_fresh_process_without_initializer_or_reactivation() {
    match std::env::var(MODE).as_deref() {
        Ok("writer") => worker(true),
        Ok("reader") => worker(false),
        Ok(mode) => panic!("unknown support process {mode}"),
        Err(_) => {
            let path = fixture::location();
            let writer = child(&path, "writer");
            let reader = child(&path, "reader");
            let revision = |output: &str| {
                let values: Vec<_> = output
                    .lines()
                    .filter_map(|line| {
                        line.split_once("SUPPORT_RETIRED_REVISION=")
                            .map(|(_, value)| value.trim())
                    })
                    .collect();
                assert_eq!(
                    values.len(),
                    1,
                    "one public observation per fresh process: {output}"
                );
                values[0].to_owned()
            };
            assert_eq!(revision(&writer), revision(&reader));
        }
    }
}
fn child(path: &std::path::Path, mode: &str) -> String {
    let test = format!(
        "{}::retired_support_is_readmitted_in_a_fresh_process_without_initializer_or_reactivation",
        module_path!().split_once("::").unwrap().1
    );
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", &test, "--nocapture"])
        .env(MODE, mode)
        .env(PATH, path)
        .env_remove("RUST_MIN_STACK")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{mode} exited {}: {}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let output = String::from_utf8(output.stdout).unwrap();
    assert!(output.contains("1 passed; 0 failed"));
    output
}
fn worker(writer: bool) {
    let initialized = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let location = std::env::var_os(PATH).unwrap().into();
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request(location, initialized.clone()).open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("support worker open: {refused:?}"));
    let declaration = HomeIdentitySchema::declaration().unwrap();
    let extra = ApplicationProgramAuthoring::<HomeIdentitySchema, UnusedProgram>::begin()
        .with_schema(&declaration)
        .validated_program()
        .unwrap();
    let revision = *extra.revision();
    if writer {
        assert!(home
            .runtime()
            .supported_program::<UnusedProgram>()
            .is_some());
        home.runtime()
            .retire_program_support(&execution, &revision)
            .expect("genuine unused support retires");
    } else {
        assert!(
            matches!(home.runtime().retire_program_support(&execution, &revision), Err(WorthQueryProgramSupportRetirementFailure::Policy(WorthQueryProgramSupportRetirementDenial::AlreadyRetired { revision: refused })) if refused == revision)
        );
    }
    assert!(home
        .runtime()
        .supported_program::<UnusedProgram>()
        .is_none());
    assert!(home.runtime().presented_program().is_some());
    assert_eq!(
        initialized.load(Ordering::SeqCst),
        if writer { 1 } else { 0 }
    );
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("support worker close: {refused:?}"))
        .reclaim();
    println!("SUPPORT_RETIRED_REVISION={revision}");
}
