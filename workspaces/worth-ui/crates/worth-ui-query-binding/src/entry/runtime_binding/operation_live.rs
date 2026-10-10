use crate::{
    WorthUiInstalledQueryBindingReference, WorthUiOperationLiveAdmissionDenial,
    WorthUiOperationLiveAdmissionStop, WorthUiOperationLiveResource,
};

use super::{WorthUiQueryViewExecutionEvidenceDenial, WorthUiRuntimeQueryBinding};

impl WorthUiRuntimeQueryBinding {
    pub fn into_operation_live_retirement(mut self) -> crate::WorthUiOperationLiveRetirement {
        let mut resources = Vec::new();
        self.drain_operation_live_resources_into(&mut resources);
        crate::WorthUiOperationLiveRetirement::new(resources)
    }

    pub fn operation_live_observation(&self) -> crate::WorthUiOperationLiveObservation {
        match self {
            Self::QueryFree => Default::default(),
            Self::Installed(binding) => binding.operation_live_observation(),
        }
    }

    pub fn admit_operation_live(
        &mut self,
        resource: WorthUiOperationLiveResource,
    ) -> Result<(), WorthUiOperationLiveAdmissionStop> {
        match self {
            Self::QueryFree => Err(WorthUiOperationLiveAdmissionStop::new(
                WorthUiOperationLiveAdmissionDenial::QueryNotInstalled,
                resource,
            )),
            Self::Installed(binding) => binding.admit_operation_live(resource),
        }
    }

    pub fn refresh_operation_live(
        &mut self,
        execution: &worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
        request: crate::WorthUiOperationLiveRefreshRequest<'_>,
    ) -> Result<crate::WorthUiOperationLiveRefreshOutcome, crate::WorthUiOperationLiveRefreshError>
    {
        let (reference, workspace) = request.into_parts();
        match self {
            Self::QueryFree => Err(crate::WorthUiOperationLiveRefreshError::Ui(
                crate::WorthUiOperationLiveRefreshDenial::ResourceNotRetained,
            )),
            Self::Installed(binding) => {
                binding.refresh_operation_live(execution, &reference, workspace)
            }
        }
    }

    pub fn admit_operation_live_change(
        &mut self,
        consequence: crate::WorthUiCollectionChangeConsequence,
    ) -> Result<
        crate::WorthUiCollectionChangeStagingReceipt,
        crate::WorthUiCollectionChangeAdmissionStop,
    > {
        match self {
            Self::QueryFree => Err(crate::WorthUiCollectionChangeAdmissionStop::new(
                crate::WorthUiCollectionChangeAdmissionDenial::QueryNotInstalled,
                consequence,
            )),
            Self::Installed(binding) => binding.admit_operation_live_change(consequence),
        }
    }

    pub fn admit_operation_live_change_for_publication(
        &mut self,
        consequence: crate::WorthUiCollectionChangeConsequence,
    ) -> Result<
        crate::WorthUiAdmittedCollectionChangePublication,
        crate::WorthUiCollectionChangeAdmissionStop,
    > {
        match self {
            Self::QueryFree => Err(crate::WorthUiCollectionChangeAdmissionStop::new(
                crate::WorthUiCollectionChangeAdmissionDenial::QueryNotInstalled,
                consequence,
            )),
            Self::Installed(binding) => {
                binding.admit_operation_live_change_for_publication(consequence)
            }
        }
    }

    pub fn publish_admitted_operation_live_change(
        &mut self,
        admission: crate::WorthUiAdmittedCollectionChangePublication,
    ) -> Result<
        crate::WorthUiCollectionChangePublicationReceipt,
        crate::WorthUiCollectionChangePublicationStop,
    > {
        match self {
            Self::QueryFree => Err(crate::WorthUiCollectionChangePublicationStop::new(
                crate::WorthUiCollectionChangePublicationDenial::QueryNotInstalled,
                admission,
            )),
            Self::Installed(binding) => binding.publish_admitted_operation_live_change(admission),
        }
    }

    pub fn withdraw_admitted_operation_live_change(
        &mut self,
        admission: crate::WorthUiAdmittedCollectionChangePublication,
    ) -> Result<
        crate::WorthUiCollectionChangeConsequence,
        crate::WorthUiCollectionChangePublicationStop,
    > {
        match self {
            Self::QueryFree => Err(crate::WorthUiCollectionChangePublicationStop::new(
                crate::WorthUiCollectionChangePublicationDenial::QueryNotInstalled,
                admission,
            )),
            Self::Installed(binding) => binding.withdraw_admitted_operation_live_change(admission),
        }
    }

    pub fn validate_operation_live_change_observation(
        &self,
        consequence: crate::WorthUiCollectionChangeConsequence,
    ) -> Result<
        crate::WorthUiValidatedCollectionChangeObservation,
        crate::WorthUiCollectionChangeAdmissionStop,
    > {
        match self {
            Self::QueryFree => Err(crate::WorthUiCollectionChangeAdmissionStop::new(
                crate::WorthUiCollectionChangeAdmissionDenial::QueryNotInstalled,
                consequence,
            )),
            Self::Installed(binding) => {
                binding.validate_operation_live_change_observation(consequence)
            }
        }
    }

    pub fn publish_staged_operation_live_changes(
        &mut self,
    ) -> crate::WorthUiCollectionChangePublicationReceipt {
        match self {
            Self::QueryFree => crate::WorthUiCollectionChangePublicationReceipt::new(0),
            Self::Installed(binding) => binding.publish_staged_operation_live_changes(),
        }
    }

    pub fn operation_live_change_observation_for(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Result<crate::WorthUiOperationLiveChangeObservation, WorthUiQueryViewExecutionEvidenceDenial>
    {
        match self {
            Self::QueryFree => Err(WorthUiQueryViewExecutionEvidenceDenial::QueryNotInstalled),
            Self::Installed(binding) => binding.operation_live_change_observation_for(reference),
        }
    }

    pub fn retry_operation_live_change_handoff(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Result<
        crate::WorthUiCollectionChangeConsequence,
        crate::WorthUiCollectionChangeHandoffRetryDenial,
    > {
        match self {
            Self::QueryFree => {
                Err(crate::WorthUiCollectionChangeHandoffRetryDenial::QueryNotInstalled)
            }
            Self::Installed(binding) => binding.retry_operation_live_change_handoff(reference),
        }
    }

    pub fn exact_operation_live_resource_evidence_for(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Result<
        Option<crate::WorthUiExactOperationLiveResourceEvidence>,
        WorthUiQueryViewExecutionEvidenceDenial,
    > {
        match self {
            Self::QueryFree => Err(WorthUiQueryViewExecutionEvidenceDenial::QueryNotInstalled),
            Self::Installed(binding) => {
                binding.exact_operation_live_resource_evidence_for(reference)
            }
        }
    }

    pub fn retains_operation_live_resource_for(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Result<bool, WorthUiQueryViewExecutionEvidenceDenial> {
        match self {
            Self::QueryFree => Err(WorthUiQueryViewExecutionEvidenceDenial::QueryNotInstalled),
            Self::Installed(binding) => binding.retains_operation_live_resource_for(reference),
        }
    }

    pub(crate) fn take_operation_live_resource(
        &mut self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Option<WorthUiOperationLiveResource> {
        match self {
            Self::QueryFree => None,
            Self::Installed(binding) => binding.take_operation_live_resource(reference),
        }
    }

    pub(crate) fn replace_operation_live_resource(
        &mut self,
        reference: &WorthUiInstalledQueryBindingReference,
        resource: WorthUiOperationLiveResource,
    ) -> Option<WorthUiOperationLiveResource> {
        let Self::Installed(binding) = self else {
            unreachable!("validated successor reference requires an installed binding")
        };
        binding.replace_operation_live_resource(reference, resource)
    }

    pub(crate) fn drain_operation_live_resources_into(
        &mut self,
        retirement: &mut impl Extend<WorthUiOperationLiveResource>,
    ) {
        if let Self::Installed(binding) = self {
            binding.drain_operation_live_resources_into(retirement);
        }
    }

    pub(crate) fn retain_only_operation_live_resources_for(
        &mut self,
        retained: &std::collections::BTreeSet<crate::WorthUiQueryViewIdentity>,
        retirement: &mut impl Extend<WorthUiOperationLiveResource>,
    ) {
        if let Self::Installed(binding) = self {
            binding.retain_only_operation_live_resources_for(retained, retirement);
        }
    }

    pub(crate) fn finish_operation_live_succession(
        &mut self,
        retirement: &mut impl Extend<WorthUiOperationLiveResource>,
    ) {
        if let Self::Installed(binding) = self {
            binding.finish_operation_live_succession(retirement);
        }
    }

    pub(crate) fn operation_live_resource_count(&self) -> usize {
        match self {
            Self::QueryFree => 0,
            Self::Installed(binding) => binding.operation_live_resource_count(),
        }
    }

    pub(crate) fn has_staged_operation_live_changes(&self) -> bool {
        match self {
            Self::QueryFree => false,
            Self::Installed(binding) => binding.has_staged_operation_live_changes(),
        }
    }
}
