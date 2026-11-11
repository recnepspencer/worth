use crate::basis_lifecycle::BasisOperationLane;

pub(crate) struct WorthQueryAdmittedProjectionSharing {
    evidence: WorthQueryProjectionSharingEvidence,
    subject_source_identity: String,
    subject_closure_evidence:
        crate::domain_installation::WorthQuerySemanticDependencyClosureEvidence,
    subject_affinity: crate::domain_installation::WorthQueryOperationAuthorityBasis,
    candidate: Option<WorthQueryAdmittedSharingCandidate>,
}

enum WorthQueryProjectionSharingEvidence {
    Singleton(crate::domain_installation::WorthQuerySemanticDependencyClosureEvidence),
    Equivalent(crate::domain_installation::WorthQueryDependencyClosureReuseWitness),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorthQueryProjectionSharingContinuity {
    Singleton,
    Equivalent,
}

struct WorthQueryAdmittedSharingCandidate {
    source_identity: String,
    affinity: crate::domain_installation::WorthQueryOperationAuthorityBasis,
}

impl WorthQueryAdmittedProjectionSharing {
    pub(crate) const fn continuity(&self) -> WorthQueryProjectionSharingContinuity {
        match &self.evidence {
            WorthQueryProjectionSharingEvidence::Singleton(_) => {
                WorthQueryProjectionSharingContinuity::Singleton
            }
            WorthQueryProjectionSharingEvidence::Equivalent(_) => {
                WorthQueryProjectionSharingContinuity::Equivalent
            }
        }
    }

    pub(super) fn equivalent<D, O, F, L: BasisOperationLane>(
        reuse: crate::domain_installation::WorthQueryDependencyClosureReuseWitness,
        subject: &super::super::WorthQuerySettledDomainProjection<D, O, F, L>,
        candidate: &super::super::WorthQuerySettledDomainProjection<D, O, F, L>,
    ) -> Self {
        use crate::domain_installation::operation_authority_chain::operation_phase_basis;
        Self {
            evidence: WorthQueryProjectionSharingEvidence::Equivalent(reuse),
            subject_source_identity: subject.identity().to_string(),
            subject_closure_evidence: subject
                .semantic_aspect_dependency_closure()
                .closure_evidence(),
            subject_affinity: operation_phase_basis(subject.bound_operation().authority_proof())
                .clone(),
            candidate: Some(WorthQueryAdmittedSharingCandidate {
                source_identity: candidate.identity().to_string(),
                affinity: operation_phase_basis(candidate.bound_operation().authority_proof())
                    .clone(),
            }),
        }
    }

    pub(super) fn singleton<D, O, F, L: BasisOperationLane>(
        subject: &super::super::WorthQuerySettledDomainProjection<D, O, F, L>,
    ) -> Self {
        use crate::domain_installation::operation_authority_chain::operation_phase_basis;
        let closure_evidence = subject
            .semantic_aspect_dependency_closure()
            .closure_evidence();
        Self {
            evidence: WorthQueryProjectionSharingEvidence::Singleton(closure_evidence),
            subject_source_identity: subject.identity().to_string(),
            subject_closure_evidence: closure_evidence,
            subject_affinity: operation_phase_basis(subject.bound_operation().authority_proof())
                .clone(),
            candidate: None,
        }
    }

    pub(crate) fn subject_source_identity(&self) -> &str {
        &self.subject_source_identity
    }

    pub(crate) fn subject_affinity(
        &self,
    ) -> &crate::domain_installation::WorthQueryOperationAuthorityBasis {
        &self.subject_affinity
    }

    pub(crate) fn candidate(
        &self,
    ) -> Option<(
        &str,
        &crate::domain_installation::WorthQueryOperationAuthorityBasis,
    )> {
        self.candidate
            .as_ref()
            .map(|candidate| (candidate.source_identity.as_str(), &candidate.affinity))
    }

    pub(crate) fn readmits_lease(
        &self,
        source_identity: &str,
        affinity: &crate::domain_installation::WorthQueryOperationAuthorityBasis,
        closure: &crate::domain_installation::WorthQueryCompiledSemanticAspectDependencyClosure,
    ) -> bool {
        let admitted_affinity = if source_identity == self.subject_source_identity {
            Some(&self.subject_affinity)
        } else {
            self.candidate()
                .and_then(|(candidate_source, candidate_affinity)| {
                    (source_identity == candidate_source).then_some(candidate_affinity)
                })
        };
        admitted_affinity.is_some_and(|admitted| admitted == affinity)
            && closure.affinity == *affinity
            && closure.closure_evidence() == self.subject_closure_evidence
            && match &self.evidence {
                WorthQueryProjectionSharingEvidence::Singleton(evidence) => {
                    *evidence == self.subject_closure_evidence
                }
                WorthQueryProjectionSharingEvidence::Equivalent(_reuse) => true,
            }
    }
}
#[cfg(test)]
#[path = "owner_admission_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
