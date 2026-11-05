//! Private World publication of an owner-admitted external completion.

pub(crate) mod custody;
pub(crate) mod installed_transport;
pub(crate) mod outcome;
pub(crate) mod progression;
pub(crate) mod transport_progression;

pub(super) use custody::InstalledTransportCompletionCustody;
pub(in crate::domain_computation::primary_graph) use custody::{
    InstalledTransportPendingReason, InstalledTransportResumeOutcome,
};
pub(in crate::domain_computation::primary_graph) use installed_transport::{
    InstalledTransportCompletion, PerformedInstalledTransportCompletion,
};
pub(in crate::domain_computation) use outcome::{
    WorthQueryInboundPublicationDenial, WorthQueryInboundPublicationOutcome,
    WorthQueryPerformedInboundCompletion, WorthQueryUnpublishedInboundCompletion,
};

