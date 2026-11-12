use crate::basis::ResolvedBasisProof;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::{
    AdmittedHistoricalPathClass, HistoricalCapabilityDescriptor, HistoricalEvaluationRequest,
    HistoricalMaterializationDescriptor, HistoricalPathReuseDescriptor,
    ResolvedHistoricalPathClass,
};

fn basis(label: impl Into<String>) -> Result<ResolvedBasisProof, IdentityDerivationDenial> {
    super::bridge_historical_basis(&label.into())
}

impl HistoricalEvaluationRequest {
    pub(crate) fn retained_snapshot_for_test(
        label: impl Into<String>,
        replay_budget: usize,
        reconstruction_budget: usize,
        reuse: HistoricalPathReuseDescriptor,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::retained_snapshot(
            &basis(label)?,
            replay_budget,
            reconstruction_budget,
            reuse,
        ))
    }

    pub(crate) fn delta_replay_for_test(
        label: impl Into<String>,
        replay_budget: usize,
        reconstruction_budget: usize,
        reuse: HistoricalPathReuseDescriptor,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::delta_replay(
            &basis(label)?,
            replay_budget,
            reconstruction_budget,
            reuse,
        ))
    }

    pub(crate) fn full_reconstruction_for_test(
        label: impl Into<String>,
        replay_budget: usize,
        reconstruction_budget: usize,
        reuse: HistoricalPathReuseDescriptor,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::full_reconstruction(
            &basis(label)?,
            replay_budget,
            reconstruction_budget,
            reuse,
        ))
    }
}

impl HistoricalCapabilityDescriptor {
    pub(crate) fn retained_snapshot_for_test(
        label: impl Into<String>,
        reuse: HistoricalPathReuseDescriptor,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::retained_snapshot(&basis(label)?, reuse))
    }

    pub(crate) fn delta_replay_for_test(
        label: impl Into<String>,
        reuse: HistoricalPathReuseDescriptor,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::delta_replay(&basis(label)?, reuse))
    }

    pub(crate) fn full_reconstruction_for_test(
        label: impl Into<String>,
        reuse: HistoricalPathReuseDescriptor,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::full_reconstruction(&basis(label)?, reuse))
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_for_test(
        label: impl Into<String>,
        admitted: Option<AdmittedHistoricalPathClass>,
        replay_permitted: bool,
        replay_required: bool,
        retention_available: bool,
        historical_lookup_available: bool,
        reuse: HistoricalPathReuseDescriptor,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            basis(label)?,
            admitted,
            replay_permitted,
            replay_required,
            retention_available,
            historical_lookup_available,
            reuse,
        ))
    }
}

impl HistoricalMaterializationDescriptor {
    pub(crate) fn retained_snapshot_for_test(
        label: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::retained_snapshot(&basis(label)?))
    }

    pub(crate) fn delta_replay_for_test(
        label: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::delta_replay(&basis(label)?))
    }

    pub(crate) fn full_reconstruction_for_test(
        label: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::full_reconstruction(&basis(label)?))
    }

    pub(crate) fn new_for_test(
        label: impl Into<String>,
        resolved: ResolvedHistoricalPathClass,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(basis(label)?, resolved))
    }
}
