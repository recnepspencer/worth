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
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        authenticated: WorthQueryAuthenticatedInboundOccurrence<'a, Schema>,
    ) -> Result<WorthQueryCorrelatedInboundOccurrence<'a, Schema>, WorthQueryInboundAdmissionDenial>
    {
        use WorthQueryInboundAdmissionDenial as Denial;
        let driver = self
            .runtime
            .dispatch_journal
            .begin_inbound_admission(execution)
            .map_err(|_| Denial::RetryBeforeAcceptance)?;
        self.synchronize_durable_inbound(&driver, execution)?;
        drop(driver);
        let operation = authenticated.operation.as_str();
        let claims = &authenticated.claims;
        let envelope = authenticated.envelope;
        let installed = &authenticated.installed;
        let custody = self
            .runtime
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
            WorthQueryInboundSourcePosture::ControlPending => {
                return Err(Denial::RetryBeforeAcceptance)
            }
        }
        let correlation = ExternalEffectCorrelationIdentity::from_digest(CanonicalDigestId::new(
            claims.correlation_token,
        ));
        installed
            .cost
            .terminal_key_probes
            .fetch_add(1, Ordering::Relaxed);
        match self
            .runtime
            .primary_provider
            .lookup_completed_inbound(&correlation)
        {
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
                return Ok(WorthQueryCorrelatedInboundOccurrence {
                    authenticated,
                    decision: CorrelationDecision::Completed(posture),
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
            .observe_committed_dispatch_outbox_for_correlation(execution, &correlation)
            .map_err(map_owner_read)?;
        installed
            .cost
            .selected_outbox_records
            .fetch_add(1, Ordering::Relaxed);
        let publication = owner.committed_product_publication();
        if owner.relational_runtime_instance_id()
            != self
                .runtime
                .product_runtime
                .source()
                .authoritative_source_profile()
                .runtime_instance_id()
            || publication.product_branch().owner_identity()
                != self.runtime.product_runtime.owner.owner_identity()
            || publication.product_incarnation().owner_identity()
                != self.runtime.product_runtime.owner.owner_identity()
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
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        correlated: WorthQueryCorrelatedInboundOccurrence<'a, Schema>,
    ) -> Result<WorthQueryAdmittedInboundOccurrence<'a, Schema>, WorthQueryInboundAdmissionDenial>
    {
        use WorthQueryInboundAdmissionDenial as Denial;
        let WorthQueryCorrelatedInboundOccurrence {
            authenticated,
            decision,
        } = correlated;
        let driver = self
            .runtime
            .dispatch_journal
            .begin_inbound_admission(execution)
            .map_err(|_| Denial::RetryBeforeAcceptance)?;
        self.synchronize_durable_inbound(&driver, execution)?;
        self.validate_inbound_acceptance_cutoff(&authenticated.claims)?;
        let admission = match decision {
            CorrelationDecision::Completed(posture) => {
                self.accept_completed_inbound(execution, &driver, &authenticated, posture)
            }
            CorrelationDecision::ExistingCustody(existing) => {
                let mut custody = self
                    .runtime
                    .inbound_custody
                    .lock()
                    .unwrap_or_else(|p| p.into_inner());
                map_custody(&mut custody, existing)
            }
            CorrelationDecision::Original(owner) => {
                self.accept_original_inbound(execution, &driver, &authenticated, owner)
            }
        }
        .map_err(|denial| {
            authenticated_owner_conflict(
                denial,
                authenticated.claims.message_identity,
                authenticated.envelope,
            )
        })?;
        Ok(WorthQueryAdmittedInboundOccurrence {
            runtime: authenticated.runtime,
            envelope: authenticated.envelope,
            admission: Some(admission),
        })
    }

    fn validate_inbound_acceptance_cutoff(
        &self,
        claims: &WorthQueryInboundOccurrenceClaims,
    ) -> Result<(), WorthQueryInboundAdmissionDenial> {
        use WorthQueryInboundAdmissionDenial as Denial;
        let sample = self
            .runtime
            .authorization_clock
            .sample(ApplicationCapabilityValidityTimeline::UnixEpochSeconds)
            .map_err(|_| Denial::TimeUnavailable)?;
        let AspectValue::UInt64(now) = sample.value() else {
            return Err(Denial::TimeUnavailable);
        };
        if claims.expires_at_unix_seconds < *now {
            return Err(Denial::Expired);
        }
        Ok(())
    }
}

pub(super) fn authenticated_owner_conflict(
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

/// A retry before acceptance is asked for only while a later read can
/// succeed. A committed basis that is gone, or snapshot identities that never
/// free, keep their exact cause so the sender is not told to retry forever.
fn map_owner_read(
    denial: WorthQueryCommittedDispatchOutboxReadDenial,
) -> WorthQueryInboundAdmissionDenial {
    use WorthQueryCommittedDispatchOutboxReadDenial as Read;
    use WorthQueryInboundAdmissionDenial as Denial;
    match denial {
        Read::ReadExecution(_) => Denial::OwnerReadDenied(denial),
        Read::ForeignPhase => Denial::OwnerReadDenied(denial),
        Read::Missing => Denial::UnknownCorrelation,
        // Publication settles, the index is repaired and snapshots in use
        // are released, so the same callback later succeeds.
        Read::PendingPublication
        | Read::CommittedIndexUnavailable
        | Read::ActiveSnapshotCapacityExhausted { .. } => Denial::RetryBeforeAcceptance,
        Read::ExactCommitUnavailable
        | Read::SnapshotIdentityExhausted
        | Read::ForeignRuntime
        | Read::AmbiguousCorrelation
        | Read::WrongRecordKind
        | Read::NotAuthoritative
        | Read::Malformed
        | Read::CommitMismatch
        | Read::RecordMismatch => Denial::OwnerReadDenied(denial),
    }
}

#[cfg(test)]
#[path = "admit/owner_read_tests.rs"]
mod owner_read_tests;

pub(super) fn map_custody(
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

mod court_doors_for_test;
