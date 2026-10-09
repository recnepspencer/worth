#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
enum Stage {
    Decision,
    Candidate,
}

fn counts() -> &'static std::sync::Mutex<std::collections::BTreeMap<(u64, Stage), usize>> {
    static COUNTS: std::sync::OnceLock<
        std::sync::Mutex<std::collections::BTreeMap<(u64, Stage), usize>>,
    > = std::sync::OnceLock::new();
    COUNTS.get_or_init(Default::default)
}

pub(super) fn record_candidate(retention_days: u64) {
    record(retention_days, Stage::Candidate);
}

pub(super) fn record_decision(retention_days: u64) {
    record(retention_days, Stage::Decision);
}

fn record(retention_days: u64, stage: Stage) {
    *counts()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .entry((retention_days, stage))
        .or_default() += 1;
}

pub fn reset_candidate_count(retention_days: u64) {
    reset(retention_days, Stage::Candidate);
}

pub fn reset_decision_count(retention_days: u64) {
    reset(retention_days, Stage::Decision);
}

fn reset(retention_days: u64, stage: Stage) {
    counts()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .remove(&(retention_days, stage));
}

pub fn candidate_count(retention_days: u64) -> usize {
    count(retention_days, Stage::Candidate)
}

pub fn decision_count(retention_days: u64) -> usize {
    count(retention_days, Stage::Decision)
}

fn count(retention_days: u64, stage: Stage) -> usize {
    counts()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&(retention_days, stage))
        .copied()
        .unwrap_or(0)
}
