//! Private World publication of an owner-admitted external completion.

mod custody;
mod installed_transport;
mod outcome;
mod progression;
mod transport_progression;

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

#[cfg(test)]
pub(in crate::domain_computation::primary_graph) use installed_transport::InstalledTransportPublicationOutcome;
#[cfg(test)]
pub(in crate::domain_computation::primary_graph) use progression::completion_candidate as authenticated_completion_candidate;
#[cfg(test)]
pub(in crate::domain_computation::primary_graph) use transport_progression::completion_candidate as transport_completion_candidate;
