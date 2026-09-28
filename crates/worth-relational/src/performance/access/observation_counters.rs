use super::PerformanceAccess;

impl PerformanceAccess<'_> {
    pub(crate) fn count_observation_commit_selection(&self, ancestry_visits: usize) {
        self.runtime.services.instrumentation.count(|counters| {
            counters.observation_commit_selections += 1;
            counters.observation_commit_ancestry_visits += ancestry_visits;
        });
    }
}
