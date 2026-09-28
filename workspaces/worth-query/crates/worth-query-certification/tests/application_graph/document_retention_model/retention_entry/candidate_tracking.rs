fn counts() -> &'static std::sync::Mutex<std::collections::BTreeMap<u64, usize>> {
    static COUNTS: std::sync::OnceLock<std::sync::Mutex<std::collections::BTreeMap<u64, usize>>> =
        std::sync::OnceLock::new();
    COUNTS.get_or_init(Default::default)
}

pub(super) fn record_candidate(retention_days: u64) {
    *counts()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .entry(retention_days)
        .or_default() += 1;
}

pub fn reset_candidate_count(retention_days: u64) {
    counts()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .remove(&retention_days);
}

pub fn candidate_count(retention_days: u64) -> usize {
    counts()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&retention_days)
        .copied()
        .unwrap_or(0)
}
