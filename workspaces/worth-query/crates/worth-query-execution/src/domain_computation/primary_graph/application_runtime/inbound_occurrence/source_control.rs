//! Source control changes one Home fact before acknowledging host configuration.
use super::*;
use crate::domain_computation::application_aftermath::dispatch_journal::QueryDispatchJournalDenial;
use crate::domain_computation::primary_graph::WorthQueryAdvancementDenial;
use worth_foundational::facade::{DurableStateFamily, InboundSourceControl};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryInboundSourceControlDenial {
    ForeignVerifier,
    Execution(WorthQueryAdvancementDenial),
    DurableStateAdmission(crate::domain_computation::execution_runtime::product_world::WorthQueryDurableStateAdmissionDenial),
    Journal(QueryDispatchJournalDenial),
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Stop accepting new messages from this source throughout the Home.
    /// Already accepted work may finish unless the source is revoked.
    pub fn retire_inbound_occurrence_source(
        &self,
        handle: &WorthQueryInboundVerifierHandle,
    ) -> Result<(), WorthQueryInboundSourceControlDenial> {
        self.control_inbound_source(handle, InboundSourceControl::Retired)
    }

    /// Fence fresh consumption from the source, including accepted work that
    /// has not entered its World attempt. Directory success follows durable recording.
    pub fn revoke_inbound_occurrence_source(
        &self,
        handle: &WorthQueryInboundVerifierHandle,
    ) -> Result<(), WorthQueryInboundSourceControlDenial> {
        self.control_inbound_source(handle, InboundSourceControl::Revoked)
    }

    fn control_inbound_source(
        &self,
        handle: &WorthQueryInboundVerifierHandle,
        posture: InboundSourceControl,
    ) -> Result<(), WorthQueryInboundSourceControlDenial> {
        use WorthQueryInboundSourceControlDenial as Denial;
        let installed = self
            .installed_inbound_verifier(handle)
            .ok_or(Denial::ForeignVerifier)?;
        self.with_host_advancement(|phase| {
            let product = &self.runtime.product_runtime;
            let execution = phase
                .execution_request_for(product)
                .map_err(|cause| Denial::Execution(cause.into()))?;
            product
                .admit_directory_family(DurableStateFamily::InboundSourceControls)
                .map_err(Denial::DurableStateAdmission)?;
            let journal = &self.runtime.dispatch_journal;
            if product.owner.owner_identity().durable().is_some() && !journal.is_durable() {
                return Err(Denial::Journal(
                    QueryDispatchJournalDenial::JournalUnavailable,
                ));
            }
            journal
                .record_source_control(installed.contract.source_identity(), posture, &execution)
                .map_err(Denial::Journal)
        })
        .map_err(Denial::Execution)?
    }

    pub(super) fn inbound_source_consumption_denial(
        &self,
        operation: &str,
    ) -> Option<receive::WorthQueryInboundAdmissionDenial> {
        use receive::WorthQueryInboundAdmissionDenial as Denial;
        match self.inbound_source_posture_for_operation(operation) {
            Some(
                WorthQueryInboundSourcePosture::Active | WorthQueryInboundSourcePosture::Retired,
            ) => None,
            Some(WorthQueryInboundSourcePosture::Revoked) => Some(Denial::SourceRevoked),
            Some(WorthQueryInboundSourcePosture::ControlPending) | None => {
                Some(Denial::RetryBeforeAcceptance)
            }
        }
    }
}
