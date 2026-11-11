//! Query-local publication allocation; this DATA grants no publication authority.
use super::*;
use std::num::NonZeroU64;

#[derive(Debug)]
pub(super) struct QueryDispatchPublicationGenerations {
    home: HomeIdentity,
    allocated: u64,
}
impl QueryDispatchPublicationGenerations {
    /// Construct once after cold fold. Keep this allocator through every live
    /// pending/no-effect interval; do not reseed it from an unchanged ledger.
    pub(super) fn from_journal(ledger: &QueryDispatchLedger) -> Self {
        Self {
            home: ledger.home(),
            allocated: ledger.publication_generation(),
        }
    }
    pub(super) fn allocate(
        &mut self,
        ledger: &QueryDispatchLedger,
    ) -> Result<NonZeroU64, QueryDispatchJournalDenial> {
        if self.home != ledger.home() {
            return Err(QueryDispatchJournalDenial::ForeignHome);
        }
        let next = self
            .allocated
            .checked_add(1)
            .ok_or(QueryDispatchJournalDenial::IdentityExhausted)?;
        if next <= ledger.publication_generation() {
            return Err(QueryDispatchJournalDenial::PublicationGenerationMismatch);
        }
        self.allocated = next;
        Ok(NonZeroU64::new(next).expect("checked positive successor"))
    }
}
