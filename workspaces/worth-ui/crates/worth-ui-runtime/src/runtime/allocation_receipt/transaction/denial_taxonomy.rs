#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum UiAllocationDenialFamily {
    MissingSelection,
    CandidateMismatch,
    CandidatePlanning,
    Reuse,
    RecomputePending,
    TransactionIdentity,
    GenerationMismatch,
    CommitBudget,
    DurableMutationBudget,
    ResizeBasis,
    PortalAnchor,
    DurableSemanticState,
    CatalogBinding,
    CounterExhaustion,
    SourceSequence,
    SourcePolicy,
    SourceAuthority,
    StaleHostEvidence,
    UnsupportedScrollOwnership,
    ContradictoryScrollOwnership,
    BrokenPortalAnchor,
    NeighborhoodLocality,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAllocationDenialEvidenceIdentity(u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAllocationDenialEvidence {
    identity: UiAllocationDenialEvidenceIdentity,
    family: UiAllocationDenialFamily,
    ordinal: Option<u16>,
    attempted: Option<u16>,
    maximum: Option<u16>,
    reuse_reason: Option<super::UiAllocationReuseDenial>,
    denied_reuse_attempts: u16,
}

impl super::UiAllocationReplanTransactionCommitDenial {
    pub fn evidence(&self) -> UiAllocationDenialEvidence {
        let (family, ordinal, attempted, maximum, reuse_reason) = denial_parts(self);
        evidence(0x10, family, ordinal, attempted, maximum, reuse_reason)
    }
}

impl crate::runtime::UiAllocationInvalidationNarrowingDenial {
    pub fn denial_evidence(&self) -> UiAllocationDenialEvidence {
        use crate::runtime::UiAllocationInvalidationNarrowingDenial as Denial;
        use UiAllocationDenialFamily as Family;
        let (family, ordinal, attempted, maximum) = match self {
            Denial::CardinalityExhausted | Denial::OrdinalExhausted => {
                (Family::CounterExhaustion, None, None, None)
            }
            Denial::SourceCardinalityMismatch {
                invalidations,
                sources,
            } => (
                Family::SourcePolicy,
                None,
                Some(*invalidations),
                Some(*sources),
            ),
            Denial::SourceFamilyMismatch { ordinal }
            | Denial::QuerySettlementFamilyMissing { ordinal } => {
                (Family::SourcePolicy, Some(*ordinal), None, None)
            }
            Denial::GraphTargetNotAdmitted { ordinal }
            | Denial::HostMeasurementTargetNotAdmitted { ordinal }
            | Denial::QueryTargetNotAdmitted { ordinal }
            | Denial::DurableResizeTargetNotAdmitted { ordinal } => {
                (Family::MissingSelection, Some(*ordinal), None, None)
            }
            Denial::QueryAuthorityNotIndexable { ordinal }
            | Denial::QueryBasisMismatch { ordinal }
            | Denial::QueryContractMismatch { ordinal }
            | Denial::QueryConsumptionReceiptMismatch { ordinal } => {
                (Family::SourceAuthority, Some(*ordinal), None, None)
            }
            Denial::HostEvidenceGenerationMismatch { ordinal }
            | Denial::HostNormalizationAuthorityMismatch { ordinal } => {
                (Family::StaleHostEvidence, Some(*ordinal), None, None)
            }
            Denial::PortalAnchorNotAdmitted { ordinal }
            | Denial::PortalAnchorObservationInvalid { ordinal }
            | Denial::PortalAnchorEvidenceStale { ordinal }
            | Denial::PortalAnchorSuccessorBasisDenied { ordinal } => {
                (Family::BrokenPortalAnchor, Some(*ordinal), None, None)
            }
            Denial::ScrollOwnershipNotAdmitted { ordinal } => (
                Family::UnsupportedScrollOwnership,
                Some(*ordinal),
                None,
                None,
            ),
            Denial::ContradictoryScrollOwnership { ordinal } => (
                Family::ContradictoryScrollOwnership,
                Some(*ordinal),
                None,
                None,
            ),
            Denial::ViewportTargetBudgetExceeded {
                ordinal,
                attempted,
                maximum,
            }
            | Denial::DragResizeTargetBudgetExceeded {
                ordinal,
                attempted,
                maximum,
            } => (
                Family::CommitBudget,
                Some(*ordinal),
                Some(*attempted),
                Some(*maximum),
            ),
            Denial::QuerySourceGenerationMismatch { ordinal }
            | Denial::QuerySourceOrderMismatch { ordinal }
            | Denial::QueryExtentUnordered { ordinal } => {
                (Family::GenerationMismatch, Some(*ordinal), None, None)
            }
            Denial::AuthorityCounterExhausted { ordinal } => {
                (Family::CounterExhaustion, Some(*ordinal), None, None)
            }
        };
        evidence(0x11, family, ordinal, attempted, maximum, None)
    }
}

impl crate::runtime::UiScrollOwnerAcquisitionDenial {
    pub fn denial_evidence(&self) -> UiAllocationDenialEvidence {
        use crate::runtime::UiScrollOwnerAcquisitionDenial as Denial;
        use UiAllocationDenialFamily as Family;
        let family = match self {
            Denial::OwnerNotActive
            | Denial::AmbiguousOwner
            | Denial::ReceiptNotActive
            | Denial::SourceNotAdmitted => Family::UnsupportedScrollOwnership,
            Denial::ReceiptGenerationMismatch | Denial::ReceiptEquivalenceMismatch => {
                Family::GenerationMismatch
            }
            Denial::ContradictorySource => Family::ContradictoryScrollOwnership,
            Denial::AuthorityCounterExhausted => Family::CounterExhaustion,
        };
        evidence(0x12, family, None, None, None, None)
    }
}

impl crate::runtime::UiAllocationFrameResolutionDenial {
    pub fn denial_evidence(&self) -> UiAllocationDenialEvidence {
        use crate::runtime::UiAllocationFrameResolutionDenial as Denial;
        let family = match self {
            Denial::SourceSequenceDuplicate { .. }
            | Denial::SourceSequenceRegression { .. }
            | Denial::SourceSequenceGap { .. } => UiAllocationDenialFamily::SourceSequence,
            Denial::UnsupportedSourcePosture | Denial::Policy(_) => {
                UiAllocationDenialFamily::SourcePolicy
            }
        };
        evidence(0x14, family, None, None, None, None)
    }
}

impl crate::graph::UiReplanLocalityDenial {
    pub fn denial_evidence(&self) -> UiAllocationDenialEvidence {
        let family = match self {
            Self::AdmittedGenerationSetChanged => UiAllocationDenialFamily::GenerationMismatch,
            Self::CounterExhausted => UiAllocationDenialFamily::CounterExhaustion,
            Self::EmptyScrollConsequence | Self::ContradictoryScrollConsequence => {
                UiAllocationDenialFamily::ContradictoryScrollOwnership
            }
            Self::EmptyInvalidationSet
            | Self::ConflictingNeighborhoodForTarget
            | Self::ForbiddenRootFallback
            | Self::UnsupportedWideningFamily
            | Self::MissingAdmittedCandidate
            | Self::IncompleteReplacementLineage
            | Self::OverlappingNeighborhoods { .. }
            | Self::OverlappingNeighborhoodSupersessionRequired
            | Self::QueryMeasurementSuccessorDenied
            | Self::HostMeasurementSuccessorDenied => {
                UiAllocationDenialFamily::NeighborhoodLocality
            }
        };
        evidence(0x15, family, None, None, None, None)
    }
}

fn denial_parts(
    denial: &super::UiAllocationReplanTransactionCommitDenial,
) -> (
    UiAllocationDenialFamily,
    Option<u16>,
    Option<u16>,
    Option<u16>,
    Option<super::UiAllocationReuseDenial>,
) {
    use super::UiAllocationReplanTransactionCommitDenial as Denial;
    use UiAllocationDenialFamily as Family;
    match denial {
        Denial::MissingSelection => (Family::MissingSelection, None, None, None, None),
        Denial::CandidateCardinalityMismatch => (Family::CandidateMismatch, None, None, None, None),
        Denial::CandidateNeighborhoodMismatch { ordinal } => {
            (Family::CandidateMismatch, Some(*ordinal), None, None, None)
        }
        Denial::CandidatePlanningDenied { ordinal } => {
            (Family::CandidatePlanning, Some(*ordinal), None, None, None)
        }
        Denial::ReuseDenied { ordinal, reason } => {
            (Family::Reuse, Some(*ordinal), None, None, Some(*reason))
        }
        Denial::RecomputePending { ordinal } => {
            (Family::RecomputePending, Some(*ordinal), None, None, None)
        }
        Denial::TransactionIdentityDenied => (Family::TransactionIdentity, None, None, None, None),
        Denial::StaleTransactionFrame | Denial::AdmittedGenerationSetChanged => {
            (Family::GenerationMismatch, None, None, None, None)
        }
        Denial::CommitBudgetExceeded { attempted, maximum } => (
            Family::CommitBudget,
            None,
            Some(*attempted),
            Some(*maximum),
            None,
        ),
        Denial::DurableMutationBudgetExceeded { attempted, maximum } => (
            Family::DurableMutationBudget,
            None,
            Some(*attempted),
            Some(*maximum),
            None,
        ),
        Denial::ResizeBasisDenied => (Family::ResizeBasis, None, None, None, None),
        Denial::PortalPriorReceiptMismatch { ordinal } => {
            (Family::PortalAnchor, Some(*ordinal), None, None, None)
        }
        Denial::PortalBindingSuccession(_) | Denial::PortalCommitBind(_) => {
            (Family::PortalAnchor, None, None, None, None)
        }
        Denial::AllocationAuthoritySuccession(denial) => {
            use super::UiAllocationAuthoritySuccessionDenial as Authority;
            match denial {
                Authority::ScrollAuthority { ordinal } => (
                    Family::UnsupportedScrollOwnership,
                    Some(*ordinal),
                    None,
                    None,
                    None,
                ),
                Authority::ScrollBinding => {
                    (Family::ContradictoryScrollOwnership, None, None, None, None)
                }
                Authority::PortalAuthority { ordinal } => {
                    (Family::BrokenPortalAnchor, Some(*ordinal), None, None, None)
                }
                Authority::PortalBinding => (Family::BrokenPortalAnchor, None, None, None, None),
                Authority::MissingReplanAdmission { ordinal } => {
                    (Family::SourceAuthority, Some(*ordinal), None, None, None)
                }
                Authority::CatalogCardinalityMismatch
                | Authority::StalePredecessor
                | Authority::DerivedIndexDiverged => {
                    (Family::CatalogBinding, None, None, None, None)
                }
            }
        }
        Denial::DurableSemanticStateMissing => {
            (Family::DurableSemanticState, None, None, None, None)
        }
        Denial::CatalogBindingMismatch => (Family::CatalogBinding, None, None, None, None),
        Denial::AuthorityCounterExhausted(_) | Denial::EvidenceCounterExhausted => {
            (Family::CounterExhaustion, None, None, None, None)
        }
    }
}

fn evidence(
    lane: u64,
    family: UiAllocationDenialFamily,
    ordinal: Option<u16>,
    attempted: Option<u16>,
    maximum: Option<u16>,
    reuse_reason: Option<super::UiAllocationReuseDenial>,
) -> UiAllocationDenialEvidence {
    let mut identity = 0x776f7274682d6465u64 ^ lane.rotate_left(7) ^ family as u64;
    identity ^= u64::from(ordinal.unwrap_or(0)).rotate_left(17);
    identity ^= u64::from(attempted.unwrap_or(0)).rotate_left(31);
    identity ^= u64::from(maximum.unwrap_or(0)).rotate_left(47);
    identity ^= reuse_reason
        .map_or(0, reuse_reason_identity_word)
        .rotate_left(53);
    UiAllocationDenialEvidence {
        identity: UiAllocationDenialEvidenceIdentity(identity),
        family,
        ordinal,
        attempted,
        maximum,
        reuse_reason,
        denied_reuse_attempts: u16::from(family == UiAllocationDenialFamily::Reuse),
    }
}

impl UiAllocationDenialEvidence {
    pub const fn identity(self) -> UiAllocationDenialEvidenceIdentity {
        self.identity
    }
    pub const fn family(self) -> UiAllocationDenialFamily {
        self.family
    }
    pub const fn ordinal(self) -> Option<u16> {
        self.ordinal
    }
    pub const fn attempted(self) -> Option<u16> {
        self.attempted
    }
    pub const fn maximum(self) -> Option<u16> {
        self.maximum
    }
    pub const fn reuse_reason(self) -> Option<super::UiAllocationReuseDenial> {
        self.reuse_reason
    }
    pub const fn denied_reuse_attempts(self) -> u16 {
        self.denied_reuse_attempts
    }
    pub const fn maximum_denied_reuse_attempts(self) -> u16 {
        1
    }
}

const fn reuse_reason_identity_word(reason: super::UiAllocationReuseDenial) -> u64 {
    match reason {
        super::UiAllocationReuseDenial::ReceiptIdentityMismatch => 1,
        super::UiAllocationReuseDenial::GenerationMismatch => 2,
        super::UiAllocationReuseDenial::EquivalenceBasisMismatch => 3,
        super::UiAllocationReuseDenial::UnsupportedPartialReuse => 4,
    }
}

impl UiAllocationDenialEvidenceIdentity {
    pub const fn diagnostic_identity(self) -> u64 {
        self.0
    }
}
