use crate::{WorthUiInstalledQueryBindingReference, WorthUiQueryViewExecutionEvidenceReference};

use super::WorthUiInstalledDownstreamQueryState;

mod operation_live;

/// Runtime binding posture. The installed variant owns downstream UI state,
/// never a Query operating world or move-only Query progression value.
#[derive(Debug)]
pub enum WorthUiRuntimeQueryBinding {
    QueryFree,
    Installed(Box<WorthUiInstalledDownstreamQueryState>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiQueryViewExecutionEvidenceDenial {
    QueryNotInstalled,
    ForeignInstalledReference,
    ProjectionNotAdmitted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiSettledSnapshotAdmissionDenial {
    QueryNotInstalled,
    ForeignInstalledReference,
    DuplicateSettlement,
    MissingPredecessorSettlement,
    SourceGenerationExhausted,
    SourceOrderExhausted,
}

pub struct WorthUiSettledSnapshotAdmissionStop {
    denial: WorthUiSettledSnapshotAdmissionDenial,
    projection: Box<crate::WorthUiSettledSnapshotProjection>,
}

impl std::fmt::Debug for WorthUiSettledSnapshotAdmissionStop {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthUiSettledSnapshotAdmissionStop")
            .field("denial", &self.denial)
            .field("projection", &"returned exact Query projection")
            .finish()
    }
}

impl WorthUiSettledSnapshotAdmissionStop {
    pub(crate) fn new(
        denial: WorthUiSettledSnapshotAdmissionDenial,
        projection: crate::WorthUiSettledSnapshotProjection,
    ) -> Self {
        Self {
            denial,
            projection: Box::new(projection),
        }
    }

    pub fn denial(&self) -> WorthUiSettledSnapshotAdmissionDenial {
        self.denial
    }
    pub fn into_projection(self) -> crate::WorthUiSettledSnapshotProjection {
        *self.projection
    }
}

impl WorthUiRuntimeQueryBinding {
    pub fn scalar_projection_registration(
        &self,
        identity: &crate::WorthUiQueryViewIdentity,
    ) -> Option<&crate::UiScalarProjectionRegistration> {
        match self {
            Self::QueryFree => None,
            Self::Installed(binding) => binding.scalar_projection_registration(identity),
        }
    }

    pub fn application_scalar_projection_registration(
        &self,
        identity: &crate::WorthUiQueryViewIdentity,
    ) -> Option<&crate::UiApplicationScalarProjectionRegistration> {
        match self {
            Self::QueryFree => None,
            Self::Installed(binding) => {
                binding.application_scalar_projection_registration(identity)
            }
        }
    }

    pub fn collection_projection_registration(
        &self,
        identity: &crate::WorthUiQueryViewIdentity,
    ) -> Option<&crate::UiCollectionProjectionRegistration> {
        match self {
            Self::QueryFree => None,
            Self::Installed(binding) => binding.collection_projection_registration(identity),
        }
    }

    pub fn state_observation(&self) -> crate::WorthUiRuntimeQueryStateObservation {
        match self {
            Self::QueryFree => Default::default(),
            Self::Installed(binding) => binding.query_state_observation(),
        }
    }

    pub fn reference_membership_observation(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> crate::WorthUiQueryReferenceMembershipObservation {
        match self {
            Self::QueryFree => crate::WorthUiQueryReferenceMembershipObservation::QueryFree,
            Self::Installed(binding) if binding.validate_reference(reference).is_ok() => {
                crate::WorthUiQueryReferenceMembershipObservation::ExactInstalledReference
            }
            Self::Installed(_) => {
                crate::WorthUiQueryReferenceMembershipObservation::ForeignInstalledReference
            }
        }
    }

    pub fn admit_settled_snapshot(
        &mut self,
        projection: crate::WorthUiSettledSnapshotProjection,
    ) -> Result<
        std::sync::Arc<crate::WorthUiSettledSnapshotFact>,
        WorthUiSettledSnapshotAdmissionStop,
    > {
        match self {
            Self::QueryFree => Err(WorthUiSettledSnapshotAdmissionStop::new(
                WorthUiSettledSnapshotAdmissionDenial::QueryNotInstalled,
                projection,
            )),
            Self::Installed(binding) => binding.admit_settled_snapshot(projection),
        }
    }

    pub fn refresh_settled_snapshot(
        &mut self,
        projection: crate::WorthUiSettledSnapshotProjection,
    ) -> Result<
        std::sync::Arc<crate::WorthUiSettledSnapshotFact>,
        WorthUiSettledSnapshotAdmissionStop,
    > {
        match self {
            Self::QueryFree => Err(WorthUiSettledSnapshotAdmissionStop::new(
                WorthUiSettledSnapshotAdmissionDenial::QueryNotInstalled,
                projection,
            )),
            Self::Installed(binding) => binding.refresh_settled_snapshot(projection),
        }
    }

    pub fn settled_snapshot_fact_for(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Result<&crate::WorthUiSettledSnapshotFact, WorthUiQueryViewExecutionEvidenceDenial> {
        match self {
            Self::QueryFree => Err(WorthUiQueryViewExecutionEvidenceDenial::QueryNotInstalled),
            Self::Installed(binding) => binding.settled_snapshot_fact_for(reference),
        }
    }

    /// Returns a shallow reference to the exact UI fact retained with the
    /// binding-owned Query projection. Cloning this reference never clones
    /// native facts or Query proof state.
    pub fn settled_snapshot_fact_reference_for(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Result<
        std::sync::Arc<crate::WorthUiSettledSnapshotFact>,
        WorthUiQueryViewExecutionEvidenceDenial,
    > {
        match self {
            Self::QueryFree => Err(WorthUiQueryViewExecutionEvidenceDenial::QueryNotInstalled),
            Self::Installed(binding) => binding.settled_snapshot_fact_reference_for(reference),
        }
    }

    pub fn readmit_settled_snapshot_fact<'a>(
        &self,
        fact: &'a crate::WorthUiSettledSnapshotFact,
    ) -> Result<
        crate::WorthUiReadmittedSettledSnapshotFact<'a>,
        crate::WorthUiSettledSnapshotReadmissionDenial,
    > {
        let current = self
            .settled_snapshot_fact_for(fact.binding_reference().installed_reference())
            .map_err(|denial| match denial {
                WorthUiQueryViewExecutionEvidenceDenial::QueryNotInstalled => {
                    crate::WorthUiSettledSnapshotReadmissionDenial::QueryNotInstalled
                }
                WorthUiQueryViewExecutionEvidenceDenial::ForeignInstalledReference => {
                    crate::WorthUiSettledSnapshotReadmissionDenial::ForeignInstalledReference
                }
                WorthUiQueryViewExecutionEvidenceDenial::ProjectionNotAdmitted => {
                    crate::WorthUiSettledSnapshotReadmissionDenial::ProjectionNotAdmitted
                }
            })?;
        if current.settlement_reference() != fact.settlement_reference() {
            return Err(crate::WorthUiSettledSnapshotReadmissionDenial::StaleSettlementReference);
        }
        Ok(crate::WorthUiReadmittedSettledSnapshotFact::new(fact))
    }

    pub fn settlement_touch_reference_for(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Result<
        crate::WorthUiAdmittedQuerySettlementTouchReference,
        WorthUiQueryViewExecutionEvidenceDenial,
    > {
        match self {
            Self::QueryFree => Err(WorthUiQueryViewExecutionEvidenceDenial::QueryNotInstalled),
            Self::Installed(binding) => binding.settlement_touch_reference_for(reference),
        }
    }

    pub fn readmits_settlement_touch_reference(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
        touch: &crate::WorthUiAdmittedQuerySettlementTouchReference,
    ) -> Result<bool, WorthUiQueryViewExecutionEvidenceDenial> {
        match self {
            Self::QueryFree => Err(WorthUiQueryViewExecutionEvidenceDenial::QueryNotInstalled),
            Self::Installed(binding) => {
                binding.readmits_settlement_touch_reference(reference, touch)
            }
        }
    }

    pub fn exact_settled_snapshot_evidence_for(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Result<
        Option<crate::WorthUiExactSettledSnapshotEvidence>,
        WorthUiQueryViewExecutionEvidenceDenial,
    > {
        match self {
            Self::QueryFree => Err(WorthUiQueryViewExecutionEvidenceDenial::QueryNotInstalled),
            Self::Installed(binding) => binding.exact_settled_snapshot_evidence_for(reference),
        }
    }

    pub(crate) fn admits_reference(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> bool {
        match self {
            Self::QueryFree => false,
            Self::Installed(binding) => binding.validate_reference(reference).is_ok(),
        }
    }

    pub(crate) fn installation_is_current(&self) -> bool {
        match self {
            Self::QueryFree => true,
            Self::Installed(binding) => binding.installation_is_current(),
        }
    }

    pub(crate) fn installed_references(&self) -> Vec<WorthUiInstalledQueryBindingReference> {
        match self {
            Self::QueryFree => Vec::new(),
            Self::Installed(binding) => binding.installed_references().collect(),
        }
    }

    pub fn execution_evidence_for(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Result<WorthUiQueryViewExecutionEvidenceReference, WorthUiQueryViewExecutionEvidenceDenial>
    {
        match self {
            Self::QueryFree => Err(WorthUiQueryViewExecutionEvidenceDenial::QueryNotInstalled),
            Self::Installed(binding) => binding.execution_evidence_for(reference),
        }
    }

    pub fn frame_evidence_for(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Result<crate::WorthUiQueryFrameEvidence, WorthUiQueryViewExecutionEvidenceDenial> {
        match self {
            Self::QueryFree => Err(WorthUiQueryViewExecutionEvidenceDenial::QueryNotInstalled),
            Self::Installed(binding) => binding.frame_evidence_for(reference),
        }
    }

    pub(crate) fn take_settled_snapshot(
        &mut self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Option<crate::WorthUiSettledSnapshotProjection> {
        match self {
            Self::QueryFree => None,
            Self::Installed(binding) => binding.take_settled_snapshot(reference),
        }
    }

    pub(crate) fn replace_settled_snapshot(
        &mut self,
        projection: crate::WorthUiSettledSnapshotProjection,
    ) {
        let Self::Installed(binding) = self else {
            unreachable!("validated exact Query settlement requires an installed binding")
        };
        binding.replace_settled_snapshot(projection);
    }

    pub(crate) fn retain_only_settlements_for(
        &mut self,
        retained: &std::collections::BTreeSet<crate::WorthUiQueryViewIdentity>,
    ) {
        if let Self::Installed(binding) = self {
            binding.retain_only_settlements_for(retained);
        }
    }
}
