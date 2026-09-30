//! Exact correlation and finite custody admission after installed authentication.

use super::phases::{
    CorrelationDecision, WorthQueryAdmittedInboundOccurrence,
    WorthQueryAuthenticatedInboundOccurrence, WorthQueryCorrelatedInboundOccurrence,
};
use super::WorthQueryInboundPermanentDenialKind as Permanent;
use super::*;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    #[cfg(test)]
    pub(in crate::domain_computation) fn admit_inbound_occurrence(
        &self,
        handle: &WorthQueryInboundVerifierHandle,
        envelope: &[u8],
    ) -> Result<WorthQueryInboundAdmission, WorthQueryInboundAdmissionDenial> {
        self.authenticate_inbound_occurrence(handle, envelope)?
            .correlate()?
            .accept()
            .map(WorthQueryAdmittedInboundOccurrence::into_admission)
    }

    pub(super) fn correlate_authenticated_inbound<'a>(
        &'a self,
        authenticated: WorthQueryAuthenticatedInboundOccurrence<'a, Schema>,
    ) -> Result<WorthQueryCorrelatedInboundOccurrence<'a, Schema>, WorthQueryInboundAdmissionDenial>
    {
        use WorthQueryInboundAdmissionDenial as Denial;
        let operation = authenticated.operation.as_str();
        let claims = &authenticated.claims;
        let envelope = authenticated.envelope;
        let installed = &authenticated.installed;
        let custody = self
            .inbound_custody
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        installed
            .cost
            .custody_key_probes
            .fetch_add(1, Ordering::Relaxed);
        if let Some(duplicate) = custody.duplicate(operation, claims, envelope) {
            return Ok(WorthQueryCorrelatedInboundOccurrence {
                authenticated,
                decision: CorrelationDecision::ExistingCustody(duplicate),
            });
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
        installed
            .cost
            .terminal_key_probes
            .fetch_add(1, Ordering::Relaxed);
        match self.primary_provider.lookup_completed_inbound(&correlation) {
            Ok(Some(terminal)) => {
                if !terminal.matches_effect(operation, claims) {
                    return Err(authenticated_owner_conflict(
                        Denial::CorrelationAlreadyOwned,
                        claims.message_identity,
                        envelope,
                    ));
                }
                let posture = if terminal.matches_message(claims) {
                    WorthQueryInboundReceiptPosture::Performed
                } else if terminal.authenticated_message_identity()
                    == Some(&claims.message_identity)
                    && terminal.authenticated_key_epoch() == Some(claims.key_epoch)
                {
                    return Err(authenticated_owner_conflict(
                        Denial::MessageIdentityConflict,
                        claims.message_identity,
                        envelope,
                    ));
                } else {
                    WorthQueryInboundReceiptPosture::AlreadyCompleted
                };
                let completed_claims = claims.clone();
                return Ok(WorthQueryCorrelatedInboundOccurrence {
                    authenticated,
                    decision: CorrelationDecision::Completed(completed_claims, posture),
                });
            }
            Ok(None) => {}
            Err(_) => return Err(Denial::RetryBeforeAcceptance),
        }
        installed
            .cost
            .outbox_key_probes
            .fetch_add(1, Ordering::Relaxed);
        let owner = self
            .observe_committed_dispatch_outbox_for_correlation(&correlation)
            .map_err(map_owner_read)?;
        installed
            .cost
            .selected_outbox_records
            .fetch_add(1, Ordering::Relaxed);
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
        if owner.record().inbound().is_none() {
            return Err(Denial::OriginalDispatchHasNoInboundSupport);
        }
        if owner.record().operation_slot() != Some(operation)
            || owner.record().inbound() != Some(&installed.contract)
            || owner.record().effect() != installed.contract.effect()
            || owner.record().correlation_family().as_str() != claims.correlation_family
            || owner.record().protocol_identity() != &claims.protocol_identity
            || owner.record().protocol_version() != claims.protocol_version
            || owner.record().payload() != claims.payload.as_slice()
        {
            return Err(Denial::UnsupportedOutbox);
        }
        Ok(WorthQueryCorrelatedInboundOccurrence {
            authenticated,
            decision: CorrelationDecision::Original(owner),
        })
    }

    pub(super) fn accept_correlated_inbound<'a>(
        &'a self,
        correlated: WorthQueryCorrelatedInboundOccurrence<'a, Schema>,
    ) -> Result<WorthQueryAdmittedInboundOccurrence<'a, Schema>, WorthQueryInboundAdmissionDenial>
    {
        use WorthQueryInboundAdmissionDenial as Denial;
        let WorthQueryCorrelatedInboundOccurrence {
            authenticated,
            decision,
        } = correlated;
        let WorthQueryAuthenticatedInboundOccurrence {
            runtime,
            operation,
            installed,
            envelope,
            claims,
        } = authenticated;
        // A caller may hold either public phase beyond the signed cutoff.
        let sample = self
            .authorization_clock
            .sample(ApplicationCapabilityValidityTimeline::UnixEpochSeconds)
            .map_err(|_| Denial::TimeUnavailable)?;
        let AspectValue::UInt64(now) = sample.value() else {
            return Err(Denial::TimeUnavailable);
        };
        if claims.expires_at_unix_seconds < *now {
            return Err(Denial::Expired);
        }
        let admission = match decision {
            CorrelationDecision::Completed(claims, posture) => {
                WorthQueryInboundAdmission::Completed(claims, posture)
            }
            CorrelationDecision::ExistingCustody(existing) => {
                let mut custody = self
                    .inbound_custody
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                map_custody(&mut custody, existing).map_err(|denial| {
                    authenticated_owner_conflict(denial, claims.message_identity, envelope)
                })?
            }
            CorrelationDecision::Original(owner) => {
                // Recheck the source gate at the actual custody cutover.
                match installed.source_posture() {
                    WorthQueryInboundSourcePosture::Active => {}
                    WorthQueryInboundSourcePosture::Retired => return Err(Denial::SourceRetired),
                    WorthQueryInboundSourcePosture::Revoked => return Err(Denial::SourceRevoked),
                }
                let mut custody = self
                    .inbound_custody
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                installed
                    .cost
                    .custody_key_probes
                    .fetch_add(1, Ordering::Relaxed);
                let message_identity = claims.message_identity;
                let admission =
                    custody.accept(&operation, &installed.contract, claims, envelope, owner);
                map_custody(&mut custody, admission).map_err(|denial| {
                    authenticated_owner_conflict(denial, message_identity, envelope)
                })?
            }
        };
        Ok(WorthQueryAdmittedInboundOccurrence {
            runtime,
            envelope,
            admission: Some(admission),
        })
    }
}

fn authenticated_owner_conflict(
    denial: WorthQueryInboundAdmissionDenial,
    message_identity: [u8; 32],
    envelope: &[u8],
) -> WorthQueryInboundAdmissionDenial {
    let kind = match denial {
        WorthQueryInboundAdmissionDenial::MessageIdentityConflict => {
            Permanent::MessageIdentityConflict
        }
        WorthQueryInboundAdmissionDenial::CorrelationAlreadyOwned => {
            Permanent::CorrelationAlreadyOwned
        }
        other => return other,
    };
    WorthQueryInboundAdmissionDenial::AuthenticatedPermanent(
        WorthQueryInboundAuthenticatedPermanentDenial::seal(kind, message_identity, envelope),
    )
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
    custody: &mut crate::domain_computation::application_aftermath::WorthQueryInboundCustody,
    admission: WorthQueryInboundCustodyAdmission,
) -> Result<WorthQueryInboundAdmission, WorthQueryInboundAdmissionDenial> {
    use WorthQueryInboundAdmission as Admission;
    use WorthQueryInboundAdmissionDenial as Denial;
    match admission {
        WorthQueryInboundCustodyAdmission::New(value, publish_now) => {
            Ok(Admission::New(value, publish_now))
        }
        WorthQueryInboundCustodyAdmission::Duplicate(value) => {
            let claim = custody.claim_retryable_publication(&value);
            Ok(Admission::Duplicate(value, claim))
        }
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
