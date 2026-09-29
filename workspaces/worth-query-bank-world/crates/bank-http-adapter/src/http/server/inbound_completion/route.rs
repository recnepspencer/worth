//! Fixed route installation and private translation to a signed custody ACK.

use std::io;
use std::sync::Arc;

use bank_server::{
    BankEstateRailCompletionRoute, BankIdentityRuntime, BankPaymentRailCompletionRoute,
};
use ed25519_dalek::SigningKey;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::primary_graph::{
    WorthQueryInboundAdmissionDenial, WorthQueryInboundMaintenanceReport,
    WorthQueryInboundReceiptPosture, WorthQueryInboundTerminalObservation,
    WorthQueryInboundVerificationDenial,
};

use super::configuration::BankRailCompletionServerInstallation;
use crate::http::protocol::inbound_completion::{
    completion_protocol_for_selection, sign_custody_ack, BankCustodyAckPosture,
    CompletionWireDenial, RailCompletionProtocol,
};
use crate::AuthentikBankIdentity;

pub(in crate::http::server) trait BankRailCompletionRuntime:
    Send + Sync + 'static
{
    fn runtime(&self) -> &BankIdentityRuntime;
}

impl BankRailCompletionRuntime for AuthentikBankIdentity {
    fn runtime(&self) -> &BankIdentityRuntime {
        self.runtime()
    }
}

impl BankRailCompletionRuntime for BankIdentityRuntime {
    fn runtime(&self) -> &BankIdentityRuntime {
        self
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct BankRailMaintenancePool {
    pub(super) signed_available_before: usize,
    pub(super) signed_selected: usize,
    pub(super) transport_available_before: usize,
    pub(super) transport_selected: usize,
}

#[derive(Clone, Copy, Debug, Default)]
pub(in crate::http::server) struct BankRailMaintenanceBatch {
    pub(super) selected: usize,
    /// Estate and payment each retain independent signed and transport work.
    pub(super) pools: [BankRailMaintenancePool; 2],
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
            pools: [
                BankRailMaintenancePool {
                    signed_available_before: report.signed_available_before(),
                    signed_selected: report.signed_selected(),
                    transport_available_before: report.transport_available_before(),
                    transport_selected: report.transport_selected(),
                },
                BankRailMaintenancePool::default(),
            ],
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
    estate: BankEstateRailCompletionRoute,
    payment: BankPaymentRailCompletionRoute,
    ack_signer: SigningKey,
}

fn combine_maintenance(
    estate: Result<BankRailMaintenanceBatch, WorthQueryInboundAdmissionDenial>,
    payment: Result<BankRailMaintenanceBatch, WorthQueryInboundAdmissionDenial>,
) -> Result<BankRailMaintenanceBatch, WorthQueryInboundAdmissionDenial> {
    let (estate, payment) = match (estate, payment) {
        (Err(denial), Err(_)) => return Err(denial),
        (estate, payment) => (estate.unwrap_or_default(), payment.unwrap_or_default()),
    };
    Ok(BankRailMaintenanceBatch {
        selected: estate.selected + payment.selected,
        pools: [estate.pools[0], payment.pools[0]],
        advanced: estate.advanced + payment.advanced,
        performed: estate.performed + payment.performed,
        blocked: estate.blocked + payment.blocked,
        reclaimed: estate.reclaimed + payment.reclaimed,
        remaining: estate.remaining + payment.remaining,
        next_expiry_unix_seconds: estate
            .next_expiry_unix_seconds
            .into_iter()
            .chain(payment.next_expiry_unix_seconds)
            .min(),
    })
}

impl<A: BankRailCompletionRuntime> BankRailCompletionRoute for InstalledRoute<A> {
    fn maintain_custody(
        &self,
        request: &WorthQueryRequestScope,
    ) -> Result<BankRailMaintenanceBatch, WorthQueryInboundAdmissionDenial> {
        let estate = self
            .application
            .runtime()
            .maintain_estate_rail_completion(&self.estate, request)
            .map(BankRailMaintenanceBatch::from);
        let payment = self
            .application
            .runtime()
            .maintain_payment_rail_completion(&self.payment, request)
            .map(BankRailMaintenanceBatch::from);
        combine_maintenance(estate, payment)
    }

    fn observe_terminal(
        &self,
        correlation_token: [u8; 32],
    ) -> Option<WorthQueryInboundTerminalObservation> {
        self.application
            .runtime()
            .observe_estate_rail_completion(&self.estate, correlation_token)
            .or_else(|| {
                self.application
                    .runtime()
                    .observe_payment_rail_completion(&self.payment, correlation_token)
            })
    }

    fn receive_and_sign(
        &self,
        envelope: &[u8],
        request: &WorthQueryRequestScope,
    ) -> Result<(Vec<u8>, bool), WorthQueryInboundAdmissionDenial> {
        let receipt =
            match completion_protocol_for_selection(envelope).map_err(|denial| match denial {
                CompletionWireDenial::Oversized => WorthQueryInboundAdmissionDenial::Oversized,
                _ => WorthQueryInboundAdmissionDenial::Verification(
                    WorthQueryInboundVerificationDenial::Malformed,
                ),
            })? {
                RailCompletionProtocol::EstateDeathNotice => self
                    .application
                    .runtime()
                    .receive_estate_rail_completion(&self.estate, envelope, request)?,
                RailCompletionProtocol::ApprovedPaymentSettlement => self
                    .application
                    .runtime()
                    .receive_payment_rail_completion(&self.payment, envelope, request)?,
            };
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

pub(in crate::http::server) fn install<A: BankRailCompletionRuntime>(
    application: Arc<A>,
    configuration: BankRailCompletionServerInstallation,
) -> io::Result<Arc<dyn BankRailCompletionRoute>> {
    let estate = application
        .runtime()
        .install_estate_rail_completion_verifier(Arc::new(configuration.estate_verifier()))
        .map_err(|denial| io::Error::other(format!("rail completion installation: {denial:?}")))?;
    let payment = application
        .runtime()
        .install_payment_rail_completion_verifier(Arc::new(configuration.payment_verifier()))
        .map_err(|denial| {
            io::Error::other(format!("payment completion installation: {denial:?}"))
        })?;
    Ok(Arc::new(InstalledRoute {
        application,
        estate,
        payment,
        ack_signer: configuration.ack_signer(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_failed_operation_does_not_discard_the_other_maintenance_report() {
        let estate = BankRailMaintenanceBatch {
            selected: 4,
            pools: [
                BankRailMaintenancePool {
                    signed_available_before: 4,
                    signed_selected: 4,
                    ..Default::default()
                },
                BankRailMaintenancePool::default(),
            ],
            ..Default::default()
        };
        let payment = BankRailMaintenanceBatch {
            selected: 1,
            pools: [
                BankRailMaintenancePool {
                    transport_available_before: 1,
                    transport_selected: 1,
                    ..Default::default()
                },
                BankRailMaintenancePool::default(),
            ],
            performed: 1,
            ..Default::default()
        };
        let combined = combine_maintenance(Ok(estate), Ok(payment)).unwrap();
        assert_eq!(combined.pools[0].signed_selected, 4);
        assert_eq!(combined.pools[1].transport_selected, 1);
        let denial = WorthQueryInboundAdmissionDenial::RecoveryUnavailable;
        let payment_only = combine_maintenance(Err(denial), Ok(payment)).unwrap();
        assert_eq!(payment_only.performed, 1);
        assert_eq!(payment_only.pools[1].transport_selected, 1);
        let estate_only = combine_maintenance(
            Ok(estate),
            Err(WorthQueryInboundAdmissionDenial::RecoveryUnavailable),
        )
        .unwrap();
        assert_eq!(estate_only.pools[0].signed_selected, 4);
    }
}
