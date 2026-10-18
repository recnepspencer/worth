use super::MILESTONE_NINE_ONE_REQUIRED_COMPILE_FAIL_TARGETS;
use crate::harness::certification::digest_parts;

pub(super) fn compile_fail_boundary_digest() -> String {
    let mut parts = MILESTONE_NINE_ONE_REQUIRED_COMPILE_FAIL_TARGETS
        .iter()
        .flat_map(|target| {
            [
                format!("target:{target}"),
                format!(
                    "stderr:{}",
                    target.trim_end_matches(".rs").to_string() + ".stderr"
                ),
            ]
        })
        .collect::<Vec<_>>();
    parts.sort();
    digest_parts(&parts)
}
