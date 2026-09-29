//! Private World publication of an owner-admitted external completion.

mod custody;
mod installed_transport;
mod outcome;
mod progression;
#[cfg(test)]
mod tests;
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
