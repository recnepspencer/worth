//! Passive temporal receipts from actual registry mutation, without retained authority.
/// The actual mutation owner, independent of later cutoff eligibility.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryRegistryCustodyTransitionForTest {
    SettlementPublished,
    CachedReadyClosed,
    AdmissionSupersession,
    SourceRetirement,
    CachedReclamation,
    PostingRelease,
}

/// Counts copied from a row already selected by the owning registry operation.
/// Rows without a posted settlement have no consumer custody to report.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryRegistryCustodyForTest {
    pub transition: WorthQueryRegistryCustodyTransitionForTest,
    pub settlements: usize,
    pub prerequisites: usize,
}

pub(in crate::domain_computation::primary_graph) fn registry_custody(
    transition: WorthQueryRegistryCustodyTransitionForTest,
    settlements: usize,
    prerequisites: usize,
) {
    if settlements == 0 {
        return;
    }
    super::record(|observed| {
        let receipt = WorthQueryRegistryCustodyForTest {
            transition,
            settlements,
            prerequisites,
        };
        match observed
            .registry_custody
            .iter_mut()
            .find(|slot| slot.is_none())
        {
            Some(slot) => *slot = Some(receipt),
            None => super::add(&mut observed.registry_custody_omitted, 1),
        }
    });
}
