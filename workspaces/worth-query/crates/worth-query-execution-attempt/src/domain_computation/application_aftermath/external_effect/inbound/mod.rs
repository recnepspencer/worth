//! Authentication input to the external-effect aftermath owner.
//!
//! Claims are plain data, never a completion grant. Only a verifier installed
//! for an exact operation binding may produce claims that Query considers for
//! correlation and owner admission.

mod claims;
pub mod custody;
mod terminal;
mod verifier;

pub use claims::WorthQueryInboundOccurrenceClaims;
pub use custody::recovery::WorthQueryInboundRecoveryState;
pub use custody::WorthQueryInboundCleanupReport;
pub use custody::{
    WorthQueryAcceptedInboundOccurrence, WorthQueryInboundCustody,
    WorthQueryInboundCustodyAdmission, WorthQueryInboundPublicationClaim,
    WorthQueryTransportPublicationPermitDenial,
};
pub use terminal::WorthQueryInboundTerminalOwnerResult;
pub use verifier::{WorthQueryInboundOccurrenceVerifier, WorthQueryInboundVerificationDenial};

pub use custody::publication_permit::WorthQueryTransportPublicationPermit;
