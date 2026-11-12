//! Private World publication of an owner-admitted external completion.

pub(crate) mod custody;
pub(crate) mod installed_transport;
pub(crate) mod outcome;
pub(crate) mod progression;
pub(in crate::domain_computation) mod recovered;
pub(crate) mod transport_progression;

pub(in crate::domain_computation::primary_graph) use custody::InstalledTransportCloseCustody;
pub(super) use custody::InstalledTransportCompletionCustody;
pub(in crate::domain_computation::primary_graph) use custody::{
    InstalledTransportPendingReason, InstalledTransportResumeOutcome,
};
pub(in crate::domain_computation::primary_graph) use installed_transport::InstalledTransportCompletion;
pub use installed_transport::PerformedInstalledTransportCompletion;
pub use outcome::WorthQueryPerformedInboundCompletion;
pub use outcome::WorthQueryUnpublishedInboundCompletion;
pub(in crate::domain_computation) use outcome::{
    WorthQueryInboundPublicationDenial, WorthQueryInboundPublicationOutcome,
};

mod completion_publication;

pub use recovered::WorthQueryRecordedCompletionDenial;
