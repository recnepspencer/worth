use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::WorthQueryManagedLiveCheckpointReceipt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryManagedLiveResumeReceipt {
    continuation_identity: WorthQueryEvidenceIdentity,
    resumed_delivery_sequence: Option<u64>,
    pending_delivery_batch_count: usize,
    resume_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryManagedLiveResumeReceipt {
    pub(super) fn new(
        checkpoint: &WorthQueryManagedLiveCheckpointReceipt,
        resumed_delivery_sequence: Option<u64>,
        pending_delivery_batch_count: usize,
    ) -> Result<Self, IdentityDerivationDenial> {
        let resume_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "worth_query_managed_live_resume_v1",
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("continuation"),
            checkpoint.continuation_identity(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("resumed_delivery_sequence"),
            resumed_delivery_sequence
                .map(|sequence| sequence.to_string())
                .unwrap_or_else(|| "none".to_string()),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("pending_delivery_batch_count"),
            pending_delivery_batch_count,
        )
        .seal()?;
        Ok(Self {
            continuation_identity: checkpoint.continuation_identity().clone(),
            resumed_delivery_sequence,
            pending_delivery_batch_count,
            resume_identity,
        })
    }

    pub fn continuation_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.continuation_identity
    }

    pub fn resumed_delivery_sequence(&self) -> Option<u64> {
        self.resumed_delivery_sequence
    }

    pub fn pending_delivery_batch_count(&self) -> usize {
        self.pending_delivery_batch_count
    }

    pub fn resume_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.resume_identity
    }
}
