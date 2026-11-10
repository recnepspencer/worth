use crate::memory_workspace::WorthQueryEntityIdentity;

/// A domain executor's descriptive correspondence observation. These shapes
/// can report candidates, ambiguity, or a break; none can mint continuity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryInstalledCorrespondenceObservation {
    AdvisoryCandidatePair {
        subject: WorthQueryEntityIdentity,
        candidate: WorthQueryEntityIdentity,
    },
    AmbiguousCandidatePair {
        subject: WorthQueryEntityIdentity,
        candidate: WorthQueryEntityIdentity,
    },
    ExplicitContinuityBreak {
        subject: WorthQueryEntityIdentity,
        rejected_candidate: WorthQueryEntityIdentity,
    },
}

impl WorthQueryInstalledCorrespondenceObservation {
    pub fn advisory_candidate_pair(
        subject: WorthQueryEntityIdentity,
        candidate: WorthQueryEntityIdentity,
    ) -> Self {
        Self::AdvisoryCandidatePair { subject, candidate }
    }

    pub fn ambiguous_candidate_pair(
        subject: WorthQueryEntityIdentity,
        candidate: WorthQueryEntityIdentity,
    ) -> Self {
        Self::AmbiguousCandidatePair { subject, candidate }
    }

    pub fn explicit_continuity_break(
        subject: WorthQueryEntityIdentity,
        rejected_candidate: WorthQueryEntityIdentity,
    ) -> Self {
        Self::ExplicitContinuityBreak {
            subject,
            rejected_candidate,
        }
    }

    pub(crate) fn into_engine_comparison(
        self,
    ) -> crate::identity_evolution::CorrespondenceIdentityComparison {
        let identities = match &self {
            Self::AdvisoryCandidatePair { subject, candidate }
            | Self::AmbiguousCandidatePair { subject, candidate }
            | Self::ExplicitContinuityBreak {
                subject,
                rejected_candidate: candidate,
            } => (
                subject.evidence_identity().as_str().to_owned(),
                candidate.evidence_identity().as_str().to_owned(),
            ),
        };
        match self {
            Self::AdvisoryCandidatePair { .. } => {
                crate::identity_evolution::CorrespondenceIdentityComparison::advisory_between(
                    identities.0,
                    identities.1,
                )
            }
            Self::AmbiguousCandidatePair { .. } => {
                crate::identity_evolution::CorrespondenceIdentityComparison::ambiguous_between(
                    identities.0,
                    identities.1,
                )
            }
            Self::ExplicitContinuityBreak { .. } => {
                crate::identity_evolution::CorrespondenceIdentityComparison::explicit_break(
                    identities.0,
                    identities.1,
                )
            }
        }
    }
}
