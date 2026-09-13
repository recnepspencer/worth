use std::sync::Arc;

use super::*;

impl CompositeAttemptProgress {
    /// Exact read-only evidence for the owner record. Performed authority is
    /// never duplicated: its identity and admitted basis name owner repair.
    pub(crate) fn retained_image(&self) -> Self {
        Self::new(
            self.relational.retained_image(),
            self.signal.retained_image(),
        )
    }
}

impl RelationalAttemptProgress {
    pub(crate) fn retained_image(&self) -> Self {
        let evidence = match &self.evidence {
            Some(RelationalProgressEvidence::Performed(performed)) => {
                return Self::settlement_required(
                    performed.commit_identity(),
                    performed.next_basis().clone(),
                );
            }
            Some(RelationalProgressEvidence::SettlementRequired {
                commit_identity,
                successor_basis,
            }) => Some(RelationalProgressEvidence::SettlementRequired {
                commit_identity: commit_identity.clone(),
                successor_basis: successor_basis.clone(),
            }),
            Some(RelationalProgressEvidence::SettlementPending {
                commit_identity,
                successor_basis,
                settlement,
            }) => Some(RelationalProgressEvidence::SettlementPending {
                commit_identity: commit_identity.clone(),
                successor_basis: successor_basis.clone(),
                settlement: settlement.clone(),
            }),
            Some(RelationalProgressEvidence::SettledReceipt {
                commit_identity,
                successor_basis,
                receipt,
            }) => Some(RelationalProgressEvidence::SettledReceipt {
                commit_identity: commit_identity.clone(),
                successor_basis: successor_basis.clone(),
                receipt: receipt.clone(),
            }),
            Some(RelationalProgressEvidence::Settled {
                commit_identity,
                successor_basis,
                result,
            }) => Some(RelationalProgressEvidence::Settled {
                commit_identity: commit_identity.clone(),
                successor_basis: successor_basis.clone(),
                result: Arc::clone(result),
            }),
            None => None,
        };
        Self {
            posture: self.posture,
            evidence,
            fork: self.fork.clone(),
            fork_successor_basis: self.fork_successor_basis.clone(),
        }
    }
}

impl SignalAttemptProgress {
    pub(crate) fn retained_image(&self) -> Self {
        let evidence = match &self.evidence {
            #[cfg(test)]
            Some(SignalProgressEvidence::Prepared) => Some(SignalProgressEvidence::Prepared),
            Some(SignalProgressEvidence::Advanced(outcome)) => {
                Some(SignalProgressEvidence::Advanced(Arc::clone(outcome)))
            }
            Some(SignalProgressEvidence::Forked(outcome)) => {
                Some(SignalProgressEvidence::Forked(outcome.clone()))
            }
            None => None,
        };
        Self {
            posture: self.posture,
            evidence,
        }
    }
}
