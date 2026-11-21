//! Crate-boundary port for internal `WorthQueryApplicationReadObservation`
//! accessors and construction. Never exported on `worth-query-execution::facade`
//! or Host.

use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_runtime_world::facade::ProductBranchObservation;

use crate::basis::WorthQueryProductObservationLease;
use crate::domain_computation::primary_graph::application_output_demand::OriginalSettledProducerObservation;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationReadObservation;

pub trait ReadObservationInternalPort: Sized {
    fn from_parts(
        runtime_authority: u64,
        schema_binding: ApplicationSchemaBindingIdentity,
        product: WorthQueryProductObservationLease,
        producer: Option<Arc<OriginalSettledProducerObservation>>,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Arc<Self>;

    fn runtime_authority(&self) -> u64;
    fn schema_binding(&self) -> &ApplicationSchemaBindingIdentity;
    fn producer_arc(&self) -> Option<Arc<OriginalSettledProducerObservation>>;
    fn original_settled_producer(&self) -> Option<&OriginalSettledProducerObservation>;
    fn retained_product_lease(&self) -> WorthQueryProductObservationLease;
    fn product_observation(&self) -> &ProductBranchObservation;
    fn same_application_occurrence(&self, other: &Self) -> bool;
    fn matches_occurrence(
        &self,
        runtime_authority: u64,
        schema_binding: &ApplicationSchemaBindingIdentity,
        selected: &ProductBranchObservation,
    ) -> bool;
}

impl ReadObservationInternalPort for WorthQueryApplicationReadObservation {
    fn from_parts(
        runtime_authority: u64,
        schema_binding: ApplicationSchemaBindingIdentity,
        product: WorthQueryProductObservationLease,
        producer: Option<Arc<OriginalSettledProducerObservation>>,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Arc<Self> {
        Self::from_parts(runtime_authority, schema_binding, product, producer, memory)
    }

    fn runtime_authority(&self) -> u64 {
        Self::runtime_authority(self)
    }

    fn schema_binding(&self) -> &ApplicationSchemaBindingIdentity {
        Self::schema_binding(self)
    }

    fn producer_arc(&self) -> Option<Arc<OriginalSettledProducerObservation>> {
        Self::producer_arc(self)
    }

    fn original_settled_producer(&self) -> Option<&OriginalSettledProducerObservation> {
        Self::original_settled_producer(self)
    }

    fn retained_product_lease(&self) -> WorthQueryProductObservationLease {
        Self::retained_product_lease(self)
    }

    fn product_observation(&self) -> &ProductBranchObservation {
        Self::product_observation(self)
    }

    fn same_application_occurrence(&self, other: &Self) -> bool {
        Self::same_application_occurrence(self, other)
    }

    fn matches_occurrence(
        &self,
        runtime_authority: u64,
        schema_binding: &ApplicationSchemaBindingIdentity,
        selected: &ProductBranchObservation,
    ) -> bool {
        Self::matches_occurrence(self, runtime_authority, schema_binding, selected)
    }
}
