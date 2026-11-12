//! Acknowledged recovery claims and their one terminal disposition.
use super::*;
use worth_foundational::facade::{
    CorrectionMaterialPinIdentity, DurableCompositeCommitIdentity,
    DurableCompositePublicationAttemptIdentity, DurableRecoveryClaimIdentity,
    RecoveryClaimDisposition,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct QueryRecoveryClaim {
    pub original: DurableCompositeCommitIdentity,
    pub attempt: DurableCompositePublicationAttemptIdentity,
    pub expires_at_unix_ms: u64,
    pub material_pin: CorrectionMaterialPinIdentity,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) enum QueryRecoveryRecord {
    Claim(QueryRecoveryClaim),
    Disposition {
        claim: DurableRecoveryClaimIdentity,
        terminal: RecoveryClaimDisposition,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct QueryRecordedRecoveryClaim {
    pub identity: DurableRecoveryClaimIdentity,
    pub claim: QueryRecoveryClaim,
    pub terminal: Option<RecoveryClaimDisposition>,
}
#[derive(Clone, Debug, Default)]
pub(super) struct QueryRecoveryLedger {
    claims: BTreeMap<DurableCompositeCommitIdentity, QueryRecordedRecoveryClaim>,
    by_identity: BTreeMap<DurableRecoveryClaimIdentity, DurableCompositeCommitIdentity>,
}
impl QueryRecoveryLedger {
    pub(super) fn validate(
        &self,
        home: HomeIdentity,
        record: &QueryRecoveryRecord,
    ) -> Result<(), QueryDispatchJournalDenial> {
        use QueryDispatchJournalDenial::MalformedFrame;
        match record {
            QueryRecoveryRecord::Claim(claim) => {
                if claim.original.owner().home() != home
                    || claim.attempt.owner() != claim.original.owner()
                    || claim.expires_at_unix_ms == 0
                    || claim.material_pin.home() != home
                    || self.claims.get(&claim.original).is_some_and(|prior| {
                        prior.terminal != Some(RecoveryClaimDisposition::Relinquished)
                    })
                    || self.claims.values().any(|prior| {
                        prior.terminal.is_none() && prior.claim.material_pin == claim.material_pin
                    })
                {
                    return Err(MalformedFrame);
                }
            }
            QueryRecoveryRecord::Disposition { claim, .. } => {
                let original = self.by_identity.get(claim).ok_or(MalformedFrame)?;
                let current = self.claims.get(original).ok_or(MalformedFrame)?;
                if claim.home() != home || current.identity != *claim || current.terminal.is_some()
                {
                    return Err(MalformedFrame);
                }
            }
        }
        Ok(())
    }
    pub(super) fn fold(&mut self, home: HomeIdentity, position: u64, record: QueryRecoveryRecord) {
        match record {
            QueryRecoveryRecord::Claim(claim) => {
                let identity = DurableRecoveryClaimIdentity::from_recorded(
                    home,
                    std::num::NonZeroU64::new(position).expect("validated positive position"),
                );
                if let Some(prior) = self.claims.insert(
                    claim.original,
                    QueryRecordedRecoveryClaim {
                        identity,
                        claim,
                        terminal: None,
                    },
                ) {
                    self.by_identity.remove(&prior.identity);
                }
                self.by_identity.insert(identity, claim.original);
            }
            QueryRecoveryRecord::Disposition { claim, terminal } => {
                let original = self
                    .by_identity
                    .get(&claim)
                    .expect("validated original claim");
                self.claims
                    .get_mut(original)
                    .expect("validated original")
                    .terminal = Some(terminal);
            }
        }
    }
    pub(super) fn claim(
        &self,
        original: &DurableCompositeCommitIdentity,
    ) -> Option<QueryRecordedRecoveryClaim> {
        self.claims.get(original).copied()
    }
    pub(super) fn claims(&self) -> impl Iterator<Item = QueryRecordedRecoveryClaim> + '_ {
        self.claims.values().copied()
    }
}
