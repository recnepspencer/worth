//! Honest bounds for work whose owning effect has already happened.
use super::super::refreshed_rejoin::OccurrenceSearchCharge;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
use core::convert::Infallible;

/// Each walk contacts only the selected producer occurrence's rows. An
/// owning row may release several prerequisite occurrences; each is a walk.
/// Discovery also copies its rows' recorded postings; those are separate
/// from the occurrence-row contact bound.
pub(super) struct PostCommitWalk {
    _completed: (),
}
impl PostCommitWalk {
    /// The owner has installed this outcome before cleanup can begin.
    pub(super) fn after_effect() -> Self {
        Self { _completed: () }
    }
    pub(super) fn after_publication(
        _: &crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity,
    ) -> Self {
        Self { _completed: () }
    }
    pub(super) fn after_settlement(_: &super::super::WorthQueryAcceptedOutputAuthority) -> Self {
        Self { _completed: () }
    }
    pub(super) fn after_registered(_: &super::super::SupersededSettlements) -> Self {
        Self { _completed: () }
    }
}
/// Product retirement scans N records and C custody entries, then releases
/// E edges with O(sum R_upstream) occurrence contacts. Source-wide retirement
/// scans S rows of a source coordinate (potentially several producers), then
/// O(sum R_upstream) contacts. These synchronous closure sweeps are explicit
/// debt: admission-free readers must see the closed state immediately.
/// A binding reader resolves only noted custody: C_noted entries, two source-row
/// passes, and a stale-owner occurrence walk for each inspected source row.
pub(super) struct PostCommitSweep {
    _completed: (),
}
impl PostCommitSweep {
    pub(super) fn after_source_commit(_: &WorthQueryApplicationCommitReceipt) -> Self {
        Self { _completed: () }
    }

    pub(super) fn after_effect() -> Self {
        Self { _completed: () }
    }
}
impl OccurrenceSearchCharge for PostCommitWalk {
    type Refusal = Infallible;
    fn occurrence_path(&mut self, _: usize) -> Result<(), Infallible> {
        Ok(())
    }
    fn inspect_row(&mut self) -> Result<(), Infallible> {
        Ok(())
    }
}
#[cfg(test)]
thread_local! { static SWEEP_CONTACTS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) }; }
#[cfg(test)]
pub(super) fn sweep_contacts() -> u64 {
    SWEEP_CONTACTS.with(std::cell::Cell::get)
}
impl OccurrenceSearchCharge for PostCommitSweep {
    type Refusal = Infallible;
    fn occurrence_path(&mut self, _: usize) -> Result<(), Infallible> {
        Ok(())
    }
    fn inspect_row(&mut self) -> Result<(), Infallible> {
        #[cfg(test)]
        SWEEP_CONTACTS.with(|count| count.set(count.get() + 1));
        Ok(())
    }
}
