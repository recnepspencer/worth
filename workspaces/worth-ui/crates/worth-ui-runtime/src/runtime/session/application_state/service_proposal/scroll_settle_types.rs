//! What can refuse a Scroll settle publication, and why each refusal is a
//! separate fact the caller can act on.
//!
//! A settle is refused before it reaches Motion for distinct reasons the spec
//! names, and each arrives through the lane's own vocabulary rather than an
//! invented one: an owner that no longer exists or whose incarnation moved on
//! is refused by the Scroll transition staging that produced the request; an
//! application with no Motion service installed is refused by preflight as
//! `UnsupportedFamily(Motion)`.

#[derive(Debug)]
pub(crate) enum UiScrollSettlePublicationDenial {
    /// The published frame carries no presentation basis for the surface the
    /// settling owner lives on, so there is nothing to bind the track to.
    UnpublishedSurface,
    RequestBasis(crate::runtime::session::service_proposal::UiServiceRequestBasisDenial),
    Demand(crate::runtime::session::service_proposal::UiServiceProposalDemandConstructionDenial),
    Preflight(crate::runtime::session::service_proposal::UiServiceProposalPreflightDenial),
    Reservation(crate::runtime::session::service_proposal::UiServiceProposalReservationDenial),
    Staging(crate::runtime::session::service_proposal::UiServiceProposalStagingDenial),
    Publication(crate::runtime::session::service_proposal::UiServiceProposalPublicationDenial),
    MotionStaging(crate::runtime::motion::UiMotionStagingDenial),
    /// An earlier settle of the same owner still holds the occupancy this one
    /// asked for. The incumbent proposal is named so the caller can say which.
    Coalesced(crate::runtime::session::service_proposal::UiServiceProposalIdentity),
}
