#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub enum Stage {
    Decision,
    Candidate,
    AssessmentDecision,
    AssessmentCandidate,
}

pub fn counts() -> &'static std::sync::Mutex<std::collections::BTreeMap<(u64, Stage), usize>> {
    static COUNTS: std::sync::OnceLock<
        std::sync::Mutex<std::collections::BTreeMap<(u64, Stage), usize>>,
    > = std::sync::OnceLock::new();
    COUNTS.get_or_init(Default::default)
}

pub fn record_candidate(retention_days: u64) {
    record(retention_days, Stage::Candidate);
}

pub fn record_decision(retention_days: u64) {
    record(retention_days, Stage::Decision);
}

pub fn record(retention_days: u64, stage: Stage) {
    *counts()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .entry((retention_days, stage))
        .or_default() += 1;
}

pub fn reset_candidate_count(retention_days: u64) {
    reset(retention_days, Stage::Candidate);
}

pub fn reset(retention_days: u64, stage: Stage) {
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

pub fn count(retention_days: u64, stage: Stage) -> usize {
    counts()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&(retention_days, stage))
        .copied()
        .unwrap_or(0)
}

pub fn record_assessment_decision(retention_days: u64) {
    record(retention_days, Stage::AssessmentDecision);
}
pub fn record_assessment_candidate(retention_days: u64) {
    record(retention_days, Stage::AssessmentCandidate);
}
pub fn assessment_counts(retention_days: u64) -> (usize, usize) {
    (
        count(retention_days, Stage::AssessmentDecision),
        count(retention_days, Stage::AssessmentCandidate),
    )
}
