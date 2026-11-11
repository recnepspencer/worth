//! Sealed host progression from authenticated bytes to owner admission.

use std::sync::atomic::Ordering;
use std::sync::Arc;

use worth_foundational::facade::AspectValue;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline;
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryInstalledInboundVerifier;
use super::{
    WorthQueryInboundAdmission, WorthQueryInboundAdmissionDenial as Denial,
    WorthQueryInboundReceipt,
};
use crate::domain_computation::application_aftermath::{
    WorthQueryInboundCustodyAdmission, WorthQueryInboundOccurrenceClaims,
    WorthQueryInboundPublicationClaim,
};
use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::{
    WorthQueryCommittedDispatchOutboxObservation, WorthQueryInboundVerifierHandle,
};

/// Exact bounded envelope authenticated by this runtime's installed verifier.
/// Fields and constructor are private; signed bytes alone cannot mint this phase.
pub struct WorthQueryAuthenticatedInboundOccurrence<'a, Schema> {
    pub(super) runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(super) operation: String,
    pub(super) installed: Arc<WorthQueryInstalledInboundVerifier>,
    pub(super) envelope: &'a [u8],
    pub(super) claims: WorthQueryInboundOccurrenceClaims,
}

pub(super) enum CorrelationDecision {
    Original(WorthQueryCommittedDispatchOutboxObservation),
    ExistingCustody(WorthQueryInboundCustodyAdmission),
    Completed(super::WorthQueryInboundReceiptPosture),
}

/// Authenticated meaning tied to exact owner provenance or retained custody.
pub struct WorthQueryCorrelatedInboundOccurrence<'a, Schema> {
    pub(super) authenticated: WorthQueryAuthenticatedInboundOccurrence<'a, Schema>,
    pub(super) decision: CorrelationDecision,
}

/// Finite accepted custody, or an already completed exact owner result.
pub struct WorthQueryAdmittedInboundOccurrence<'a, Schema: ApplicationSchema> {
    pub(super) runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(super) envelope: &'a [u8],
    pub(super) admission: Option<WorthQueryInboundAdmission>,
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Authenticate exact bytes against the installed source, sampled Query
    /// clock and finite verifier work before any correlation or owner lookup.
    pub fn authenticate_inbound_occurrence<'a>(
        &'a self,
        handle: &WorthQueryInboundVerifierHandle,
        envelope: &'a [u8],
    ) -> Result<WorthQueryAuthenticatedInboundOccurrence<'a, Schema>, Denial> {
        self.with_host_advancement(|phase| {
            self.authenticate_inbound_in_advancement(&phase, handle, envelope)
        })
        .map_err(Denial::advancement)?
    }

    /// The phase witnesses admission before verifier, provenance, or custody access.
    pub(super) fn authenticate_inbound_in_advancement<'a>(
        &'a self,
        _phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        handle: &WorthQueryInboundVerifierHandle,
        envelope: &'a [u8],
    ) -> Result<WorthQueryAuthenticatedInboundOccurrence<'a, Schema>, Denial> {
        let installed = self
            .installed_inbound_verifier(handle)
            .ok_or(Denial::ForeignVerifier)?;
        let limits = installed.contract.limits();
        let envelope_len = u64::try_from(envelope.len()).map_err(|_| Denial::Oversized)?;
        if envelope_len > limits.maximum_envelope_bytes.get() {
            return Err(Denial::Oversized);
        }
        installed
            .cost
            .verifier_input_bytes
            .fetch_add(envelope_len, Ordering::Relaxed);
        let sample = self
            .runtime
            .authorization_clock
            .sample(ApplicationCapabilityValidityTimeline::UnixEpochSeconds)
            .map_err(|_| Denial::TimeUnavailable)?;
        let AspectValue::UInt64(now) = sample.value() else {
            return Err(Denial::TimeUnavailable);
        };
        let claims = installed
            .verifier
            .verify(envelope, *now, limits.maximum_verifier_work)
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
            return Err(Denial::ValidityWindowExceeded);
        }
        // Installed skew is zero; the Query clock is the cutoff authority.
        if claims.expires_at_unix_seconds < *now {
            return Err(Denial::Expired);
        }
        Ok(WorthQueryAuthenticatedInboundOccurrence {
            runtime: self,
            operation: handle.operation().to_owned(),
            installed,
            envelope,
            claims,
        })
    }
}

impl<'a, Schema: ApplicationSchema> WorthQueryAuthenticatedInboundOccurrence<'a, Schema> {
    /// Resolve exact owner provenance. This does not yet reserve accepted custody.
    pub fn correlate(self) -> Result<WorthQueryCorrelatedInboundOccurrence<'a, Schema>, Denial> {
        let runtime = self.runtime;
        runtime
            .with_host_advancement(|phase| self.correlate_in_advancement(&phase))
            .map_err(Denial::advancement)?
    }

    /// The phase witnesses admission before verifier, provenance, or custody access.
    pub(super) fn correlate_in_advancement(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> Result<WorthQueryCorrelatedInboundOccurrence<'a, Schema>, Denial> {
        let execution = phase
            .execution_request_for(&self.runtime.runtime.product_runtime)
            .map_err(|cause| Denial::advancement(cause.into()))?;
        self.runtime
            .correlate_authenticated_inbound(&execution, self)
    }
}

impl<'a, Schema: ApplicationSchema> WorthQueryCorrelatedInboundOccurrence<'a, Schema> {
    /// Reserve finite owner custody or return the already retained terminal.
    pub fn accept(self) -> Result<WorthQueryAdmittedInboundOccurrence<'a, Schema>, Denial> {
        let runtime = self.authenticated.runtime;
        runtime
            .with_host_advancement(|phase| self.accept_in_advancement(&phase))
            .map_err(Denial::advancement)?
    }

    /// The phase witnesses admission before verifier, provenance, or custody access.
    pub(super) fn accept_in_advancement(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> Result<WorthQueryAdmittedInboundOccurrence<'a, Schema>, Denial> {
        let execution = phase
            .execution_request_for(&self.authenticated.runtime.runtime.product_runtime)
            .map_err(|cause| Denial::advancement(cause.into()))?;
        self.authenticated
            .runtime
            .accept_correlated_inbound(&execution, self)
    }
}

impl<Schema: ApplicationSchema> WorthQueryAdmittedInboundOccurrence<'_, Schema> {
    /// Progress the same owner path as the common callback entry.
    pub fn execute(
        mut self,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryInboundReceipt, Denial> {
        self.runtime
            .with_host_advancement(|phase| {
                let phase = &phase;
                self.progress(phase, request)
            })
            .map_err(Denial::advancement)?
    }

    /// Consume accepted custody under the supplied actual runtime advancement.
    pub fn execute_in_advancement(
        mut self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryInboundReceipt, Denial> {
        self.progress(phase, request)
    }

    fn progress(
        &mut self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryInboundReceipt, Denial> {
        let execution = phase
            .execution_request_for(&self.runtime.runtime.product_runtime)
            .map_err(|cause| Denial::advancement(cause.into()))?;
        // Progression owns settlement of this claim. Disarm Drop before it can
        // become retryable and another caller can claim the same custody slot.
        let admission = self.admission.take().expect("sealed admitted phase");
        self.runtime
            .execute_inbound_admission(&execution, phase, admission, self.envelope, request)
    }

    pub fn progress_for_test(
        &mut self,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryInboundReceipt, Denial> {
        self.runtime
            .with_host_advancement(|phase| {
                let phase = &phase;
                self.progress(phase, request)
            })
            .map_err(Denial::advancement)?
    }

    pub(in crate::domain_computation) fn into_admission(mut self) -> WorthQueryInboundAdmission {
        self.admission.take().expect("sealed admitted phase")
    }
}

impl<Schema: ApplicationSchema> Drop for WorthQueryAdmittedInboundOccurrence<'_, Schema> {
    fn drop(&mut self) {
        let accepted = match self.admission.as_ref() {
            Some(WorthQueryInboundAdmission::New(accepted, true))
            | Some(WorthQueryInboundAdmission::Duplicate(
                accepted,
                WorthQueryInboundPublicationClaim::Claimed,
            )) => accepted,
            _ => return,
        };
        self.runtime
            .runtime
            .inbound_custody
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .mark_publication_retryable(accepted);
    }
}

impl<Schema: ApplicationSchema> WorthQueryAuthenticatedInboundOccurrence<'_, Schema> {
    /// Recorded DATA from this exact authenticated phase and original owner.
    /// A raw claims value cannot enter the journal's acceptance operation.
    pub(in crate::domain_computation) fn accepted_message_record(
        &self,
        journal: &crate::domain_computation::application_aftermath::dispatch_journal::WorthQueryDispatchJournal,
        owner: &crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation,
    ) -> Result<crate::domain_computation::application_aftermath::dispatch_journal::QueryInboundAcceptedMessage,
    crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchJournalDenial>{
        use crate::domain_computation::application_aftermath::dispatch_journal::{
            QueryDispatchJournalDenial, QueryInboundAcceptedMessage, QueryInboundDispatchSource,
        };
        if !std::ptr::eq(self.runtime.runtime.dispatch_journal.as_ref(), journal)
            || owner.relational_runtime_instance_id()
                != self
                    .runtime
                    .runtime
                    .product_runtime
                    .source()
                    .authoritative_source_profile()
                    .runtime_instance_id()
            || owner
                .committed_product_publication()
                .product_branch()
                .owner_identity()
                != self.runtime.runtime.product_runtime.owner.owner_identity()
        {
            return Err(QueryDispatchJournalDenial::ForeignHome);
        }
        let record = owner.record();
        if record.operation_slot() != Some(self.operation.as_str())
            || record.inbound() != Some(&self.installed.contract)
            || record.effect() != self.installed.contract.effect()
            || record.correlation().bytes() != &self.claims.correlation_token
            || record.correlation_family().as_str() != self.claims.correlation_family
            || record.protocol_identity() != &self.claims.protocol_identity
            || record.protocol_version() != self.claims.protocol_version
            || record.payload() != self.claims.payload
        {
            return Err(QueryDispatchJournalDenial::MalformedFrame);
        }
        Ok(QueryInboundAcceptedMessage {
            operation: self.operation.clone(),
            claims: self.claims.clone(),
            envelope_bytes: self.envelope.len() as u64,
            source: QueryInboundDispatchSource::from_owner(owner)?,
        })
    }
}
