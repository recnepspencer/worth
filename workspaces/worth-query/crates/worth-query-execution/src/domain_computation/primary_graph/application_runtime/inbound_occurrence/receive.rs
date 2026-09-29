//! Authentication, exact correlation and owner-retained acceptance.

use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_foundational::facade::{AspectValue, CanonicalDigestId};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;

use super::{
    WorthQueryInboundSourcePosture, WorthQueryInboundVerifierHandle,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::domain_computation::application_aftermath::{
    ExternalEffectCorrelationIdentity, WorthQueryAcceptedInboundOccurrence,
    WorthQueryInboundCustodyAdmission, WorthQueryInboundOccurrenceClaims,
    WorthQueryInboundPublicationClaim, WorthQueryInboundVerificationDenial,
};
use crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxReadDenial;

pub(in crate::domain_computation) enum WorthQueryInboundAdmission {
    New(Arc<WorthQueryAcceptedInboundOccurrence>),
    Duplicate(Arc<WorthQueryAcceptedInboundOccurrence>),
    Completed(
        WorthQueryInboundOccurrenceClaims,
        WorthQueryInboundReceiptPosture,
    ),
}

/// Why an authenticated completion could not enter or finish owner custody.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInboundAdmissionDenial {
    ForeignVerifier,
    Oversized,
    Verification(WorthQueryInboundVerificationDenial),
    IncompatibleMeaning,
    Expired,
    TimeUnavailable,
    UnknownCorrelation,
    RetryBeforeAcceptance,
    ForeignOwner,
    UnsupportedOutbox,
    MessageIdentityConflict,
    CorrelationAlreadyOwned,
    CapacityExhausted,
    TerminalCleanupUnavailable,
    PublicationInProgress,
    PublicationRetryRequired,
    RecoveryStaleProduct,
    RecoveryUnavailable,
    SourceRetired,
    SourceRevoked,
    OwnerReadDenied(WorthQueryCommittedDispatchOutboxReadDenial),
}

/// Owner-confirmed state safe for the transport custodian to acknowledge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInboundReceiptPosture {
    AcceptedPending,
    AlreadyAccepted,
    Performed,
    AlreadyCompleted,
}

/// Exact envelope-bound result for Bank's private custody ACK signer. It does
/// not confer completion authority on the caller.
pub struct WorthQueryInboundReceipt {
    message_identity: [u8; 32],
    envelope_digest: [u8; 32],
    posture: WorthQueryInboundReceiptPosture,
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

    /// An accepted custody entry can leave retained cleanup work even when
    /// its exact terminal is already known to the caller.
    pub const fn requires_maintenance_cue(&self) -> bool {
        self.requires_maintenance_cue
    }

    fn from_accepted(
        accepted: &WorthQueryAcceptedInboundOccurrence,
        envelope: &[u8],
        posture: WorthQueryInboundReceiptPosture,
    ) -> Self {
        Self {
            message_identity: accepted.claims().message_identity,
            envelope_digest: Sha256::digest(envelope).into(),
            posture,
            requires_maintenance_cue: true,
        }
    }

    fn from_claims(
        claims: &WorthQueryInboundOccurrenceClaims,
        envelope: &[u8],
        posture: WorthQueryInboundReceiptPosture,
    ) -> Self {
        Self {
            message_identity: claims.message_identity,
            envelope_digest: Sha256::digest(envelope).into(),
            posture,
            requires_maintenance_cue: false,
        }
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    /// Accept only through the installed verifier and exact original dispatch.
    /// The resulting World publication, if performed, is sealed in aftermath
    /// custody before the original dispatch lease is released.
    pub fn receive_inbound_occurrence(
        &self,
        handle: &WorthQueryInboundVerifierHandle,
        envelope: &[u8],
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryInboundReceipt, WorthQueryInboundAdmissionDenial> {
        use WorthQueryInboundReceiptPosture as Posture;
        let admitted = self.admit_inbound_occurrence(handle, envelope)?;
        let accepted = match admitted {
            WorthQueryInboundAdmission::Completed(claims, posture) => {
                return Ok(WorthQueryInboundReceipt::from_claims(
                    &claims, envelope, posture,
                ));
            }
            WorthQueryInboundAdmission::Duplicate(accepted) => {
                let claim = self
                    .inbound_custody
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .claim_retryable_publication(&accepted);
                match claim {
                    WorthQueryInboundPublicationClaim::Claimed => accepted,
                    WorthQueryInboundPublicationClaim::AtCapacity => {
                        return Err(WorthQueryInboundAdmissionDenial::CapacityExhausted);
                    }
                    WorthQueryInboundPublicationClaim::Publishing => {
                        return Err(WorthQueryInboundAdmissionDenial::PublicationInProgress);
                    }
                    WorthQueryInboundPublicationClaim::Unpublished => {
                        if self.inbound_source_posture_for_operation(accepted.operation())
                            == Some(WorthQueryInboundSourcePosture::Revoked)
                        {
                            return Err(WorthQueryInboundAdmissionDenial::SourceRevoked);
                        }
                        let posture = self.progress_unpublished_inbound_occurrence(
                            Arc::clone(&accepted),
                            request,
                        )?;
                        return Ok(WorthQueryInboundReceipt::from_accepted(
                            &accepted, envelope, posture,
                        ));
                    }
                    WorthQueryInboundPublicationClaim::Terminal => {
                        self.release_retained_inbound_terminal(&accepted)?;
                        return Ok(WorthQueryInboundReceipt::from_accepted(
                            &accepted,
                            envelope,
                            Posture::Performed,
                        ));
                    }
                }
            }
            WorthQueryInboundAdmission::New(accepted) => accepted,
        };
        let posture = self.progress_accepted_inbound_occurrence(Arc::clone(&accepted), request)?;
        Ok(WorthQueryInboundReceipt::from_accepted(
            &accepted, envelope, posture,
        ))
    }

    pub(in crate::domain_computation) fn admit_inbound_occurrence(
        &self,
        handle: &WorthQueryInboundVerifierHandle,
        envelope: &[u8],
    ) -> Result<WorthQueryInboundAdmission, WorthQueryInboundAdmissionDenial> {
        use WorthQueryInboundAdmissionDenial as Denial;
        let installed = self
            .installed_inbound_verifier(handle)
            .ok_or(Denial::ForeignVerifier)?;
        let limits = installed.contract.limits();
        let envelope_len = u64::try_from(envelope.len()).map_err(|_| Denial::Oversized)?;
        if envelope_len > limits.maximum_envelope_bytes.get() {
            return Err(Denial::Oversized);
        }
        let sample = self
            .authorization_clock
            .sample(ApplicationCapabilityValidityTimeline::UnixEpochSeconds)
            .map_err(|_| Denial::TimeUnavailable)?;
        let AspectValue::UInt64(now) = sample.value() else {
            return Err(Denial::TimeUnavailable);
        };
        let claims = installed
            .verifier
            .verify(envelope, *now)
            .map_err(Denial::Verification)?;
        if claims.audience != installed.verifier.audience()
            || claims.source_identity != installed.contract.source_identity()
            || claims.protocol_identity != *installed.contract.protocol().identity()
            || claims.protocol_version != installed.contract.protocol().version()
            || claims.key_epoch == 0
        {
            return Err(Denial::IncompatibleMeaning);
        }
        let payload_len = u64::try_from(claims.payload.len()).map_err(|_| Denial::Oversized)?;
        if payload_len > limits.maximum_payload_bytes.get() {
            return Err(Denial::Oversized);
        }
        let validity_ms = claims
            .expires_at_unix_seconds
            .checked_sub(claims.issued_at_unix_seconds)
            .and_then(|seconds| seconds.checked_mul(1_000))
            .ok_or(Denial::Expired)?;
        if validity_ms > limits.replay_window_milliseconds.get() {
            return Err(Denial::Expired);
        }
        if claims.expires_at_unix_seconds < *now {
            return Err(Denial::Expired);
        }
        let mut custody = self
            .inbound_custody
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(duplicate) = custody.duplicate(handle.operation(), &claims, envelope) {
            return map_custody(duplicate);
        }
        drop(custody);
        match installed.source_posture() {
            WorthQueryInboundSourcePosture::Active => {}
            WorthQueryInboundSourcePosture::Retired => return Err(Denial::SourceRetired),
            WorthQueryInboundSourcePosture::Revoked => return Err(Denial::SourceRevoked),
        }

        let correlation = ExternalEffectCorrelationIdentity::from_digest(CanonicalDigestId::new(
            claims.correlation_token,
        ));
        match self.primary_provider.lookup_completed_inbound(&correlation) {
            Ok(Some(terminal)) => {
                if !terminal.matches_effect(handle.operation(), &claims) {
                    return Err(Denial::CorrelationAlreadyOwned);
                }
                let posture = if terminal.matches_message(&claims) {
                    WorthQueryInboundReceiptPosture::Performed
                } else if terminal.authenticated_message_identity()
                    == Some(&claims.message_identity)
                {
                    return Err(Denial::MessageIdentityConflict);
                } else {
                    WorthQueryInboundReceiptPosture::AlreadyCompleted
                };
                return Ok(WorthQueryInboundAdmission::Completed(claims, posture));
            }
            Ok(None) => {}
            Err(_) => return Err(Denial::RetryBeforeAcceptance),
        }
        let owner = self
            .observe_committed_dispatch_outbox_for_correlation(&correlation)
            .map_err(map_owner_read)?;
        let publication = owner.committed_product_publication();
        if owner.relational_runtime_instance_id()
            != self
                .product_runtime
                .source
                .authoritative_source_profile()
                .runtime_instance_id()
            || publication.product_branch().owner_identity()
                != self.product_runtime.owner.owner_identity()
            || publication.product_incarnation().owner_identity()
                != self.product_runtime.owner.owner_identity()
            || owner.commit_reference() != publication.relational_commit()
        {
            return Err(Denial::ForeignOwner);
        }
        if owner.record().operation_slot() != Some(handle.operation())
            || owner.record().inbound() != Some(&installed.contract)
            || owner.record().effect() != installed.contract.effect()
            || owner.record().correlation_family().as_str() != claims.correlation_family
            || owner.record().protocol_identity() != &claims.protocol_identity
            || owner.record().protocol_version() != claims.protocol_version
            || owner.record().payload() != claims.payload.as_slice()
        {
            return Err(Denial::UnsupportedOutbox);
        }
        custody = self
            .inbound_custody
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        map_custody(custody.accept(
            handle.operation(),
            &installed.contract,
            claims,
            envelope,
            owner,
        ))
    }
}

fn map_owner_read(
    denial: WorthQueryCommittedDispatchOutboxReadDenial,
) -> WorthQueryInboundAdmissionDenial {
    use WorthQueryCommittedDispatchOutboxReadDenial as Read;
    use WorthQueryInboundAdmissionDenial as Denial;
    match denial {
        Read::Missing => Denial::UnknownCorrelation,
        Read::PendingPublication
        | Read::CommittedIndexUnavailable
        | Read::ExactCommitUnavailable
        | Read::ActiveSnapshotCapacityExhausted { .. }
        | Read::SnapshotIdentityExhausted => Denial::RetryBeforeAcceptance,
        other => Denial::OwnerReadDenied(other),
    }
}

fn map_custody(
    admission: WorthQueryInboundCustodyAdmission,
) -> Result<WorthQueryInboundAdmission, WorthQueryInboundAdmissionDenial> {
    use WorthQueryInboundAdmission as Admission;
    use WorthQueryInboundAdmissionDenial as Denial;
    match admission {
        WorthQueryInboundCustodyAdmission::New(value) => Ok(Admission::New(value)),
        WorthQueryInboundCustodyAdmission::Duplicate(value) => Ok(Admission::Duplicate(value)),
        WorthQueryInboundCustodyAdmission::CompactTerminalReplay(claims) => Ok(
            Admission::Completed(claims, WorthQueryInboundReceiptPosture::AlreadyCompleted),
        ),
        WorthQueryInboundCustodyAdmission::MessageIdentityConflict => {
            Err(Denial::MessageIdentityConflict)
        }
        WorthQueryInboundCustodyAdmission::CorrelationAlreadyOwned => {
            Err(Denial::CorrelationAlreadyOwned)
        }
        WorthQueryInboundCustodyAdmission::CapacityExhausted => Err(Denial::CapacityExhausted),
    }
}
