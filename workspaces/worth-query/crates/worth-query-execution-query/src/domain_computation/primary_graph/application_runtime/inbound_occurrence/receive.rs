//! Authentication, exact correlation and owner-retained acceptance.

use std::sync::atomic::Ordering;
use std::sync::Arc;
use worth_query_execution_attempt::facade::ports::InboundAdmissionDenialInternalPort as _;
use worth_query_execution_attempt::facade::ports::InboundReceiptInternalPort as _;

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
    WorthQueryInboundPublicationClaim,
};
use crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxReadDenial;

pub enum WorthQueryInboundAdmission {
    New(Arc<WorthQueryAcceptedInboundOccurrence>, bool),
    Duplicate(
        Arc<WorthQueryAcceptedInboundOccurrence>,
        WorthQueryInboundPublicationClaim,
    ),
    Completed(
        WorthQueryInboundOccurrenceClaims,
        WorthQueryInboundReceiptPosture,
    ),
}

pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_runtime::inbound_occurrence::WorthQueryInboundAdmissionDenial;

mod admit;
mod denial;
mod home_acceptance;
pub mod phases;
pub mod receipt;
pub use denial::{
    WorthQueryInboundAuthenticatedPermanentDenial, WorthQueryInboundPermanentDenialKind,
};
pub use phases::{
    WorthQueryAdmittedInboundOccurrence, WorthQueryAuthenticatedInboundOccurrence,
    WorthQueryCorrelatedInboundOccurrence,
};
pub use receipt::{
    WorthQueryInboundPendingReason, WorthQueryInboundReceipt, WorthQueryInboundReceiptPosture,
};

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
        // Custody must retain an authenticated external completion even when
        // the caller has canceled publication. The host owns this advancement;
        // the caller's control still governs the publication attempt below.
        self.with_host_advancement(|phase| {
            let phase = &phase;
            self.authenticate_inbound_in_advancement(phase, handle, envelope)?
                .correlate_in_advancement(phase)?
                .accept_in_advancement(phase)?
                .execute_in_advancement(phase, request)
        })
        .map_err(WorthQueryInboundAdmissionDenial::advancement)?
    }

    fn execute_inbound_admission(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        admitted: WorthQueryInboundAdmission,
        envelope: &[u8],
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryInboundReceipt, WorthQueryInboundAdmissionDenial> {
        use WorthQueryInboundReceiptPosture as Posture;
        let accepted = match admitted {
            WorthQueryInboundAdmission::Completed(claims, posture) => {
                return Ok(WorthQueryInboundReceipt::from_claims(
                    &claims, envelope, posture,
                ));
            }
            WorthQueryInboundAdmission::Duplicate(accepted, claim) => match claim {
                WorthQueryInboundPublicationClaim::Claimed => accepted,
                WorthQueryInboundPublicationClaim::AtCapacity => {
                    return Ok(WorthQueryInboundReceipt::pending(
                        &accepted,
                        envelope,
                        Posture::AcceptedPending,
                        WorthQueryInboundPendingReason::PublicationAtCapacity,
                    ));
                }
                WorthQueryInboundPublicationClaim::Publishing => {
                    return Ok(WorthQueryInboundReceipt::pending(
                        &accepted,
                        envelope,
                        Posture::AlreadyAccepted,
                        WorthQueryInboundPendingReason::PublicationInProgress,
                    ));
                }
                WorthQueryInboundPublicationClaim::Unpublished => {
                    if let Some(denial) =
                        self.inbound_source_consumption_denial(accepted.operation())
                    {
                        return self.accepted_progress_error(&accepted, envelope, denial);
                    }
                    let result = self.progress_unpublished_inbound_occurrence(
                        phase,
                        Arc::clone(&accepted),
                        request,
                    );
                    return self.accepted_progress_result(&accepted, envelope, result);
                }
                WorthQueryInboundPublicationClaim::Terminal => {
                    if let Err(denial) =
                        self.release_retained_inbound_terminal(execution, &accepted)
                    {
                        return self.accepted_progress_error(&accepted, envelope, denial);
                    }
                    return Ok(WorthQueryInboundReceipt::from_accepted(
                        &accepted,
                        envelope,
                        Posture::Performed,
                    ));
                }
                WorthQueryInboundPublicationClaim::Gone => {
                    return Err(WorthQueryInboundAdmissionDenial::RetryBeforeAcceptance);
                }
            },
            WorthQueryInboundAdmission::New(accepted, false) => {
                return Ok(WorthQueryInboundReceipt::pending(
                    &accepted,
                    envelope,
                    Posture::AcceptedPending,
                    WorthQueryInboundPendingReason::PublicationAtCapacity,
                ));
            }
            WorthQueryInboundAdmission::New(accepted, true) => accepted,
        };
        let result = self.progress_accepted_inbound_occurrence(
            execution,
            phase,
            Arc::clone(&accepted),
            request,
        );
        self.accepted_progress_result(&accepted, envelope, result)
    }

    fn accepted_progress_result(
        &self,
        accepted: &Arc<WorthQueryAcceptedInboundOccurrence>,
        envelope: &[u8],
        result: Result<WorthQueryInboundReceiptPosture, WorthQueryInboundAdmissionDenial>,
    ) -> Result<WorthQueryInboundReceipt, WorthQueryInboundAdmissionDenial> {
        match result {
            Ok(posture) => {
                let unpublished = posture == WorthQueryInboundReceiptPosture::AcceptedPending
                    && self
                        .runtime
                        .inbound_custody
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .retains_unpublished(accepted);
                if unpublished {
                    Ok(WorthQueryInboundReceipt::pending(
                        accepted,
                        envelope,
                        posture,
                        WorthQueryInboundPendingReason::RetainedUnpublished,
                    ))
                } else {
                    Ok(WorthQueryInboundReceipt::from_accepted(
                        accepted, envelope, posture,
                    ))
                }
            }
            Err(denial) => self.accepted_progress_error(accepted, envelope, denial),
        }
    }

    fn accepted_progress_error(
        &self,
        accepted: &Arc<WorthQueryAcceptedInboundOccurrence>,
        envelope: &[u8],
        denial: WorthQueryInboundAdmissionDenial,
    ) -> Result<WorthQueryInboundReceipt, WorthQueryInboundAdmissionDenial> {
        let retained = self
            .runtime
            .inbound_custody
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .accepted_by_correlation(accepted.owner().record().correlation())
            .is_some_and(|current| Arc::ptr_eq(&current, accepted));
        if !retained {
            return Err(denial);
        }
        let reason = denial.pending_reason();
        Ok(WorthQueryInboundReceipt::pending(
            accepted,
            envelope,
            WorthQueryInboundReceiptPosture::AcceptedPending,
            reason,
        ))
    }
}
