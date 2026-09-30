//! Envelope-bound accepted or terminal result for a transport custodian.

use sha2::{Digest, Sha256};

use crate::domain_computation::application_aftermath::{
    WorthQueryAcceptedInboundOccurrence, WorthQueryInboundOccurrenceClaims,
};

/// Owner-confirmed state safe for the transport custodian to acknowledge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInboundReceiptPosture {
    AcceptedPending,
    AlreadyAccepted,
    Performed,
    AlreadyCompleted,
}

/// Retained work preventing an accepted occurrence from finishing now.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInboundPendingReason {
    PublicationAtCapacity,
    PublicationInProgress,
    RetainedUnpublished,
    SourceRevoked,
    OwnerRetryRequired,
    TerminalCleanupUnavailable,
    RecoveryUnavailable,
    CorrelationConflict,
}

/// Exact envelope-bound result for Bank's private custody ACK signer. It does
/// not confer completion authority on the caller.
pub struct WorthQueryInboundReceipt {
    message_identity: [u8; 32],
    envelope_digest: [u8; 32],
    posture: WorthQueryInboundReceiptPosture,
    pending_reason: Option<WorthQueryInboundPendingReason>,
    requires_maintenance_cue: bool,
}

impl WorthQueryInboundReceipt {
    pub const fn message_identity(&self) -> &[u8; 32] {
        &self.message_identity
    }

    pub const fn envelope_digest(&self) -> &[u8; 32] {
        &self.envelope_digest
    }

    pub const fn posture(&self) -> WorthQueryInboundReceiptPosture {
        self.posture
    }

    pub const fn pending_reason(&self) -> Option<WorthQueryInboundPendingReason> {
        self.pending_reason
    }

    /// An accepted custody entry can leave retained cleanup work even when
    /// its exact terminal is already known to the caller.
    pub const fn requires_maintenance_cue(&self) -> bool {
        self.requires_maintenance_cue
    }

    pub(super) fn from_accepted(
        accepted: &WorthQueryAcceptedInboundOccurrence,
        envelope: &[u8],
        posture: WorthQueryInboundReceiptPosture,
    ) -> Self {
        Self {
            message_identity: accepted.claims().message_identity,
            envelope_digest: Sha256::digest(envelope).into(),
            posture,
            pending_reason: None,
            requires_maintenance_cue: true,
        }
    }

    pub(super) fn from_claims(
        claims: &WorthQueryInboundOccurrenceClaims,
        envelope: &[u8],
        posture: WorthQueryInboundReceiptPosture,
    ) -> Self {
        Self {
            message_identity: claims.message_identity,
            envelope_digest: Sha256::digest(envelope).into(),
            posture,
            pending_reason: None,
            requires_maintenance_cue: false,
        }
    }

    pub(super) fn pending(
        accepted: &WorthQueryAcceptedInboundOccurrence,
        envelope: &[u8],
        posture: WorthQueryInboundReceiptPosture,
        reason: WorthQueryInboundPendingReason,
    ) -> Self {
        let mut receipt = Self::from_accepted(accepted, envelope, posture);
        receipt.pending_reason = Some(reason);
        receipt
    }
}
