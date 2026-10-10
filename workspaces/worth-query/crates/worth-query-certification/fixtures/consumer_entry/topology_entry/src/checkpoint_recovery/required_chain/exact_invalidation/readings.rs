//! Advancement readings keep lifetime counters inside their owning handle.
use super::Cost;

pub(super) use super::super::contact_readings::Reading;

impl Reading {
    /// The handle's contacts since its preceding reading and the source queries
    /// during this advance. Queries used to judge the answer are outside it.
    pub(super) fn measure<T>(&mut self, advance: impl FnOnce() -> (T, usize)) -> (T, Cost) {
        let ((answer, lifetime), source_queries) = Self::queries(advance);
        let producer_contacts = self.contacts(lifetime);
        (
            answer,
            Cost {
                producer_contacts,
                source_queries,
            },
        )
    }

    /// A request's query runs between its two readings, excluding other calls.
    pub(super) fn queries<T>(run: impl FnOnce() -> T) -> (T, u64) {
        let before = super::query_entries();
        let answer = run();
        let delta = super::query_entries()
            .checked_sub(before)
            .expect("the query run counter never decreases");
        (answer, delta)
    }

    /// Decision evidence is consumed once: only decisions since the preceding
    /// judgment are returned, never a lifetime count from an earlier demand.
    pub(super) fn decisions() -> Vec<(String, Vec<u64>)> {
        super::super::binding::take_all_decisions()
    }
}
