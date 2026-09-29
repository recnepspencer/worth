//! Fixed route installation and private translation to a signed custody ACK.

use std::io;
use std::sync::Arc;

use bank_server::BankEstateRailCompletionRoute;
use ed25519_dalek::SigningKey;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::primary_graph::{
    WorthQueryInboundAdmissionDenial, WorthQueryInboundMaintenanceReport,
    WorthQueryInboundReceiptPosture, WorthQueryInboundTerminalObservation,
};

use super::configuration::BankRailCompletionServerInstallation;
use crate::http::protocol::inbound_completion::{sign_custody_ack, BankCustodyAckPosture};
use crate::http::server::authentication::BankHttpApplicationAuthenticator;

#[derive(Clone, Copy, Debug, Default)]
pub(in crate::http::server) struct BankRailMaintenanceBatch {
    pub(super) selected: usize,
    pub(super) signed_available_before: usize,
    pub(super) signed_selected: usize,
    pub(super) transport_available_before: usize,
    pub(super) transport_selected: usize,
    pub(super) advanced: usize,
    pub(super) performed: usize,
    pub(super) blocked: usize,
    pub(super) reclaimed: u64,
    pub(super) remaining: usize,
    pub(super) next_expiry_unix_seconds: Option<u64>,
}

impl From<WorthQueryInboundMaintenanceReport> for BankRailMaintenanceBatch {
    fn from(report: WorthQueryInboundMaintenanceReport) -> Self {
        Self {
            selected: report.selected(),
            signed_available_before: report.signed_available_before(),
            signed_selected: report.signed_selected(),
            transport_available_before: report.transport_available_before(),
            transport_selected: report.transport_selected(),
            advanced: report.advanced(),
            performed: report.performed(),
            blocked: report.blocked(),
            reclaimed: report.reclaimed(),
            remaining: report.remaining(),
            next_expiry_unix_seconds: report.next_expiry_unix_seconds(),
        }
    }
}

pub(in crate::http::server) trait BankRailCompletionRoute:
    Send + Sync
{
    fn receive_and_sign(
        &self,
        envelope: &[u8],
        request: &WorthQueryRequestScope,
    ) -> Result<(Vec<u8>, bool), WorthQueryInboundAdmissionDenial>;
    fn observe_terminal(
        &self,
        correlation_token: [u8; 32],
    ) -> Option<WorthQueryInboundTerminalObservation>;
    fn maintain_custody(
        &self,
        request: &WorthQueryRequestScope,
    ) -> Result<BankRailMaintenanceBatch, WorthQueryInboundAdmissionDenial>;
}

struct InstalledRoute<A> {
    application: Arc<A>,
    route: BankEstateRailCompletionRoute,
    ack_signer: SigningKey,
}

impl<A: BankHttpApplicationAuthenticator> BankRailCompletionRoute for InstalledRoute<A> {
    fn maintain_custody(
        &self,
        request: &WorthQueryRequestScope,
    ) -> Result<BankRailMaintenanceBatch, WorthQueryInboundAdmissionDenial> {
        self.application
            .runtime()
            .maintain_estate_rail_completion(&self.route, request)
            .map(Into::into)
    }

    fn observe_terminal(
        &self,
        correlation_token: [u8; 32],
    ) -> Option<WorthQueryInboundTerminalObservation> {
        self.application
            .runtime()
            .observe_estate_rail_completion(&self.route, correlation_token)
    }

    fn receive_and_sign(
        &self,
        envelope: &[u8],
        request: &WorthQueryRequestScope,
    ) -> Result<(Vec<u8>, bool), WorthQueryInboundAdmissionDenial> {
        let receipt = self.application.runtime().receive_estate_rail_completion(
            &self.route,
            envelope,
            request,
        )?;
        let posture = match receipt.posture() {
            WorthQueryInboundReceiptPosture::AcceptedPending => {
                BankCustodyAckPosture::AcceptedPending
            }
            WorthQueryInboundReceiptPosture::AlreadyAccepted => {
                BankCustodyAckPosture::AlreadyAccepted
            }
            WorthQueryInboundReceiptPosture::Performed => BankCustodyAckPosture::Performed,
            WorthQueryInboundReceiptPosture::AlreadyCompleted => {
                BankCustodyAckPosture::AlreadyCompleted
            }
        };
        let retained_work = receipt.requires_maintenance_cue();
        Ok((
            sign_custody_ack(
                receipt.envelope_digest(),
                *receipt.message_identity(),
                posture,
                &self.ack_signer,
            ),
            retained_work,
        ))
    }
}

pub(in crate::http::server) fn install<A: BankHttpApplicationAuthenticator>(
    application: Arc<A>,
    configuration: BankRailCompletionServerInstallation,
) -> io::Result<Arc<dyn BankRailCompletionRoute>> {
    let route = application
        .runtime()
        .install_estate_rail_completion_verifier(Arc::new(configuration.verifier()))
        .map_err(|denial| io::Error::other(format!("rail completion installation: {denial:?}")))?;
    Ok(Arc::new(InstalledRoute {
        application,
        route,
        ack_signer: configuration.ack_signer(),
    }))
}
