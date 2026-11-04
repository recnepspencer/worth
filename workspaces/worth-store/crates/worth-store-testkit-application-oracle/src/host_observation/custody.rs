//! Detached original recovery meaning; no owner effects or proofs escape.
use super::{
    comparison::{compare_head, HostDivergence, ProjectionPart},
    projection::CustodyProjection,
};
use crate::facade::{DurableIdentityCorrespondence, ObservedDurableIdentity as Id};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum CustodyCause {
    OpenAttemptRetained,
    WorldOccurrencePending,
    OriginalObservationStopped(
        worth_relational::facade::transactions::RelationalExecutionDenialCause,
    ),
    OriginalObservationUnavailable,
    WorldOccurrenceRefused,
    WorldOccurrenceReadmissionRequired,
    SiblingOwnerDenied,
    CallerAbandoned,
    SettlementPending,
    CancellationAfterEffect,
    DeadlineAfterEffect,
    StaleProductHead,
    OwnerLost,
    CorrespondenceRebindRequired,
    DestinationAdmissionDenied,
    RetentionAdmissionDenied,
    ProductPublicationLost,
    RelationalDurabilityUnresolved,
}
impl CustodyCause {
    pub fn observe(cause: worth_runtime_world::facade::ProductUnpublishedCause) -> Self {
        use worth_runtime_world::facade::ProductUnpublishedCause as C;
        match cause {
            C::OpenAttemptRetained => Self::OpenAttemptRetained,
            C::WorldOccurrencePending => Self::WorldOccurrencePending,
            C::OriginalObservationStopped(cause) => Self::OriginalObservationStopped(cause),
            C::OriginalObservationUnavailable => Self::OriginalObservationUnavailable,
            C::WorldOccurrenceRefused => Self::WorldOccurrenceRefused,
            C::WorldOccurrenceReadmissionRequired => Self::WorldOccurrenceReadmissionRequired,
            C::SiblingOwnerDenied => Self::SiblingOwnerDenied,
            C::CallerAbandoned => Self::CallerAbandoned,
            C::SettlementPending => Self::SettlementPending,
            C::CancellationAfterEffect => Self::CancellationAfterEffect,
            C::DeadlineAfterEffect => Self::DeadlineAfterEffect,
            C::StaleProductHead => Self::StaleProductHead,
            C::OwnerLost => Self::OwnerLost,
            C::CorrespondenceRebindRequired => Self::CorrespondenceRebindRequired,
            C::DestinationAdmissionDenied => Self::DestinationAdmissionDenied,
            C::RetentionAdmissionDenied => Self::RetentionAdmissionDenied,
            C::ProductPublicationLost => Self::ProductPublicationLost,
            C::RelationalDurabilityUnresolved => Self::RelationalDurabilityUnresolved,
        }
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum CustodyAction {
    SettleOwnerEffects,
    ReleaseObligations,
    Inspect,
    ReopenRequired,
    StartFreshCompositePublication,
    CloseOwner,
}
impl CustodyAction {
    pub fn observe(action: worth_runtime_world::facade::ProductUnpublishedNextAction) -> Self {
        use worth_runtime_world::facade::ProductUnpublishedNextAction as A;
        match action {
            A::SettleOwnerEffects => Self::SettleOwnerEffects,
            A::ReleaseObligations => Self::ReleaseObligations,
            A::Inspect => Self::Inspect,
            A::ReopenRequired => Self::ReopenRequired,
            A::StartFreshCompositePublication => Self::StartFreshCompositePublication,
            A::CloseOwner => Self::CloseOwner,
        }
    }
}
pub fn compare(
    map: &mut DurableIdentityCorrespondence,
    home: &[CustodyProjection],
    reference: &[CustodyProjection],
) -> Result<(), HostDivergence> {
    if home.len() != reference.len() {
        return Err(HostDivergence::Meaning {
            part: ProjectionPart::CustodyCardinality,
            index: 0,
        });
    }
    for (index, (h, r)) in home.iter().zip(reference).enumerate() {
        let bad = || HostDivergence::Meaning {
            part: ProjectionPart::CustodyState,
            index,
        };
        if h.effects != r.effects
            || h.settlement != r.settlement
            || h.cause != r.cause
            || h.actions != r.actions
        {
            return Err(bad());
        }
        match (&h.expected, &r.expected) {
            (None, None) => (),
            (Some(h), Some(r)) => compare_head(map, h, r)?,
            (None, Some(_)) | (Some(_), None) => return Err(bad()),
        }
        map.admit(Id::WorldOwner(h.owner), Id::WorldOwner(r.owner))?;
        map.admit(
            Id::WorldCustody {
                owner: h.owner,
                ordinal: h.identity,
            },
            Id::WorldCustody {
                owner: r.owner,
                ordinal: r.identity,
            },
        )?;
    }
    Ok(())
}
