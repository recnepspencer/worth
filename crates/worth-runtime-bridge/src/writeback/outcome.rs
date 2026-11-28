use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::adapter::TruthWritebackReceipt;

use super::BridgeWritebackIdempotenceBasis;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeWritebackAuthorityOutcome {
    outcome_class: BridgeWritebackOutcomeClass,
    idempotence_digest: Arc<str>,
    authoritative_artifact_digest: Arc<str>,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeWritebackAuthorityOutcome {
    pub fn canonical_noop(idempotence: &BridgeWritebackIdempotenceBasis) -> Self {
        Self::new(
            BridgeWritebackOutcomeClass::CanonicalNoop,
            idempotence,
            derive_authority_outcome_artifact_digest(
                BridgeWritebackOutcomeClass::CanonicalNoop,
                idempotence,
                None,
            ),
        )
    }

    pub fn authoritative_commit(
        idempotence: &BridgeWritebackIdempotenceBasis,
        authority_receipt: &TruthWritebackReceipt,
    ) -> Self {
        Self::new(
            BridgeWritebackOutcomeClass::AuthoritativeCommit,
            idempotence,
            Arc::from(authority_receipt.authoritative_artifact_digest()),
        )
    }

    fn new(
        outcome_class: BridgeWritebackOutcomeClass,
        idempotence: &BridgeWritebackIdempotenceBasis,
        authoritative_artifact_digest: Arc<str>,
    ) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-writeback-authority-outcome|class:{outcome_class:?}|idempotence={}|authoritative={}",
            idempotence.digest(),
            authoritative_artifact_digest.as_ref(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            outcome_class,
            idempotence_digest: Arc::from(idempotence.digest().to_owned()),
            authoritative_artifact_digest,
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-writeback-authority-outcome:sha256:{digest:x}"
            )),
        }
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }

    pub fn outcome_class(&self) -> BridgeWritebackOutcomeClass {
        self.outcome_class
    }

    pub fn idempotence_digest(&self) -> &str {
        self.idempotence_digest.as_ref()
    }

    pub fn authoritative_artifact_digest(&self) -> &str {
        self.authoritative_artifact_digest.as_ref()
    }
}

fn derive_authority_outcome_artifact_digest(
    outcome_class: BridgeWritebackOutcomeClass,
    idempotence: &BridgeWritebackIdempotenceBasis,
    authority_receipt: Option<&TruthWritebackReceipt>,
) -> Arc<str> {
    let canonical_basis = format!(
        "bridge-writeback-authority-outcome-artifact|class:{outcome_class:?}|idempotence={}|causality={}|receipt={}",
        idempotence.digest(),
        idempotence.causality_digest(),
        authority_receipt.map_or("none", TruthWritebackReceipt::digest),
    );
    let digest = Sha256::digest(canonical_basis.as_bytes());
    Arc::from(format!(
        "bridge-writeback-authority-artifact:sha256:{digest:x}"
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeWritebackOutcomeClass {
    CanonicalNoop,
    AuthoritativeCommit,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeWritebackFailureClass {
    WritebackNotRequested,
    PolicyRejected,
    FamilyBindingMismatch,
    StrategyDescriptorMismatch,
    IdempotenceBasisMismatch,
    StaleTruthBasis,
    InvariantRejected,
    StrategyFailed,
    StrategyPanicked,
    AuthorityDenied,
    PreviewWritebackRejected,
}
