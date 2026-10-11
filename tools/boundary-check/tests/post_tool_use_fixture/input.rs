//! Isolated input files give the bounded child an already prepared stdin.
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use worth_harness::tooling::process_deadline::{
    output_with_deadline_and_stdin, DEFAULT_CHILD_PROCESS_DEADLINE,
};

static INPUT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) fn output_with_payload(command: &mut Command, payload: Option<&str>) -> Output {
    let input = payload.map(Input::new);
    let stdin = input.as_ref().map_or_else(Stdio::null, |input| {
        Stdio::from(File::open(&input.0).unwrap())
    });
    output_with_deadline_and_stdin(command, DEFAULT_CHILD_PROCESS_DEADLINE, stdin)
        .expect("run configured hook within its deadline")
}

struct Input(PathBuf);

impl Input {
    fn new(payload: &str) -> Self {
        let directory = Path::new(env!("CARGO_TARGET_TMPDIR"));
        fs::create_dir_all(directory).unwrap();
        let path = directory.join(format!(
            "hook-input-{}-{}",
            std::process::id(),
            INPUT_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(&path, payload).unwrap();
        Self(path)
    }
}

impl Drop for Input {
    fn drop(&mut self) {
        fs::remove_file(&self.0).unwrap();
    }
}
