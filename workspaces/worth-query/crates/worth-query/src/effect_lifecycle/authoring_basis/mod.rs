use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
mod identities;

use crate::basis_lifecycle::{
    evaluate_basis_effect_authoring_deferred_eligibility, normalize_raw_basis_intent,
    AdvisoryBasisEligibility, BasisAuthorityPosture, BasisFamily, BasisLifecyclePosture,
    DeferredBasisEligibility, InspectionLaneWitness, RawBasisIntent,
    ScopedMutationPreparationBasis, ScopedPreviewCloseoutBasis,
};
use crate::evidence_identity::WorthQueryEvidenceIdentity;

use self::identities::{
    advisory_basis_capability_identity, advisory_basis_scoped_basis_identity,
    deferred_basis_capability_identity, deferred_basis_scoped_basis_identity,
    expected_lower_runtime_binding_identity, mutation_preparation_capability_identity,
    mutation_preparation_scoped_basis_identity, preview_closeout_capability_identity,
    preview_closeout_scoped_basis_identity,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffectAuthoringBasis {
    MutationPreparation(ScopedMutationPreparationBasis),
    PreviewCloseout(ScopedPreviewCloseoutBasis),
    InspectionAdvisory(AdvisoryBasisEligibility<InspectionLaneWitness>),
    DeferredFutureNeighbor(
        DeferredBasisEligibility<crate::basis_lifecycle::EffectAuthoringLaneWitness>,
    ),
}

impl EffectAuthoringBasis {
    pub fn store_backed(store_basis_identity: impl Into<String>) -> Self {
        Self::deferred_future_neighbor(RawBasisIntent::StoreBacked {
            store_basis_identity: store_basis_identity.into(),
        })
    }

    pub fn durable_reload(reload_identity: impl Into<String>) -> Self {
        Self::deferred_future_neighbor(RawBasisIntent::DurableReload {
            reload_identity: reload_identity.into(),
        })
    }

    fn deferred_future_neighbor(raw: RawBasisIntent) -> Self {
        let normalized = normalize_raw_basis_intent(raw, "effect_authoring")
            .expect("future-neighbor effect basis should normalize");
        let deferred = evaluate_basis_effect_authoring_deferred_eligibility(normalized)
            .expect("future-neighbor effect basis should return deferred proof");
        Self::DeferredFutureNeighbor(deferred)
    }

    pub fn family(&self) -> BasisFamily {
        match self {
            Self::MutationPreparation(basis) => basis.family(),
            Self::PreviewCloseout(basis) => basis.family(),
            Self::InspectionAdvisory(advisory) => advisory.normalized().family(),
            Self::DeferredFutureNeighbor(deferred) => deferred.normalized().family(),
        }
    }

    pub fn authority(&self) -> BasisAuthorityPosture {
        match self {
            Self::MutationPreparation(basis) => basis.authority(),
            Self::PreviewCloseout(basis) => basis.authority(),
            Self::InspectionAdvisory(advisory) => advisory.normalized().authority(),
            Self::DeferredFutureNeighbor(deferred) => deferred.normalized().authority(),
        }
    }

    pub fn lifecycle(&self) -> BasisLifecyclePosture {
        match self {
            Self::MutationPreparation(basis) => basis.lifecycle(),
            Self::PreviewCloseout(basis) => basis.lifecycle(),
            Self::InspectionAdvisory(advisory) => advisory.normalized().lifecycle(),
            Self::DeferredFutureNeighbor(deferred) => deferred.normalized().lifecycle(),
        }
    }

    pub fn capability_for_reporting(&self) -> Result<String, IdentityDerivationDenial> {
        Ok(self.capability_identity()?.as_str().to_string())
    }

    pub fn capability_identity(
        &self,
    ) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(match self {
            Self::MutationPreparation(basis) => mutation_preparation_capability_identity(basis)?,
            Self::PreviewCloseout(basis) => preview_closeout_capability_identity(basis)?,
            Self::InspectionAdvisory(advisory) => {
                advisory_basis_capability_identity(advisory.normalized())?
            }
            Self::DeferredFutureNeighbor(deferred) => {
                deferred_basis_capability_identity(deferred.normalized())?
            }
        })
    }

    pub fn scoped_basis_for_reporting(&self) -> Result<String, IdentityDerivationDenial> {
        Ok(self.scoped_basis_identity()?.as_str().to_string())
    }

    pub fn scoped_basis_identity(
        &self,
    ) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(match self {
            Self::MutationPreparation(basis) => mutation_preparation_scoped_basis_identity(basis)?,
            Self::PreviewCloseout(basis) => preview_closeout_scoped_basis_identity(basis)?,
            Self::InspectionAdvisory(advisory) => {
                advisory_basis_scoped_basis_identity(advisory.normalized())?
            }
            Self::DeferredFutureNeighbor(deferred) => {
                deferred_basis_scoped_basis_identity(deferred.normalized())?
            }
        })
    }

    pub fn expected_lower_runtime_binding_digest(&self) -> Option<&str> {
        match self {
            Self::MutationPreparation(basis) => basis.expected_lower_runtime_binding_digest(),
            Self::PreviewCloseout(basis) => basis.expected_lower_runtime_binding_digest(),
            Self::InspectionAdvisory(advisory) => {
                advisory.normalized().lower_runtime_binding_digest()
            }
            Self::DeferredFutureNeighbor(deferred) => {
                deferred.normalized().lower_runtime_binding_digest()
            }
        }
    }

    pub fn expected_lower_runtime_binding_identity(
        &self,
    ) -> Result<Option<WorthQueryEvidenceIdentity>, IdentityDerivationDenial> {
        Ok(self
            .expected_lower_runtime_binding_digest()
            .map(expected_lower_runtime_binding_identity)
            .transpose()?)
    }

    pub(crate) fn requires_preview_workflow_binding(&self) -> bool {
        matches!(self, Self::PreviewCloseout(_))
    }
}

impl From<ScopedMutationPreparationBasis> for EffectAuthoringBasis {
    fn from(value: ScopedMutationPreparationBasis) -> Self {
        Self::MutationPreparation(value)
    }
}

impl From<ScopedPreviewCloseoutBasis> for EffectAuthoringBasis {
    fn from(value: ScopedPreviewCloseoutBasis) -> Self {
        Self::PreviewCloseout(value)
    }
}

impl From<AdvisoryBasisEligibility<InspectionLaneWitness>> for EffectAuthoringBasis {
    fn from(value: AdvisoryBasisEligibility<InspectionLaneWitness>) -> Self {
        Self::InspectionAdvisory(value)
    }
}
