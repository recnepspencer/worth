pub use {std::path::Path};

pub use {crate::child_process::CHILD_TEST, crate::child_process::LOCATOR_ENV, crate::child_process::ORACLE_ENV, crate::child_process::ROLE_ENV, crate::child_process::ROOT_ENV};

pub fn child_command(role: &str, root: &Path) -> std::process::Command {
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", CHILD_TEST, "--nocapture"])
        .env(ROLE_ENV, role)
        .env(ROOT_ENV, root);
    command
}

pub fn run_child(role: &str, root: &Path, locator: Option<&str>) -> String {
    let mut command = child_command(role, root);
    if let Some(locator) = locator {
        command.env(LOCATOR_ENV, locator);
    }
    let output = command.output().unwrap();
    let causal_marker = match role {
        "publication_reopener" => "C5_PREDICATE:independent-decision-path ",
        "allocation_writer" | "allocation_reader" => "C5_PREDICATE:transfer-allocation-slope ",
        _ => "",
    };
    assert!(
        output.status.success(),
        "{causal_marker}child {role} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

pub fn run_courtroom_writer(root: &Path, locators: &Path, oracle: &Path) -> String {
    run_courtroom_child("courtroom_writer", root, locators, Some(oracle))
}

pub fn run_courtroom_reopener(root: &Path, locators: &Path) -> String {
    run_courtroom_child("courtroom_reopener", root, locators, None)
}

pub fn run_courtroom_child(role: &str, root: &Path, locators: &Path, oracle: Option<&Path>) -> String {
    let mut command = child_command(role, root);
    command.env(LOCATOR_ENV, locators);
    if let Some(oracle) = oracle {
        command.env(ORACLE_ENV, oracle);
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "child {role} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}