fn counts() -> &'static std::sync::Mutex<std::collections::BTreeMap<u64, usize>> {
    static COUNTS: std::sync::OnceLock<std::sync::Mutex<std::collections::BTreeMap<u64, usize>>> =
        std::sync::OnceLock::new();
    COUNTS.get_or_init(Default::default)
}

pub(super) fn record_candidate(dimension: u64) {
    *counts()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .entry(dimension)
        .or_default() += 1;
}

pub fn reset_candidate_count(dimension: u64) {
    counts()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .remove(&dimension);
}

pub fn candidate_count(dimension: u64) -> usize {
    counts()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&dimension)
        .copied()
        .unwrap_or(0)
}
