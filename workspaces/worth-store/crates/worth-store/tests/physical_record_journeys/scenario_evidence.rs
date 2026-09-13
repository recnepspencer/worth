pub(super) use super::scenario_process_evidence::{emit_process, ScenarioProcessEvidence};

pub(super) fn assert_distinct_processes(processes: &[ScenarioProcessEvidence]) {
    let process_ids = processes
        .iter()
        .map(ScenarioProcessEvidence::process_id)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        process_ids.len(),
        processes.len(),
        "fresh-process roles must have distinct process identities"
    );
}
