//! Bank's fixed estate rail completion route over Query's installed owner.

use std::num::NonZeroUsize;
use std::sync::Arc;

use bank_domain::schema::NotifyDeathEstateOperation;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::primary_graph::{
    WorthQueryInboundAdmissionDenial, WorthQueryInboundMaintenanceReport,
    WorthQueryInboundOccurrenceVerifier, WorthQueryInboundReceipt,
    WorthQueryInboundTerminalObservation, WorthQueryInboundVerifierHandle,
    WorthQueryInboundVerifierInstallationDenial,
};

use crate::BankIdentityRuntime;

/// Opaque installation selected by the Bank process, never by callback bytes.
pub struct BankEstateRailCompletionRoute {
    handle: WorthQueryInboundVerifierHandle,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankEstateRailCompletionInstallationDenial {
    OperationUnavailable,
    Verifier(WorthQueryInboundVerifierInstallationDenial),
}

impl BankIdentityRuntime {
    pub fn install_estate_rail_completion_verifier(
        &self,
        verifier: Arc<dyn WorthQueryInboundOccurrenceVerifier>,
    ) -> Result<BankEstateRailCompletionRoute, BankEstateRailCompletionInstallationDenial> {
        let operation = self
            .application_runtime()
            .installed_schema()
            .installed_operation(NotifyDeathEstateOperation::reference())
            .map_err(|_| BankEstateRailCompletionInstallationDenial::OperationUnavailable)?;
        let handle = self
            .application_runtime()
            .install_inbound_occurrence_verifier(&operation, verifier)
            .map_err(BankEstateRailCompletionInstallationDenial::Verifier)?;
        Ok(BankEstateRailCompletionRoute { handle })
    }

    pub fn receive_estate_rail_completion(
        &self,
        route: &BankEstateRailCompletionRoute,
        envelope: &[u8],
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryInboundReceipt, WorthQueryInboundAdmissionDenial> {
        self.application_runtime()
            .receive_inbound_occurrence(&route.handle, envelope, request)
    }

    pub fn observe_estate_rail_completion(
        &self,
        route: &BankEstateRailCompletionRoute,
        correlation_token: [u8; 32],
    ) -> Option<WorthQueryInboundTerminalObservation> {
        self.application_runtime()
            .observe_inbound_terminal(&route.handle, correlation_token)
    }

    /// Advance retained custody through this installed route after the source
    /// envelope expires. The Query owner selects a bounded batch internally.
    pub fn maintain_estate_rail_completion(
        &self,
        route: &BankEstateRailCompletionRoute,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryInboundMaintenanceReport, WorthQueryInboundAdmissionDenial> {
        self.application_runtime().maintain_inbound_occurrences(
            &route.handle,
            NonZeroUsize::new(4).expect("fixed positive maintenance batch"),
            request,
        )
    }
}
