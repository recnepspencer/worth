//! Compare adaptive grants under different real process ceilings, in isolation.
use std::num::NonZeroUsize;
use worth_execution::{ExecutionAuthority, ExecutionAuthorityConfig};

pub(crate) fn isolated_authority(test_name: &str) -> Option<ExecutionAuthority> {
    const PROCESS_CAP: &str = "WORTH_SIGNAL_RESULT_GRANT_PROCESS_CAP";
    if let Ok(cap) = std::env::var(PROCESS_CAP) {
        return Some(
            ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
                max_workers: NonZeroUsize::new(4).unwrap(),
                charged_memory_bytes: Some(cap.parse().unwrap()),
            })
            .expect("one isolated result-grant authority"),
        );
    }
    let grants: Vec<_> = [32_u64 << 20, 512 << 20]
        .into_iter()
        .map(|cap| {
            let output = crate::process_deadline::output(
                std::process::Command::new(std::env::current_exe().unwrap())
                    .args(["--exact", test_name, "--nocapture"])
                    .env(PROCESS_CAP, cap.to_string()),
            )
            .expect("result-grant child should start");
            assert!(
                output.status.success(),
                "process ceiling {cap}:\n{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .find_map(|line| line.strip_prefix("RESULT_GRANT "))
                .expect("child must exercise and report its grant")
                .parse::<u64>()
                .unwrap()
        })
        .collect();
    assert!(
        grants[0] < grants[1],
        "a smaller process ceiling must reduce the grant: {grants:?}"
    );
    None
}
