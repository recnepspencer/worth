//! Staging, publishing and withdrawing one retained resource's collection
//! change. Every entry names the resource first, so a change aimed at a
//! resource this retention no longer holds is refused rather than staged.
use super::WorthUiOperationLiveRetention;
use crate::WorthUiInstalledQueryBindingReference;

impl WorthUiOperationLiveRetention {
    pub(crate) fn admit_collection_change(
        &mut self,
        consequence: crate::WorthUiCollectionChangeConsequence,
    ) -> Result<
        crate::WorthUiCollectionChangeStagingReceipt,
        crate::WorthUiCollectionChangeAdmissionStop,
    > {
        let identity = consequence
            .installed_reference()
            .definition()
            .identity()
            .clone();
        let Some(resource) = self.resources.get_mut(&identity).and_then(Option::as_mut) else {
            return Err(crate::WorthUiCollectionChangeAdmissionStop::new(
                crate::WorthUiCollectionChangeAdmissionDenial::ResourceNotRetained,
                consequence,
            ));
        };
        let receipt = resource.admit_collection_change(consequence)?;
        self.staged_sources.insert(identity);
        Ok(receipt)
    }

    pub(crate) fn admit_collection_change_for_publication(
        &mut self,
        consequence: crate::WorthUiCollectionChangeConsequence,
    ) -> Result<
        crate::WorthUiAdmittedCollectionChangePublication,
        crate::WorthUiCollectionChangeAdmissionStop,
    > {
        let identity = consequence
            .installed_reference()
            .definition()
            .identity()
            .clone();
        let Some(resource) = self.resources.get_mut(&identity).and_then(Option::as_mut) else {
            return Err(crate::WorthUiCollectionChangeAdmissionStop::new(
                crate::WorthUiCollectionChangeAdmissionDenial::ResourceNotRetained,
                consequence,
            ));
        };
        let admission = resource.admit_collection_change_for_publication(consequence)?;
        self.staged_sources.insert(identity);
        Ok(admission)
    }

    pub(crate) fn publish_admitted_collection_change(
        &mut self,
        admission: crate::WorthUiAdmittedCollectionChangePublication,
    ) -> Result<
        crate::WorthUiCollectionChangePublicationReceipt,
        crate::WorthUiCollectionChangePublicationStop,
    > {
        let identity = admission
            .installed_reference()
            .definition()
            .identity()
            .clone();
        let Some(resource) = self.resources.get_mut(&identity).and_then(Option::as_mut) else {
            return Err(crate::WorthUiCollectionChangePublicationStop::new(
                crate::WorthUiCollectionChangePublicationDenial::ResourceNotRetained,
                admission,
            ));
        };
        resource.publish_admitted_collection_change(admission)?;
        self.staged_sources.remove(&identity);
        Ok(crate::WorthUiCollectionChangePublicationReceipt::new(1))
    }

    pub(crate) fn withdraw_admitted_collection_change(
        &mut self,
        admission: crate::WorthUiAdmittedCollectionChangePublication,
    ) -> Result<
        crate::WorthUiCollectionChangeConsequence,
        crate::WorthUiCollectionChangePublicationStop,
    > {
        let identity = admission
            .installed_reference()
            .definition()
            .identity()
            .clone();
        let Some(resource) = self.resources.get_mut(&identity).and_then(Option::as_mut) else {
            return Err(crate::WorthUiCollectionChangePublicationStop::new(
                crate::WorthUiCollectionChangePublicationDenial::ResourceNotRetained,
                admission,
            ));
        };
        let consequence = resource.withdraw_admitted_collection_change(admission)?;
        self.staged_sources.remove(&identity);
        Ok(consequence)
    }

    pub(crate) fn validate_collection_change_observation(
        &self,
        consequence: crate::WorthUiCollectionChangeConsequence,
    ) -> Result<
        crate::WorthUiValidatedCollectionChangeObservation,
        crate::WorthUiCollectionChangeAdmissionStop,
    > {
        let identity = consequence.installed_reference().definition().identity();
        let Some(resource) = self.resources.get(identity).and_then(Option::as_ref) else {
            return Err(crate::WorthUiCollectionChangeAdmissionStop::new(
                crate::WorthUiCollectionChangeAdmissionDenial::ResourceNotRetained,
                consequence,
            ));
        };
        resource.validate_collection_change_observation(consequence)
    }

    pub(crate) fn retry_collection_change_handoff(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Result<
        crate::WorthUiCollectionChangeConsequence,
        crate::WorthUiCollectionChangeHandoffRetryDenial,
    > {
        self.resource(reference)
            .ok_or(crate::WorthUiCollectionChangeHandoffRetryDenial::ResourceNotRetained)?
            .retry_collection_change_handoff()
    }
}
