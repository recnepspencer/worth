use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::identity::{BridgeIdentity, WritebackLoopPreventionIdentityTag};

use super::{
    BridgeDerivedWritebackEffect, BridgeWritebackFeedbackContext,
    BridgeWritebackFeedbackProvenance, BridgeWritebackIdempotenceBasis,
};

pub type BridgeWritebackLoopPreventionIdentity = BridgeIdentity<WritebackLoopPreventionIdentityTag>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeWritebackLoopPreventionReport {
    loop_prevention_identity: BridgeWritebackLoopPreventionIdentity,
    current_feedback_provenance: BridgeWritebackFeedbackProvenance,
    incoming_feedback_context: Option<BridgeWritebackFeedbackContext>,
    idempotence: BridgeWritebackIdempotenceBasis,
    disposition: BridgeWritebackLoopDisposition,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeWritebackLoopPreventionReport {
    pub fn classify(
        effect: &BridgeDerivedWritebackEffect,
        idempotence: &BridgeWritebackIdempotenceBasis,
        incoming_feedback_context: Option<&BridgeWritebackFeedbackContext>,
    ) -> Self {
        let current_feedback_provenance = BridgeWritebackFeedbackProvenance::new(effect);
        let disposition = classify_disposition(
            &current_feedback_provenance,
            idempotence,
            incoming_feedback_context,
        );
        let incoming_feedback_context = incoming_feedback_context.cloned();
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-writeback-loop-prevention|feedback-provenance={}|causality={}|incoming-feedback={}|incoming-causality={}|idempotence={}|disposition:{disposition:?}",
            current_feedback_provenance.digest(),
            current_feedback_provenance.causality_digest(),
            incoming_feedback_context
                .as_ref()
                .map(BridgeWritebackFeedbackContext::provenance_digest)
                .unwrap_or("none"),
            incoming_feedback_context
                .as_ref()
                .map(BridgeWritebackFeedbackContext::causality_digest)
                .unwrap_or("none"),
            idempotence.digest(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());

        Self {
            loop_prevention_identity: BridgeWritebackLoopPreventionIdentity::admit_bridge_owned(
                format!("bridge-writeback-loop-prevention:sha256:{digest:x}"),
            ),
            current_feedback_provenance,
            incoming_feedback_context,
            idempotence: idempotence.clone(),
            disposition,
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-writeback-loop-prevention:sha256:{digest:x}"
            )),
        }
    }

    pub fn loop_prevention_identity(&self) -> &BridgeWritebackLoopPreventionIdentity {
        &self.loop_prevention_identity
    }

    pub fn current_feedback_provenance(&self) -> &BridgeWritebackFeedbackProvenance {
        &self.current_feedback_provenance
    }

    pub fn incoming_feedback_context(&self) -> Option<&BridgeWritebackFeedbackContext> {
        self.incoming_feedback_context.as_ref()
    }

    pub fn idempotence(&self) -> &BridgeWritebackIdempotenceBasis {
        &self.idempotence
    }

    pub fn current_feedback_provenance_digest(&self) -> &str {
        self.current_feedback_provenance.digest()
    }

    pub fn current_causality_digest(&self) -> &str {
        self.current_feedback_provenance.causality_digest()
    }

    pub fn incoming_feedback_provenance_digest(&self) -> Option<&str> {
        self.incoming_feedback_context
            .as_ref()
            .map(BridgeWritebackFeedbackContext::provenance_digest)
    }

    pub fn incoming_feedback_causality_digest(&self) -> Option<&str> {
        self.incoming_feedback_context
            .as_ref()
            .map(BridgeWritebackFeedbackContext::causality_digest)
    }

    pub fn idempotence_digest(&self) -> &str {
        self.idempotence.digest()
    }

    pub fn disposition(&self) -> BridgeWritebackLoopDisposition {
        self.disposition
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

fn classify_disposition(
    current_feedback_provenance: &BridgeWritebackFeedbackProvenance,
    idempotence: &BridgeWritebackIdempotenceBasis,
    incoming_feedback_context: Option<&BridgeWritebackFeedbackContext>,
) -> BridgeWritebackLoopDisposition {
    match incoming_feedback_context {
        None => BridgeWritebackLoopDisposition::AllowAuthoritativeAttempt,
        Some(incoming_feedback_context) => {
            let same_feedback = incoming_feedback_context.provenance_digest()
                == current_feedback_provenance.digest();
            let same_causality = incoming_feedback_context.causality_digest()
                == current_feedback_provenance.causality_digest();
            if same_feedback && same_causality {
                match idempotence.idempotence_class() {
                    crate::writeback::BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression => {
                        BridgeWritebackLoopDisposition::CanonicalNoop
                    }
                    crate::writeback::BridgeWritebackIdempotenceClass::AllowRepeatedAuthorityAttempt => {
                        BridgeWritebackLoopDisposition::RejectAsUnsafeFeedback
                    }
                }
            } else if same_feedback || same_causality {
                BridgeWritebackLoopDisposition::RejectAsUnsafeFeedback
            } else {
                BridgeWritebackLoopDisposition::AllowAuthoritativeAttempt
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeWritebackLoopDisposition {
    AllowAuthoritativeAttempt,
    CanonicalNoop,
    RejectAsUnsafeFeedback,
}
