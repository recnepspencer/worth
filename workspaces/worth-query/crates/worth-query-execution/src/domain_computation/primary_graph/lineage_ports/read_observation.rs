//! Runtime constructors and occurrence checks on `WorthQueryApplicationReadObservation`.

use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchema;

use crate::basis::WorthQueryProductObservationLease;
use crate::domain_computation::primary_graph::application_output_demand::OriginalSettledProducerObservation;
use crate::domain_computation::primary_graph::{
    OriginalSettledProducerObservationExt, WorthQueryApplicationReadObservation,
    WorthQueryPrimaryGraphApplicationRuntime,
};

pub(in crate::domain_computation::primary_graph) trait ReadObservationRuntimeExt: Sized {
    fn from_product<Schema: ApplicationSchema>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        product: WorthQueryProductObservationLease,
    ) -> Arc<Self>;

    fn from_product_with_memory<Schema: ApplicationSchema>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        product: WorthQueryProductObservationLease,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Arc<Self>;

    fn from_settled_product<Schema: ApplicationSchema>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        producer: Arc<OriginalSettledProducerObservation>,
        product: WorthQueryProductObservationLease,
    ) -> Arc<Self>;

    fn belongs_to_selected_occurrence<Schema: ApplicationSchema>(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        selected: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> bool;
}

impl ReadObservationRuntimeExt for WorthQueryApplicationReadObservation {
    fn from_product<Schema: ApplicationSchema>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        product: WorthQueryProductObservationLease,
    ) -> Arc<Self> {
        Self::from_product_with_memory(runtime, product, None)
    }

    fn from_product_with_memory<Schema: ApplicationSchema>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        product: WorthQueryProductObservationLease,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Arc<Self> {
        Self::from_parts(
            runtime.runtime.authority_identity().as_u64(),
            runtime.installed_schema.binding_identity(),
            product,
            None,
            memory,
        )
    }

    fn from_settled_product<Schema: ApplicationSchema>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        producer: Arc<OriginalSettledProducerObservation>,
        product: WorthQueryProductObservationLease,
    ) -> Arc<Self> {
        let matched = producer
            .matches(runtime, product.observation())
            .then_some(producer);
        Self::from_parts(
            runtime.runtime.authority_identity().as_u64(),
            runtime.installed_schema.binding_identity(),
            product,
            matched,
            None,
        )
    }

    fn belongs_to_selected_occurrence<Schema: ApplicationSchema>(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        selected: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> bool {
        self.matches_occurrence(
            runtime.runtime.authority_identity().as_u64(),
            &runtime.installed_schema.binding_identity(),
            selected,
        )
    }
}
