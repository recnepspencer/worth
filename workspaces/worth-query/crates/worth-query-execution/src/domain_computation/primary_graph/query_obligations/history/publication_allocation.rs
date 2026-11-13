//! The live allocator and original pending frame stay with the admitted fold.
use super::journal::*;
pub(super) struct QueryObligationJournalState {
    pub(super) ledger: QueryObligationLedger,
    generations: QueryObligationPublicationGenerations,
    pub(super) pending: Option<QueryObligationFrame>,
    pub(super) interpretation_transfer:
        Option<super::original_transfer::PendingInterpretationTransfer>,
}
impl QueryObligationJournalState {
    pub(super) fn from_ledger(ledger: QueryObligationLedger) -> Self {
        let generations = QueryObligationPublicationGenerations::from_journal(&ledger);
        Self {
            ledger,
            generations,
            pending: None,
            interpretation_transfer: None,
        }
    }
    pub(super) fn prepare_pending(
        &mut self,
        record: impl Into<QueryObligationRecord>,
    ) -> Result<(), QueryObligationDenial> {
        self.prepare_record(record.into())
    }
    fn prepare_record(
        &mut self,
        record: QueryObligationRecord,
    ) -> Result<(), QueryObligationDenial> {
        if self.pending.is_some() {
            return Err(QueryObligationDenial::PublicationPending);
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
        let mut state =
            QueryObligationJournalState::from_ledger(QueryObligationLedger::empty(home));
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
            Err(QueryObligationDenial::PublicationPending)
        );
        assert_eq!(state.pending.as_ref(), Some(&original));
        assert_eq!(state.ledger.position(), 0);
        // This value-only test inspects pending DATA. It claims no physical
        // completion or no-effect; only those witnesses can discharge it.
    }
}
