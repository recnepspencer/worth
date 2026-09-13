use worth_store::physical_runtime::ObservedRecoveryArtifact;

use super::RecoveryIntegrityIngressRejection;

/// Recovery's route/namespace join. C.4 currently supplies one canonical slot
/// observation; future duplicate-aware discovery must use `Conflicting`
/// rather than choosing one source by traversal order.
pub(crate) enum RecoveryArtifactNamespaceJoin<'media> {
    Observed(&'media ObservedRecoveryArtifact),
    Absent,
    Conflicting { observed_sources: u64 },
}

impl<'media> RecoveryArtifactNamespaceJoin<'media> {
    pub(crate) fn from_canonical(observed: &'media ObservedRecoveryArtifact) -> Self {
        Self::from_namespace(observed.bytes().map(|_| observed), 0)
    }

    fn from_namespace(
        observed: Option<&'media ObservedRecoveryArtifact>,
        conflicting_sources: u64,
    ) -> Self {
        if conflicting_sources > 1 {
            Self::Conflicting {
                observed_sources: conflicting_sources,
            }
        } else {
            observed.map_or(Self::Absent, Self::Observed)
        }
    }

    pub(crate) fn require_observed(
        self,
    ) -> Result<&'media ObservedRecoveryArtifact, RecoveryIntegrityIngressRejection> {
        match self {
            Self::Observed(observed) => Ok(observed),
            Self::Absent => Err(RecoveryIntegrityIngressRejection::Absent),
            Self::Conflicting { observed_sources } => {
                Err(RecoveryIntegrityIngressRejection::ConflictingDuplication { observed_sources })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conflicting_namespace_sources_are_rejected_without_selection() {
        assert_eq!(
            RecoveryArtifactNamespaceJoin::from_namespace(None, 2)
                .require_observed()
                .unwrap_err(),
            RecoveryIntegrityIngressRejection::ConflictingDuplication {
                observed_sources: 2
            }
        );
    }
}
