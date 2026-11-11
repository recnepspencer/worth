//! Phase 4 custody/fates (1473Ã¢â‚¬â€œ1476) and abandonment (1505Ã¢â‚¬â€œ1506).
//! Correspondence relates uncertainty with actual custody, never just terminal tags.
use super::{
    comparison::{compare_head, HostDivergence, ProjectionPart},
    projection::{HeadProjection, StateProjection},
    receipts::{MutationFate, MutationProjection},
    requests::MutationOutcome,
};
use crate::facade::{DurableIdentityCorrespondence, ObservedDurableIdentity as Id};
use serde::{Deserialize, Serialize};
use worth_foundational::facade::WorldOwnerCoordinate;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationMutationOutcome as Outcome, primary_graph::*,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PreparationStage {
    Opening,
    CheckedOut,
    OpenPending,
    OpenUnresolved,
    OpenEvidenceRequired,
    RecordingOriginalMeaning,
    RecordingCorrectionMaterial,
    BindingNativeProposal,
    PreparingNative,
    PreparingQueryCandidate,
    CompletingNative,
    Publishing,
    PublicationPending,
    PublicationRecoveryRequired,
    RecoveryOnly,
}
impl PreparationStage {
    fn observe(stage: WorthQueryProductPreparationStage) -> Self {
        use WorthQueryProductPreparationStage as S;
        match stage {
            S::Opening => Self::Opening,
            S::CheckedOut => Self::CheckedOut,
            S::OpenPending => Self::OpenPending,
            S::OpenUnresolved => Self::OpenUnresolved,
            S::OpenEvidenceRequired => Self::OpenEvidenceRequired,
            S::RecordingOriginalMeaning => Self::RecordingOriginalMeaning,
            S::RecordingCorrectionMaterial => Self::RecordingCorrectionMaterial,
            S::BindingNativeProposal => Self::BindingNativeProposal,
            S::PreparingNative => Self::PreparingNative,
            S::PreparingQueryCandidate => Self::PreparingQueryCandidate,
            S::CompletingNative => Self::CompletingNative,
            S::Publishing => Self::Publishing,
            S::PublicationPending => Self::PublicationPending,
            S::PublicationRecoveryRequired => Self::PublicationRecoveryRequired,
            S::RecoveryOnly => Self::RecoveryOnly,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum UnresolvedOrigin {
    RetainedPreparation {
        stage: PreparationStage,
        owner: WorldOwnerCoordinate,
        custody: u64,
    },
    ProviderCommitUnwind,
    QuerySnapshotSettlement {
        maximum_active_snapshots: usize,
    },
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UnresolvedEvidence {
    pub before: HeadProjection,
    pub origin: UnresolvedOrigin,
}
impl UnresolvedEvidence {
    pub(super) fn observe(outcome: &MutationOutcome, before: &HeadProjection) -> Option<Self> {
        let origin = if let Outcome::Commit(
            WorthQueryApplicationUncommitted::ProductPreparationPending(p),
        ) = outcome
        {
            let id = p.record_handle().identity();
            Some(UnresolvedOrigin::RetainedPreparation {
                stage: PreparationStage::observe(p.stage()),
                owner: super::projection::owner(id.owner_identity()),
                custody: id.ordinal(),
            })
        } else if let Outcome::Commit(WorthQueryApplicationUncommitted::Indeterminate(e)) = outcome
        {
            if e.recovery() != WorthQueryApplicationCommitRecoveryKind::CommitRecoveryRequired
                || e.cleanup_failure().is_some()
                || e.idempotency_read_denial().is_some()
            {
                return None;
            }
            if e.denial_kind() == Ok(WorthQueryProviderSessionDenialKind::ProviderPanicked) {
                Some(UnresolvedOrigin::ProviderCommitUnwind)
            } else if let Ok(
                WorthQueryProviderSessionDenialKind::ActiveSnapshotCapacityExhausted {
                    maximum_active_snapshots,
                },
            ) = e.denial_kind()
            {
                Some(UnresolvedOrigin::QuerySnapshotSettlement {
                    maximum_active_snapshots,
                })
            } else {
                None
            }
        } else {
            None
        };
        origin.map(|origin| Self {
            before: before.clone(),
            origin,
        })
    }
}
/// A comparison fact only; this value cannot publish, release or resume anything.
pub struct FateCorrespondence {
    _compared: (),
}
impl FateCorrespondence {
    pub fn admit(
        map: &mut DurableIdentityCorrespondence,
        h: &MutationProjection,
        r: &MutationProjection,
        hs: &StateProjection,
        rs: &StateProjection,
    ) -> Result<Self, HostDivergence> {
        let bad = || HostDivergence::Meaning {
            part: ProjectionPart::FateEvidence,
            index: 0,
        };
        if h.fate == r.fate {
            match (&h.unresolved, &r.unresolved) {
                (None, None) => {
                    if matches!(
                        h.fate,
                        MutationFate::Unresolved | MutationFate::ProductPreparationPending
                    ) {
                        return Err(bad());
                    }
                }
                (Some(h), Some(r)) => {
                    compare_head(map, &h.before, &r.before)?;
                    correspond_origin(map, &h.origin, &r.origin)?;
                }
                (None, Some(_)) | (Some(_), None) => return Err(bad()),
            }
            return Ok(Self { _compared: () });
        }
        if h.fate != MutationFate::ProductPreparationPending
            || r.fate != MutationFate::Unresolved
            || h.receipt.is_some()
            || r.receipt.is_some()
        {
            return Err(bad());
        }
        let (Some(h), Some(r)) = (&h.unresolved, &r.unresolved) else {
            return Err(bad());
        };
        let UnresolvedOrigin::RetainedPreparation {
            stage: PreparationStage::Publishing,
            owner,
            custody,
        } = &h.origin
        else {
            return Err(bad());
        };
        if r.origin != UnresolvedOrigin::ProviderCommitUnwind {
            return Err(bad());
        }
        if h.before != hs.head || r.before != rs.head {
            return Err(bad());
        }
        let ([hc], [rc]) = (hs.custody.as_slice(), rs.custody.as_slice()) else {
            return Err(bad());
        };
        if hc.owner != *owner
            || hc.identity != *custody
            || hc.settlement != Some(false)
            || rc.settlement != Some(false)
            || hc.effects != Some(1)
            || rc.effects != Some(1)
            || hc.original_effect.is_none()
            || rc.original_effect.is_none()
            || hc.cause != Some(super::custody::CustodyCause::CallerAbandoned)
            || rc.cause != Some(super::custody::CustodyCause::CallerAbandoned)
        {
            return Err(bad());
        }
        let (Some(he), Some(re)) = (&hc.expected, &rc.expected) else {
            return Err(bad());
        };
        if *he != hs.head || *re != rs.head {
            return Err(bad());
        }
        compare_head(map, &h.before, &r.before)?;
        super::custody::compare(map, &hs.custody, &rs.custody)?;
        map.admit(
            Id::WorldCustody {
                owner: *owner,
                ordinal: *custody,
            },
            Id::WorldCustody {
                owner: rc.owner,
                ordinal: rc.identity,
            },
        )?;
        Ok(Self { _compared: () })
    }
}

fn correspond_origin(
    map: &mut DurableIdentityCorrespondence,
    h: &UnresolvedOrigin,
    r: &UnresolvedOrigin,
) -> Result<(), HostDivergence> {
    let bad = || HostDivergence::Meaning {
        part: ProjectionPart::FateEvidence,
        index: 0,
    };
    match h {
        UnresolvedOrigin::RetainedPreparation {
            stage,
            owner,
            custody,
        } => {
            let UnresolvedOrigin::RetainedPreparation {
                stage: rs,
                owner: ro,
                custody: rc,
            } = r
            else {
                return Err(bad());
            };
            if stage != rs {
                return Err(bad());
            }
            map.admit(
                Id::WorldCustody {
                    owner: *owner,
                    ordinal: *custody,
                },
                Id::WorldCustody {
                    owner: *ro,
                    ordinal: *rc,
                },
            )?;
        }
        UnresolvedOrigin::ProviderCommitUnwind => {
            if r != h {
                return Err(bad());
            }
        }
        UnresolvedOrigin::QuerySnapshotSettlement { .. } => {
            if r != h {
                return Err(bad());
            }
        }
    }
    Ok(())
}
