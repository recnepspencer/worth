use crate::{
    operation_live::WorthUiOperationLiveRetention, WorthUiExactOperationLiveResourceEvidence,
    WorthUiInstalledQueryBindingReference, WorthUiOperationLiveAdmissionDenial,
    WorthUiOperationLiveAdmissionStop, WorthUiOperationLiveResource,
    WorthUiQueryViewExecutionEvidenceDenial,
};

use super::WorthUiInstalledDownstreamQueryState;

impl WorthUiInstalledDownstreamQueryState {
    pub(crate) fn admit_operation_live(
        &mut self,
        resource: WorthUiOperationLiveResource,
    ) -> Result<(), WorthUiOperationLiveAdmissionStop> {
        let reference = resource.installed_reference().clone();
        if self.references.validate(&reference).is_err() {
            return Err(WorthUiOperationLiveAdmissionStop::new(
                WorthUiOperationLiveAdmissionDenial::ForeignInstalledReference,
                resource,
            ));
        };
        let Some(operation_live) = self.operation_live.as_mut() else {
            return Err(WorthUiOperationLiveAdmissionStop::new(
                WorthUiOperationLiveAdmissionDenial::ForeignInstalledReference,
                resource,
            ));
        };
        if operation_live.contains(&reference) {
            return Err(WorthUiOperationLiveAdmissionStop::new(
                WorthUiOperationLiveAdmissionDenial::DuplicateResource,
                resource,
            ));
        }
        operation_live.admit(resource)
    }

    pub(crate) fn refresh_operation_live(
        &mut self,
        execution: &worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
        reference: &WorthUiInstalledQueryBindingReference,
        workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    ) -> Result<crate::WorthUiOperationLiveRefreshOutcome, crate::WorthUiOperationLiveRefreshError>
    {
        self.validate_reference(reference).map_err(|_| {
            crate::WorthUiOperationLiveRefreshError::Ui(
                crate::WorthUiOperationLiveRefreshDenial::ResourceNotRetained,
            )
        })?;
        self.operation_live
            .as_mut()
            .ok_or(crate::WorthUiOperationLiveRefreshError::Ui(
                crate::WorthUiOperationLiveRefreshDenial::ResourceNotRetained,
            ))?
            .refresh(execution, reference, workspace)
    }

    pub(crate) fn admit_operation_live_change(
        &mut self,
        consequence: crate::WorthUiCollectionChangeConsequence,
    ) -> Result<
        crate::WorthUiCollectionChangeStagingReceipt,
        crate::WorthUiCollectionChangeAdmissionStop,
    > {
        if self
            .references
            .validate(consequence.installed_reference())
            .is_err()
        {
            return Err(crate::WorthUiCollectionChangeAdmissionStop::new(
                crate::WorthUiCollectionChangeAdmissionDenial::ForeignInstalledReference,
                consequence,
            ));
        }
        let Some(operation_live) = self.operation_live.as_mut() else {
            return Err(crate::WorthUiCollectionChangeAdmissionStop::new(
                crate::WorthUiCollectionChangeAdmissionDenial::ResourceNotRetained,
                consequence,
            ));
        };
        operation_live.admit_collection_change(consequence)
    }

    pub(crate) fn admit_operation_live_change_for_publication(
        &mut self,
        consequence: crate::WorthUiCollectionChangeConsequence,
    ) -> Result<
        crate::WorthUiAdmittedCollectionChangePublication,
        crate::WorthUiCollectionChangeAdmissionStop,
    > {
        if self
            .references
            .validate(consequence.installed_reference())
            .is_err()
        {
            return Err(crate::WorthUiCollectionChangeAdmissionStop::new(
                crate::WorthUiCollectionChangeAdmissionDenial::ForeignInstalledReference,
                consequence,
            ));
        }
        let Some(operation_live) = self.operation_live.as_mut() else {
            return Err(crate::WorthUiCollectionChangeAdmissionStop::new(
                crate::WorthUiCollectionChangeAdmissionDenial::ResourceNotRetained,
                consequence,
            ));
        };
        operation_live.admit_collection_change_for_publication(consequence)
    }

    pub(crate) fn publish_admitted_operation_live_change(
        &mut self,
        admission: crate::WorthUiAdmittedCollectionChangePublication,
    ) -> Result<
        crate::WorthUiCollectionChangePublicationReceipt,
        crate::WorthUiCollectionChangePublicationStop,
    > {
        if self
            .references
            .validate(admission.installed_reference())
            .is_err()
        {
            return Err(crate::WorthUiCollectionChangePublicationStop::new(
                crate::WorthUiCollectionChangePublicationDenial::ForeignInstalledReference,
                admission,
            ));
        }
        let Some(operation_live) = self.operation_live.as_mut() else {
            return Err(crate::WorthUiCollectionChangePublicationStop::new(
                crate::WorthUiCollectionChangePublicationDenial::ResourceNotRetained,
                admission,
            ));
        };
        operation_live.publish_admitted_collection_change(admission)
    }

    pub(crate) fn withdraw_admitted_operation_live_change(
        &mut self,
        admission: crate::WorthUiAdmittedCollectionChangePublication,
    ) -> Result<
        crate::WorthUiCollectionChangeConsequence,
        crate::WorthUiCollectionChangePublicationStop,
    > {
        if self
            .references
            .validate(admission.installed_reference())
            .is_err()
        {
            return Err(crate::WorthUiCollectionChangePublicationStop::new(
                crate::WorthUiCollectionChangePublicationDenial::ForeignInstalledReference,
                admission,
            ));
        }
        let Some(operation_live) = self.operation_live.as_mut() else {
            return Err(crate::WorthUiCollectionChangePublicationStop::new(
                crate::WorthUiCollectionChangePublicationDenial::ResourceNotRetained,
                admission,
            ));
        };
        operation_live.withdraw_admitted_collection_change(admission)
    }

    pub(crate) fn validate_operation_live_change_observation(
        &self,
        consequence: crate::WorthUiCollectionChangeConsequence,
    ) -> Result<
        crate::WorthUiValidatedCollectionChangeObservation,
        crate::WorthUiCollectionChangeAdmissionStop,
    > {
        if self
            .references
            .validate(consequence.installed_reference())
            .is_err()
        {
            return Err(crate::WorthUiCollectionChangeAdmissionStop::new(
                crate::WorthUiCollectionChangeAdmissionDenial::ForeignInstalledReference,
                consequence,
            ));
        }
        let Some(operation_live) = self.operation_live.as_ref() else {
            return Err(crate::WorthUiCollectionChangeAdmissionStop::new(
                crate::WorthUiCollectionChangeAdmissionDenial::ResourceNotRetained,
                consequence,
            ));
        };
        operation_live.validate_collection_change_observation(consequence)
    }

    pub(crate) fn publish_staged_operation_live_changes(
        &mut self,
    ) -> crate::WorthUiCollectionChangePublicationReceipt {
        self.operation_live.as_mut().map_or_else(
            || crate::WorthUiCollectionChangePublicationReceipt::new(0),
            WorthUiOperationLiveRetention::publish_staged,
        )
    }

    pub(crate) fn operation_live_change_observation_for(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Result<crate::WorthUiOperationLiveChangeObservation, WorthUiQueryViewExecutionEvidenceDenial>
    {
        self.validate_reference(reference)?;
        self.operation_live
            .as_ref()
            .and_then(|retention| retention.collection_change_observation(reference))
            .ok_or(WorthUiQueryViewExecutionEvidenceDenial::ProjectionNotAdmitted)
    }

    pub(crate) fn retry_operation_live_change_handoff(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Result<
        crate::WorthUiCollectionChangeConsequence,
        crate::WorthUiCollectionChangeHandoffRetryDenial,
    > {
        self.validate_reference(reference).map_err(|_| {
            crate::WorthUiCollectionChangeHandoffRetryDenial::ForeignInstalledReference
        })?;
        self.operation_live
            .as_ref()
            .ok_or(crate::WorthUiCollectionChangeHandoffRetryDenial::ResourceNotRetained)?
            .retry_collection_change_handoff(reference)
    }

    pub(crate) fn exact_operation_live_resource_evidence_for(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Result<
        Option<WorthUiExactOperationLiveResourceEvidence>,
        WorthUiQueryViewExecutionEvidenceDenial,
    > {
        self.validate_reference(reference)?;
        Ok(self
            .operation_live
            .as_ref()
            .and_then(|operation_live| operation_live.exact_evidence(reference)))
    }

    pub(crate) fn retains_operation_live_resource_for(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Result<bool, WorthUiQueryViewExecutionEvidenceDenial> {
        self.exact_operation_live_resource_evidence_for(reference)
            .map(|evidence| evidence.is_some())
    }

    pub(crate) fn take_operation_live_resource(
        &mut self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Option<WorthUiOperationLiveResource> {
        self.operation_live
            .as_mut()
            .and_then(|operation_live| operation_live.take(reference))
    }

    pub(crate) fn replace_operation_live_resource(
        &mut self,
        reference: &WorthUiInstalledQueryBindingReference,
        resource: WorthUiOperationLiveResource,
    ) -> Option<WorthUiOperationLiveResource> {
        debug_assert_eq!(resource.installed_reference(), reference);
        match self.operation_live.as_mut() {
            Some(operation_live) => operation_live.insert(resource),
            None => Some(resource),
        }
    }

    pub(crate) fn drain_operation_live_resources_into(
        &mut self,
        retirement: &mut impl Extend<WorthUiOperationLiveResource>,
    ) {
        if let Some(operation_live) = self.operation_live.as_mut() {
            operation_live.drain_into(retirement);
        }
    }

    pub(crate) fn retain_only_operation_live_resources_for(
        &mut self,
        retained: &std::collections::BTreeSet<crate::WorthUiQueryViewIdentity>,
        retirement: &mut impl Extend<WorthUiOperationLiveResource>,
    ) {
        if let Some(operation_live) = self.operation_live.as_mut() {
            operation_live.retain_only(retained, retirement);
        }
    }

    pub(crate) fn finish_operation_live_succession(
        &mut self,
        retirement: &mut impl Extend<WorthUiOperationLiveResource>,
    ) {
        if self.references.has_live_reference() {
            return;
        }
        if let Some(mut operation_live) = self.operation_live.take() {
            operation_live.drain_into(retirement);
        }
    }

    pub(crate) fn operation_live_observation(&self) -> crate::WorthUiOperationLiveObservation {
        self.operation_live
            .as_ref()
            .map_or_else(Default::default, |operation_live| {
                operation_live.observation(|reference| self.references.validate(reference).is_ok())
            })
    }

    pub(crate) fn operation_live_resource_count(&self) -> usize {
        self.operation_live
            .as_ref()
            .map_or(0, WorthUiOperationLiveRetention::resource_count)
    }

    pub(crate) fn has_staged_operation_live_changes(&self) -> bool {
        self.operation_live
            .as_ref()
            .is_some_and(WorthUiOperationLiveRetention::has_staged_changes)
    }
}
