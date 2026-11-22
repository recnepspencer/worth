//! Owner-retained authority for reading one exact World product occurrence.

use std::sync::Arc;
use worth_query_execution_runtime::facade::ports::ProductObservationLeaseInternalPort as _;

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use crate::basis::WorthQueryProductObservationLease;

/// Owner-retained authority for reading one exact World product occurrence.
///
/// This value carries no publication source and cannot authorize mutation.
pub struct WorthQueryApplicationReadObservation {
    runtime_authority: u64,
    schema_binding: ApplicationSchemaBindingIdentity,
    product: WorthQueryProductObservationLease,
    producer:
        Option<Arc<super::super::application_output_demand::OriginalSettledProducerObservation>>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl WorthQueryApplicationReadObservation {
    pub(crate) fn from_parts(
        runtime_authority: u64,
        schema_binding: ApplicationSchemaBindingIdentity,
        product: WorthQueryProductObservationLease,
        producer: Option<
            Arc<super::super::application_output_demand::OriginalSettledProducerObservation>,
        >,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Arc<Self> {
        Arc::new(Self {
            runtime_authority,
            schema_binding,
            product,
            producer,
            _memory: memory,
        })
    }

    pub(crate) fn runtime_authority(&self) -> u64 {
        self.runtime_authority
    }

    pub(crate) fn schema_binding(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema_binding
    }

    pub(crate) fn producer_arc(
        &self,
    ) -> Option<Arc<super::super::application_output_demand::OriginalSettledProducerObservation>>
    {
        self.producer.clone()
    }

    pub(crate) fn with_original_producer(
        &self,
        producer: Arc<super::super::application_output_demand::OriginalSettledProducerObservation>,
    ) -> Arc<Self> {
        Arc::new(Self {
            runtime_authority: self.runtime_authority,
            schema_binding: self.schema_binding.clone(),
            product: self.product.retained_clone(),
            producer: Some(producer),
            _memory: None,
        })
    }

    pub(crate) fn original_settled_producer(
        &self,
    ) -> Option<&super::super::application_output_demand::OriginalSettledProducerObservation> {
        self.producer.as_deref()
    }

    pub(crate) fn retained_product_lease(&self) -> WorthQueryProductObservationLease {
        self.product.retained_clone()
    }

    pub(crate) fn product_observation(
        &self,
    ) -> &worth_runtime_world::facade::ProductBranchObservation {
        self.product.observation()
    }

    pub(crate) fn same_application_occurrence(&self, other: &Self) -> bool {
        self.runtime_authority == other.runtime_authority
            && self.schema_binding == other.schema_binding
            && self.branch_identity() == other.branch_identity()
            && self.branch_incarnation() == other.branch_incarnation()
    }

    pub fn branch_identity(&self) -> &worth_runtime_world::facade::ProductBranchIdentity {
        self.product.branch_identity()
    }

    pub fn branch_incarnation(&self) -> worth_runtime_world::facade::ProductBranchIncarnation {
        self.product.observation().lifecycle_incarnation()
    }

    pub fn selected_commit(&self) -> &worth_runtime_world::facade::CompositeCommitIdentity {
        self.product.selected_commit()
    }

    pub(crate) fn matches_occurrence(
        &self,
        runtime_authority: u64,
        schema_binding: &ApplicationSchemaBindingIdentity,
        selected: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> bool {
        self.runtime_authority == runtime_authority
            && &self.schema_binding == schema_binding
            && self.product.observation().branch_identity() == selected.branch_identity()
            && self.product.observation().lifecycle_incarnation()
                == selected.lifecycle_incarnation()
    }
}

mod court_doors_for_test;
