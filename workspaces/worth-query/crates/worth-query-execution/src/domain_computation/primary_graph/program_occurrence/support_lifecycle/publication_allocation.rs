//! The live allocator and original pending frame stay with the admitted fold.
use super::journal::*;
pub(super) struct SupportJournalState {
    pub(super) ledger: QueryProgramSupportLedger,
    generations: QueryProgramSupportPublicationGenerations,
    pub(super) pending: Option<QueryProgramSupportFrame>,
}
impl SupportJournalState {
    pub(super) fn from_ledger(ledger: QueryProgramSupportLedger) -> Self {
        let generations = QueryProgramSupportPublicationGenerations::from_journal(&ledger);
        Self {
            ledger,
            generations,
            pending: None,
        }
    }
    pub(super) fn prepare_pending(
        &mut self,
        record: QueryProgramSupportRecord,
    ) -> Result<(), QueryProgramSupportDenial> {
        if self.pending.is_some() {
            return Err(QueryProgramSupportDenial::PublicationPending);
        }
        self.pending = Some(self.ledger.prepare(record, &mut self.generations)?);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_foundational::facade::{CanonicalDigestId, HomeIdentity};
    #[test]
    fn a_pending_publication_keeps_its_original_frame_and_does_not_allocate_a_replacement() {
        let home = HomeIdentity::from_observed_bytes([1; 16]).unwrap();
        let mut state = SupportJournalState::from_ledger(QueryProgramSupportLedger::empty(home));
        state
            .prepare_pending(QueryProgramSupportRecord::Install {
                revision: CanonicalDigestId::new([1; 32]),
            })
            .unwrap();
        let original = state.pending.clone().unwrap();
        assert_eq!(
            state.prepare_pending(QueryProgramSupportRecord::Install {
                revision: CanonicalDigestId::new([2; 32])
            }),
            Err(QueryProgramSupportDenial::PublicationPending)
        );
        assert_eq!(state.pending.as_ref(), Some(&original));
        assert_eq!(state.ledger.position(), 0);
        // This value-only test inspects pending DATA. It claims no physical
        // completion or no-effect; only those witnesses can discharge it.
    }
}
