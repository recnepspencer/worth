//! A test that constructs its own physical authority runs in a fresh process.
//!
//! [`crate::ExecutionAuthority`] is unique for a process lifetime, including
//! after drop, so a fixture that needs its own authority configuration cannot
//! share the libtest process with the shared fixture authority or with another
//! such fixture. The test re-executes this test binary for exactly itself.
use std::process::Command;

const CHILD: &str = "WORTH_EXECUTION_FRESH_PROCESS_TEST";

/// Returns `true` in the parent once the named test passed in its own child
/// process, and `false` inside that child, where the caller runs its body.
pub(crate) fn ran_in_fresh_child(module: &str, test: &str) -> bool {
    let module = module
        .split_once("::")
        .map_or("", |(_crate_name, path)| path);
    let name = format!("{module}::{test}");
    if std::env::var(CHILD).is_ok_and(|selected| selected == name) {
        return false;
    }
    let output = Command::new(std::env::current_exe().expect("the running test binary"))
        .args([name.as_str(), "--exact", "--test-threads=1"])
        .env(CHILD, &name)
        .output()
        .expect("the test binary re-executes for one fresh-process case");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() && stdout.contains("test result: ok. 1 passed"),
        "`{name}` failed in its fresh process\n--- stdout ---\n{stdout}\n--- stderr ---\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    true
}
