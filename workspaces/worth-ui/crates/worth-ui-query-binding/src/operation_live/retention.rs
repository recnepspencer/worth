use std::collections::{BTreeMap, BTreeSet};

use crate::{
    WorthUiAdmittedQueryBindingReference, WorthUiAdmittedQuerySettlementReference,
    WorthUiInstalledQueryBindingReference, WorthUiOperationLiveAdmissionDenial,
    WorthUiOperationLiveAdmissionStop, WorthUiOperationLiveResource, WorthUiQueryViewIdentity,
    WorthUiSettledSnapshotSourceGeneration, WorthUiSettledSnapshotSourceOrder,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthUiOperationLiveObservation {
    subsystem_construction_count: usize,
    retained_resource_count: usize,
    orphan_resource_count: usize,
    resource_registration_count: usize,
    succession_operation_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiExactOperationLiveResourceEvidence {
    installed_reference: WorthUiInstalledQueryBindingReference,
    binding_reference: WorthUiAdmittedQueryBindingReference,
    settlement_reference: WorthUiAdmittedQuerySettlementReference,
}

#[derive(Default)]
pub(crate) struct WorthUiOperationLiveRetention {
    resources: BTreeMap<WorthUiQueryViewIdentity, Option<WorthUiOperationLiveResource>>,
    resource_registration_count: usize,
    succession_operation_count: usize,
    next_source_generation: u64,
    next_source_order: u64,
    staged_sources: BTreeSet<WorthUiQueryViewIdentity>,
}

impl std::fmt::Debug for WorthUiOperationLiveRetention {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthUiOperationLiveRetention")
            .field(
                "retained_resource_count",
                &self.resources.values().flatten().count(),
            )
            .field(
                "resource_registration_count",
                &self.resource_registration_count,
            )
            .field(
                "succession_operation_count",
                &self.succession_operation_count,
            )
            .finish()
    }
}

impl WorthUiOperationLiveRetention {
    pub(crate) fn for_identities(
        identities: impl IntoIterator<Item = WorthUiQueryViewIdentity>,
    ) -> Self {
        Self {
            resources: identities
                .into_iter()
                .map(|identity| (identity, None))
                .collect(),
            ..Self::default()
        }
    }

    pub(crate) fn contains(&self, reference: &WorthUiInstalledQueryBindingReference) -> bool {
        self.resources
            .get(reference.definition().identity())
            .is_some_and(Option::is_some)
    }

    pub(crate) fn resource_count(&self) -> usize {
        self.resources.values().flatten().count()
    }

    pub(crate) fn has_staged_changes(&self) -> bool {
        !self.staged_sources.is_empty()
    }

    pub(crate) fn insert(
        &mut self,
        resource: WorthUiOperationLiveResource,
    ) -> Option<WorthUiOperationLiveResource> {
        let identity = resource.definition().identity();
        let slot = self
            .resources
            .get_mut(identity)
            .expect("installed live references have reserved resource slots");
        let replaced = slot.replace(resource);
        if replaced.is_none() {
            self.resource_registration_count += 1;
        }
        replaced
    }

    pub(crate) fn admit(
        &mut self,
        mut resource: WorthUiOperationLiveResource,
    ) -> Result<(), WorthUiOperationLiveAdmissionStop> {
        let generation = match self.next_source_generation.checked_add(1) {
            Some(value) => value,
            None => {
                return Err(WorthUiOperationLiveAdmissionStop::new(
                    WorthUiOperationLiveAdmissionDenial::SourceGenerationExhausted,
                    resource,
                ));
            }
        };
        let order = match self.next_source_order.checked_add(1) {
            Some(value) => value,
            None => {
                return Err(WorthUiOperationLiveAdmissionStop::new(
                    WorthUiOperationLiveAdmissionDenial::SourceOrderExhausted,
                    resource,
                ));
            }
        };
        resource.attach_source_coordinates(
            WorthUiSettledSnapshotSourceGeneration::new(generation),
            WorthUiSettledSnapshotSourceOrder::new(order),
        );
        self.next_source_generation = generation;
        self.next_source_order = order;
        let displaced = self.insert(resource);
        debug_assert!(
            displaced.is_none(),
            "admission rejects a duplicate before retention inserts it"
        );
        Ok(())
    }

    pub(crate) fn refresh(
        &mut self,
        reference: &WorthUiInstalledQueryBindingReference,
        workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    ) -> Result<crate::WorthUiOperationLiveRefreshOutcome, crate::WorthUiOperationLiveRefreshError>
    {
        let resource = self
            .resources
            .get_mut(reference.definition().identity())
            .and_then(Option::as_mut)
            .ok_or(crate::WorthUiOperationLiveRefreshError::Ui(
                crate::WorthUiOperationLiveRefreshDenial::ResourceNotRetained,
            ))?;
        resource.refresh(workspace)
    }

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

    pub(crate) fn publish_staged(&mut self) -> crate::WorthUiCollectionChangePublicationReceipt {
        let staged = std::mem::take(&mut self.staged_sources);
        let mut published = 0;
        for identity in staged {
            let Some(resource) = self.resources.get_mut(&identity).and_then(Option::as_mut) else {
                continue;
            };
            published += usize::from(resource.publish_staged_collection_change());
        }
        crate::WorthUiCollectionChangePublicationReceipt::new(published)
    }

    pub(crate) fn exact_evidence(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Option<WorthUiExactOperationLiveResourceEvidence> {
        self.resources
            .get(reference.definition().identity())
            .and_then(Option::as_ref)
            .map(|resource| WorthUiExactOperationLiveResourceEvidence {
                installed_reference: resource.installed_reference().clone(),
                binding_reference: resource.fact().binding_reference().clone(),
                settlement_reference: resource.fact().settlement_reference().clone(),
            })
    }

    pub(crate) fn resource(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Option<&WorthUiOperationLiveResource> {
        self.resources
            .get(reference.definition().identity())
            .and_then(Option::as_ref)
    }

    pub(crate) fn collection_change_observation(
        &self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Option<crate::WorthUiOperationLiveChangeObservation> {
        self.resource(reference)
            .map(WorthUiOperationLiveResource::collection_change_observation)
    }

    pub(crate) fn take(
        &mut self,
        reference: &WorthUiInstalledQueryBindingReference,
    ) -> Option<WorthUiOperationLiveResource> {
        self.succession_operation_count += 1;
        self.resources
            .get_mut(reference.definition().identity())
            .and_then(Option::take)
    }

    pub(crate) fn drain_into(
        &mut self,
        retirement: &mut impl Extend<WorthUiOperationLiveResource>,
    ) {
        self.succession_operation_count += 1;
        self.staged_sources.clear();
        retirement.extend(self.resources.values_mut().filter_map(Option::take));
    }

    pub(crate) fn retain_only(
        &mut self,
        retained: &BTreeSet<WorthUiQueryViewIdentity>,
        retirement: &mut impl Extend<WorthUiOperationLiveResource>,
    ) {
        self.succession_operation_count += 1;
        self.staged_sources
            .retain(|identity| retained.contains(identity));
        for (identity, resource) in &mut self.resources {
            if !retained.contains(identity) {
                if let Some(resource) = resource.take() {
                    retirement.extend(std::iter::once(resource));
                }
            }
        }
    }

    pub(crate) fn observation(
        &self,
        reference_is_valid: impl Fn(&WorthUiInstalledQueryBindingReference) -> bool,
    ) -> WorthUiOperationLiveObservation {
        WorthUiOperationLiveObservation {
            subsystem_construction_count: 1,
            retained_resource_count: self.resources.values().flatten().count(),
            orphan_resource_count: self
                .resources
                .values()
                .flatten()
                .filter(|resource| !reference_is_valid(resource.installed_reference()))
                .count(),
            resource_registration_count: self.resource_registration_count,
            succession_operation_count: self.succession_operation_count,
        }
    }
}

impl WorthUiOperationLiveObservation {
    pub fn subsystem_construction_count(self) -> usize {
        self.subsystem_construction_count
    }

    pub fn retained_resource_count(self) -> usize {
        self.retained_resource_count
    }

    pub fn orphan_resource_count(self) -> usize {
        self.orphan_resource_count
    }

    pub fn resource_registration_count(self) -> usize {
        self.resource_registration_count
    }

    pub fn succession_operation_count(self) -> usize {
        self.succession_operation_count
    }
}

impl WorthUiExactOperationLiveResourceEvidence {
    pub fn installed_reference(&self) -> &WorthUiInstalledQueryBindingReference {
        &self.installed_reference
    }

    pub fn binding_reference(&self) -> &WorthUiAdmittedQueryBindingReference {
        &self.binding_reference
    }

    pub fn settlement_reference(&self) -> &WorthUiAdmittedQuerySettlementReference {
        &self.settlement_reference
    }
}
