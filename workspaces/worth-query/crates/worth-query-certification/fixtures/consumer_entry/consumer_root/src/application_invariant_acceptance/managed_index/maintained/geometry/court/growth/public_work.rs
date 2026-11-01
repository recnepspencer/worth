//! Both actual providers expose their genuine work, independent of invoice spans.
use super::*;
#[derive(Clone, Copy, Debug)]
pub(super) struct Snapshot {
    builds: usize,
    updates: usize,
    bounds: usize,
    summaries: usize,
}
impl Snapshot {
    pub(super) fn capture(calls: &provider::Calls) -> Self {
        Self {
            builds: calls.builds.load(Ordering::SeqCst),
            updates: calls.updates.load(Ordering::SeqCst),
            bounds: calls.source_bounds.load(Ordering::SeqCst),
            summaries: calls.summary_sources.load(Ordering::SeqCst),
        }
    }
    pub(super) fn delta(self, calls: &provider::Calls) -> Self {
        let after = Self::capture(calls);
        Self {
            builds: after.builds - self.builds,
            updates: after.updates - self.updates,
            bounds: after.bounds - self.bounds,
            summaries: after.summaries - self.summaries,
        }
    }
    pub(super) fn require_local(self, calls: &provider::Calls) {
        let delta = self.delta(calls);
        assert_eq!(
            delta.builds, 0,
            "the public Source cannot hide a warm rebuild"
        );
        assert_eq!(delta.updates, 4, "two genuine scopes in each edit");
        assert_eq!(
            delta.bounds, 8,
            "only two changed public Source occurrences per scope/edit"
        );
        assert_eq!(
            delta.summaries, 8,
            "public summary qualification stays on changed Source rows"
        );
    }
}
