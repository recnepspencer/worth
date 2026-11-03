//! Authentication input to the external-effect aftermath owner.
//!
//! Claims are plain data, never a completion grant. Only a verifier installed
//! for an exact operation binding may produce claims that Query considers for
//! correlation and owner admission.

mod claims;
mod custody;
mod terminal;
mod verifier;

pub use claims::WorthQueryInboundOccurrenceClaims;
pub use custody::WorthQueryInboundCleanupReport;
pub(in crate::domain_computation) use custody::{
    WorthQueryAcceptedInboundOccurrence, WorthQueryInboundCustody,
    WorthQueryInboundCustodyAdmission, WorthQueryInboundPublicationClaim,
    WorthQueryInboundRecoveryState, WorthQueryTransportPublicationPermit,
    WorthQueryTransportPublicationPermitDenial,
};
pub(in crate::domain_computation) use terminal::WorthQueryInboundTerminalOwnerResult;
pub use verifier::{WorthQueryInboundOccurrenceVerifier, WorthQueryInboundVerificationDenial};
