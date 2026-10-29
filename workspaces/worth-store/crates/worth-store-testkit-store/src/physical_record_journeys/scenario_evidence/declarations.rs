pub use {crate::scenario_process_evidence::emit_process, crate::scenario_process_evidence::ScenarioProcessEvidence};

pub fn assert_distinct_processes(processes: &[ScenarioProcessEvidence]) {
    let process_ids = processes
        .iter()
        .map(crate::scenario_process_evidence::ScenarioProcessEvidence::process_id)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        process_ids.len(),
        processes.len(),
        "fresh-process roles must have distinct process identities"
    );
}