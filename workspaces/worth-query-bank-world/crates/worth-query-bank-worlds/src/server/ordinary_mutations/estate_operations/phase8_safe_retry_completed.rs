//! Safe retry after the one terminal effect owner already holds completion.
//!
//! 9.17.7 converges synchronous completion, safe retry and inbound completion
//! on one terminal owner. Once that owner holds completion, no new physical
//! attempt is admitted: safe retry returns `AlreadyCompleted` with the live
//! handle, contacts nothing, and the consequence owner still holds one effect.

pub use std::sync::Arc;

pub use bank_external_rail::test_control::FaultScript;
pub use bank_external_rail::LedgerStatus;
pub use bank_server::{BankEstateProgressionDenial, BankRecoveryDenialKind};
pub use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
pub use worth_query_host::facade::primary_graph::{
    WorthQueryInboundOccurrenceClaims, WorthQueryInboundOccurrenceVerifier,
    WorthQueryInboundVerificationDenial,
};
pub use worth_query_host::facade::publication::application_aftermath::WorthQueryPublishedExternalEffectPostureKind;

pub use super::phase8_cross_gate::world;
pub use crate::facade::server::support::request_scope;

/// The Bank process installs its estate rail source before serving. This
/// journey completes through the transport, so no callback is authenticated.
pub struct EstateRailSource;

impl WorthQueryInboundOccurrenceVerifier for EstateRailSource {
    fn audience(&self) -> &str {
        "bank-estate-safe-retry"
    }

    fn source_identity(&self) -> &str {
        "rail-primary"
    }

    fn protocol_identity(&self) -> &BoundaryProtocolIdentity {
        static IDENTITY: BoundaryProtocolIdentity =
            BoundaryProtocolIdentity::new("bank.estate.death-notification");
        &IDENTITY
    }

    fn protocol_version(&self) -> BoundaryProtocolVersion {
        BoundaryProtocolVersion::new(1)
    }

    fn verify(
        &self,
        _envelope: &[u8],
        _now_unix_seconds: u64,
        _maximum_work: std::num::NonZeroU64,
    ) -> Result<WorthQueryInboundOccurrenceClaims, WorthQueryInboundVerificationDenial> {
        Err(WorthQueryInboundVerificationDenial::AuthenticationFailed)
    }
}
