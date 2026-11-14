//! Query-local publication allocation; this DATA grants no publication authority.
use super::*;
use std::num::NonZeroU64;

#[derive(Debug)]
pub struct QueryObligationPublicationGenerations {
    home: HomeIdentity,
    allocated: u64,
}
impl QueryObligationPublicationGenerations {
    /// Construct once after cold fold. Keep this allocator through every live
    /// pending/no-effect interval; do not reseed it from an unchanged ledger.
    pub fn from_journal(ledger: &QueryObligationLedger) -> Self {
        Self {
            home: ledger.home(),
            allocated: ledger.publication_generation(),
        }
    }
    pub(super) fn allocate(
        &mut self,
        ledger: &QueryObligationLedger,
    ) -> Result<NonZeroU64, QueryObligationDenial> {
        if self.home != ledger.home() {
            return Err(QueryObligationDenial::ForeignHome);
        }
        let next = self
            .allocated
            .checked_add(1)
            .ok_or(QueryObligationDenial::IdentityExhausted)?;
        if next <= ledger.publication_generation() {
            return Err(QueryObligationDenial::PublicationGenerationMismatch);
        }
        self.allocated = next;
        Ok(NonZeroU64::new(next).expect("checked positive successor"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn home(byte: u8) -> HomeIdentity {
        HomeIdentity::from_observed_bytes([byte; 16]).unwrap()
    }
    fn install(byte: u8) -> QueryProgramSupportRecord {
        QueryProgramSupportRecord::Install {
            revision: CanonicalDigestId::new([byte; 32]),
        }
    }
    #[test]
    fn fresh_publications_keep_live_gaps_and_cold_fold_preserves_the_committed_frontier() {
        let mut ledger = QueryObligationLedger::empty(home(1));
        let mut generations = QueryObligationPublicationGenerations::from_journal(&ledger);
        let first = ledger.prepare(install(1), &mut generations).unwrap();
        let fresh = ledger.prepare(install(1), &mut generations).unwrap();
        assert_eq!(first.position(), fresh.position());
        assert_eq!(first.record(), fresh.record());
        assert_ne!(first.bytes(), fresh.bytes());
        assert_eq!(fresh.publication_generation().get(), 2);
        ledger
            .readmit(QueryObligationFrame::readmit(fresh.bytes()).unwrap())
            .unwrap();
        let mut cold = QueryObligationPublicationGenerations::from_journal(&ledger);
        assert_eq!(
            ledger
                .prepare(install(2), &mut cold)
                .unwrap()
                .publication_generation()
                .get(),
            3
        );
        let regressed = super::super::codec::encode(
            home(1),
            2,
            ledger.digest(),
            first.publication_generation(),
            install(2).into(),
        );
        assert_eq!(
            ledger.readmit(regressed),
            Err(QueryObligationDenial::PublicationGenerationMismatch)
        );
        assert_eq!(ledger.position(), 1);
    }
    #[test]
    fn generation_allocation_refuses_foreign_home_and_exhaustion_before_preparation() {
        let ledger = QueryObligationLedger::empty(home(1));
        let mut foreign = QueryObligationPublicationGenerations::from_journal(
            &QueryObligationLedger::empty(home(2)),
        );
        assert_eq!(
            ledger.prepare(install(1), &mut foreign),
            Err(QueryObligationDenial::ForeignHome)
        );
        let mut exhausted = QueryObligationPublicationGenerations {
            home: home(1),
            allocated: u64::MAX,
        };
        assert_eq!(
            ledger.prepare(install(1), &mut exhausted),
            Err(QueryObligationDenial::IdentityExhausted)
        );
        assert_eq!(ledger.position(), 0);
    }
    #[test]
    fn a_checksummed_zero_generation_is_malformed() {
        use sha2::{Digest, Sha256};
        let ledger = QueryObligationLedger::empty(home(1));
        let mut generations = QueryObligationPublicationGenerations::from_journal(&ledger);
        let frame = ledger.prepare(install(1), &mut generations).unwrap();
        let mut bytes = frame.bytes().to_vec();
        bytes.truncate(bytes.len() - 32);
        bytes[68..76].copy_from_slice(&0u64.to_le_bytes());
        let checksum = Sha256::digest(&bytes);
        bytes.extend_from_slice(&checksum);
        assert_eq!(
            QueryObligationFrame::readmit(&bytes),
            Err(QueryObligationDenial::MalformedFrame)
        );
    }
}
