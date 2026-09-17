mod operation_live;
mod reference_catalog;
mod settled_snapshot;

use std::collections::BTreeMap;

use crate::operation_live::WorthUiOperationLiveRetention;
use reference_catalog::WorthUiInstalledReferenceCatalog;
use settled_snapshot::WorthUiSettledSnapshotRetention;

use crate::{
    WorthUiInstalledQueryBindingReference, WorthUiQueryViewExecutionEvidenceDenial,
    WorthUiQueryViewExecutionEvidenceReference, WorthUiQueryViewIdentity,
};

/// Downstream-only state. Query progression values cannot enter this owner;
/// each retained compatibility lifecycle has a separate physical owner.
#[derive(Debug)]
pub struct WorthUiInstalledDownstreamQueryState {
    references: WorthUiInstalledReferenceCatalog,
    scalar_projections: BTreeMap<WorthUiQueryViewIdentity, crate::UiScalarProjectionRegistration>,
    application_scalar_projections:
        BTreeMap<WorthUiQueryViewIdentity, crate::UiApplicationScalarProjectionRegistration>,
    collection_projections:
        BTreeMap<WorthUiQueryViewIdentity, crate::UiCollectionProjectionRegistration>,
    operation_live: Option<WorthUiOperationLiveRetention>,
    settled_snapshot: WorthUiSettledSnapshotRetention,
}

impl WorthUiInstalledDownstreamQueryState {
    pub(super) fn new(
        references: BTreeMap<WorthUiQueryViewIdentity, WorthUiInstalledQueryBindingReference>,
        scalar_projections: BTreeMap<
            WorthUiQueryViewIdentity,
            crate::UiScalarProjectionRegistration,
        >,
        application_scalar_projections: BTreeMap<
            WorthUiQueryViewIdentity,
            crate::UiApplicationScalarProjectionRegistration,
        >,
        collection_projections: BTreeMap<
            WorthUiQueryViewIdentity,
            crate::UiCollectionProjectionRegistration,
        >,
    ) -> Self {
        let references = WorthUiInstalledReferenceCatalog::new(references);
        let has_live_reference = references.has_live_reference();
        let operation_live = has_live_reference
            .then(|| WorthUiOperationLiveRetention::for_identities(references.live_identities()));
        let settled_snapshot =
            WorthUiSettledSnapshotRetention::for_identities(references.identities());
        Self {
            references,
            scalar_projections,
            application_scalar_projections,
            collection_projections,
            operation_live,
            settled_snapshot,
        }
    }

    pub(super) fn scalar_projection_registration(
        &self,
        identity: &WorthUiQueryViewIdentity,
    ) -> Option<&crate::UiScalarProjectionRegistration> {
        self.scalar_projections.get(identity)
    }

    pub(super) fn application_scalar_projection_registration(
        &self,
        identity: &WorthUiQueryViewIdentity,
    ) -> Option<&crate::UiApplicationScalarProjectionRegistration> {
        self.application_scalar_projections.get(identity)
    }

    pub(super) fn collection_projection_registration(
        &self,
        identity: &WorthUiQueryViewIdentity,
    ) -> Option<&crate::UiCollectionProjectionRegistration> {
        self.collection_projections.get(identity)
    }

    pub(super) fn admit_settled_snapshot(
        &mut self,
        projection: crate::WorthUiSettledSnapshotProjection,
    ) -> Result<
        std::sync::Arc<crate::WorthUiSettledSnapshotFact>,
        crate::WorthUiSettledSnapshotAdmissionStop,
    > {
        let reference = projection.installed_reference().clone();
        if self.references.validate(&reference).is_err() {
            return Err(crate::WorthUiSettledSnapshotAdmissionStop::new(
                crate::WorthUiSettledSnapshotAdmissionDenial::ForeignInstalledReference,
                projection,
            ));
        }
        self.settled_snapshot.admit(projection, &reference)
    }

    pub(super) fn refresh_settled_snapshot(
        &mut self,
        projection: crate::WorthUiSettledSnapshotProjection,
    ) -> Result<
        std::sync::Arc<crate::WorthUiSettledSnapshotFact>,
        crate::WorthUiSettledSnapshotAdmissionStop,
    > {
        let reference = projection.installed_reference().clone();
        if self.references.validate(&reference).is_err() {
            return Err(crate::WorthUiSettledSnapshotAdmissionStop::new(
                crate::WorthUiSettledSnapshotAdmissionDenial::ForeignInstalledReference,
                projection,
            ));
        }
        self.settled_snapshot.refresh(projection, &reference)
    }

    pub(super) fn settled_snapshot_fact_for(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Result<&crate::WorthUiSettledSnapshotFact, WorthUiQueryViewExecutionEvidenceDenial> {
        self.validate_reference(reference)?;
        self.settled_snapshot
            .fact_for(reference)
            .ok_or(WorthUiQueryViewExecutionEvidenceDenial::ProjectionNotAdmitted)
    }

    pub(super) fn settled_snapshot_fact_reference_for(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Result<
        std::sync::Arc<crate::WorthUiSettledSnapshotFact>,
        WorthUiQueryViewExecutionEvidenceDenial,
    > {
        self.validate_reference(reference)?;
        self.settled_snapshot
            .shared_fact_for(reference)
            .ok_or(WorthUiQueryViewExecutionEvidenceDenial::ProjectionNotAdmitted)
    }

    pub(super) fn settlement_touch_reference_for(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Result<
        crate::WorthUiAdmittedQuerySettlementTouchReference,
        WorthUiQueryViewExecutionEvidenceDenial,
    > {
        let fact = self.settled_snapshot_fact_for(reference)?;
        Ok(crate::WorthUiAdmittedQuerySettlementTouchReference::mint(
            fact,
        ))
    }

    pub(super) fn readmits_settlement_touch_reference(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
        touch: &crate::WorthUiAdmittedQuerySettlementTouchReference,
    ) -> Result<bool, WorthUiQueryViewExecutionEvidenceDenial> {
        let fact = self.settled_snapshot_fact_for(reference)?;
        Ok(fact.binding_reference() == touch.binding_reference()
            && fact.settlement_reference() == touch.settlement_reference())
    }

    pub(super) fn exact_settled_snapshot_evidence_for(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Result<
        Option<crate::WorthUiExactSettledSnapshotEvidence>,
        WorthUiQueryViewExecutionEvidenceDenial,
    > {
        self.validate_reference(reference)?;
        Ok(self.settled_snapshot.exact_evidence_for(reference))
    }

    pub(super) fn validate_reference(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Result<(), WorthUiQueryViewExecutionEvidenceDenial> {
        self.references.validate(reference)
    }

    pub(super) fn installation_is_current(&self) -> bool {
        self.references.installation_is_current()
    }

    pub(super) fn installed_references(
        &self,
    ) -> impl ExactSizeIterator<Item = WorthUiInstalledQueryBindingReference> + '_ {
        self.references.references()
    }

    pub(super) fn execution_evidence_for(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Result<WorthUiQueryViewExecutionEvidenceReference, WorthUiQueryViewExecutionEvidenceDenial>
    {
        self.validate_reference(reference)?;
        if let Some(fact) = self.settled_snapshot.shared_fact_for(reference) {
            return Ok(
                WorthUiQueryViewExecutionEvidenceReference::from_settled_snapshot(
                    reference.clone(),
                    fact,
                ),
            );
        }
        let resource = self
            .operation_live
            .as_ref()
            .ok_or(WorthUiQueryViewExecutionEvidenceDenial::ProjectionNotAdmitted)?
            .resource(reference)
            .ok_or(WorthUiQueryViewExecutionEvidenceDenial::ProjectionNotAdmitted)?;
        Ok(
            WorthUiQueryViewExecutionEvidenceReference::from_settled_snapshot(
                reference.clone(),
                resource.shared_fact(),
            ),
        )
    }

    pub(super) fn frame_evidence_for(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Result<crate::WorthUiQueryFrameEvidence, WorthUiQueryViewExecutionEvidenceDenial> {
        self.validate_reference(reference)?;
        if let Some(fact) = self.settled_snapshot.fact_for(reference) {
            return Ok(crate::WorthUiQueryFrameEvidence::from_settled_snapshot(
                reference, fact,
            ));
        }
        let resource = self
            .operation_live
            .as_ref()
            .ok_or(WorthUiQueryViewExecutionEvidenceDenial::ProjectionNotAdmitted)?
            .resource(reference)
            .ok_or(WorthUiQueryViewExecutionEvidenceDenial::ProjectionNotAdmitted)?;
        Ok(crate::WorthUiQueryFrameEvidence::from_settled_snapshot(
            reference,
            resource.fact(),
        ))
    }

    pub(super) fn retain_only_settlements_for(
        &mut self,
        retained: &std::collections::BTreeSet<WorthUiQueryViewIdentity>,
    ) {
        self.settled_snapshot.retain_only(retained);
    }

    pub(super) fn take_settled_snapshot(
        &mut self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Option<crate::WorthUiSettledSnapshotProjection> {
        self.settled_snapshot.take(reference)
    }

    pub(super) fn replace_settled_snapshot(
        &mut self,
        projection: crate::WorthUiSettledSnapshotProjection,
    ) {
        self.settled_snapshot.replace(projection);
    }

    pub(super) fn query_state_observation(&self) -> crate::WorthUiRuntimeQueryStateObservation {
        let (settled_snapshot_count, orphan_settled_snapshot_count) = self
            .settled_snapshot
            .observation_counts(|reference| self.references.validate(reference).is_ok());
        crate::WorthUiRuntimeQueryStateObservation::installed(
            self.references.len(),
            self.references.stale_reference_count(),
            settled_snapshot_count,
            orphan_settled_snapshot_count,
            self.operation_live_observation(),
        )
    }
}
