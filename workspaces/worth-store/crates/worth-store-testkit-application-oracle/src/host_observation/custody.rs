//! Detached original recovery meaning; no owner effects or proofs escape.
use super::{
    comparison::{compare_head, HostDivergence, ProjectionPart},
    projection::CustodyProjection,
};
use crate::facade::{DurableIdentityCorrespondence, ObservedDurableIdentity as Id};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OriginalOwnerEffect {
    pub attempt_owner: worth_foundational::facade::WorldOwnerCoordinate,
    pub attempt: u64,
    pub native: super::native_semantic::NativeSemanticProjection,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum CustodyCause {
    OpenAttemptRetained,
    RecoveredWithoutProductMovement,
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
            C::RecoveredWithoutProductMovement => Self::RecoveredWithoutProductMovement,
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
    ContinueOriginalPublication,
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
            A::ContinueOriginalPublication => Self::ContinueOriginalPublication,
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
        if h.effects != r.effects || h.settlement != r.settlement {
            return Err(bad());
        }

        match (&h.expected, &r.expected) {
            (None, None) => (),
            (Some(h), Some(r)) => compare_head(map, h, r)?,
            (None, Some(_)) | (Some(_), None) => return Err(bad()),
        }
        match (&h.original_effect, &r.original_effect) {
            (None, None) => (),
            (Some(h), Some(r)) => {
                map.admit(
                    Id::WorldAttempt {
                        owner: h.attempt_owner,
                        ordinal: h.attempt,
                    },
                    Id::WorldAttempt {
                        owner: r.attempt_owner,
                        ordinal: r.attempt,
                    },
                )?;
                super::native_semantic::compare(map, &h.native, &r.native)?;
            }
            (None, Some(_)) | (Some(_), None) => return Err(bad()),
        }
        CustodyMeaningCorrespondence::admit(h, r).map_err(|()| bad())?;
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

/// D5: RecoveredWithoutProductMovement retains the same abandoned original effect.
/// Phase 4 custody/fates and caller abandonment compare its actual meaning;
/// a directory's durable original continuation does not grant a live Memory open.
enum CustodyMeaningCorrespondence {
    Live,
    RecoveredAbandonment,
}
impl CustodyMeaningCorrespondence {
    fn admit(h: &CustodyProjection, r: &CustodyProjection) -> Result<Self, ()> {
        if h.cause == r.cause {
            compare_actions(h, r)?;
            return Ok(Self::Live);
        }
        use worth_foundational::facade::WorldOwnerCoordinate as Owner;
        if matches!(h.owner, Owner::Home { .. })
            && matches!(r.owner, Owner::MemoryOnly { .. })
            && h.cause == Some(CustodyCause::RecoveredWithoutProductMovement)
            && r.cause == Some(CustodyCause::CallerAbandoned)
            && h.settlement == Some(false)
            && r.settlement == Some(false)
            && h.original_effect.is_some()
            && r.original_effect.is_some()
            && h.expected.is_some()
            && r.expected.is_some()
            && h.actions.as_deref()
                == Some(&[
                    CustodyAction::Inspect,
                    CustodyAction::ContinueOriginalPublication,
                    CustodyAction::ReleaseObligations,
                    CustodyAction::CloseOwner,
                ])
            && r.actions.as_deref()
                == Some(&[
                    CustodyAction::StartFreshCompositePublication,
                    CustodyAction::ReleaseObligations,
                    CustodyAction::Inspect,
                ])
        {
            return Ok(Self::RecoveredAbandonment);
        }
        Err(())
    }
}

fn compare_actions(h: &CustodyProjection, r: &CustodyProjection) -> Result<(), ()> {
    if h.actions == r.actions {
        return Ok(());
    }
    // The owner contract derives these two live postures from the actual
    // durable-open custody (World recovery/product_unpublished/actions.rs).
    // Phase 4's unresolved meaning does not grant Home a live Memory release.
    use worth_foundational::facade::WorldOwnerCoordinate as O;
    let home = match h.owner {
        O::Home { .. } => true,
        O::MemoryOnly { .. } => false,
    };
    let memory = match r.owner {
        O::Home { .. } => false,
        O::MemoryOnly { .. } => true,
    };
    if home
        && memory
        && h.cause == Some(CustodyCause::CallerAbandoned)
        && r.cause == Some(CustodyCause::CallerAbandoned)
        && h.settlement == Some(false)
        && r.settlement == Some(false)
        && h.original_effect.is_some()
        && r.original_effect.is_some()
        && h.actions.as_deref() == Some(&[CustodyAction::Inspect, CustodyAction::ReopenRequired])
        && r.actions.as_deref()
            == Some(&[
                CustodyAction::StartFreshCompositePublication,
                CustodyAction::ReleaseObligations,
                CustodyAction::Inspect,
            ])
    {
        return Ok(());
    }
    Err(())
}
