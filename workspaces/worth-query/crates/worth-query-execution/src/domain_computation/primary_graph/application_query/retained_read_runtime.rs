//! Runtime selection and retention hooks over `WorthQueryApplicationReadObservation`.

use crate::domain_computation::primary_graph::ReadObservationRuntimeExt;
use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchema;

use crate::basis::{
    WorthQueryProductBranchAdmissionDenial, WorthQueryProductBranchAdmissionRefusalKind as AdmissionRefusal,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationReadObservation, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQuerySelectedProductOperation,
};

impl<'runtime, Schema> WorthQuerySelectedProductOperation<'runtime, Schema>
where
    Schema: ApplicationSchema,
{
    pub fn retain_application_read(self) -> Arc<WorthQueryApplicationReadObservation> {
        let producer = self.original_settled_producer().cloned();
        let (runtime, product, application_basis) = self.into_parts();
        drop(application_basis);
        match producer {
            Some(producer) => WorthQueryApplicationReadObservation::from_settled_product(
                runtime,
                producer,
                product.into_read_lease(),
            ),
            None => WorthQueryApplicationReadObservation::from_product(
                runtime,
                product.into_read_lease(),
            ),
        }
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn select_application_read_observation(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        observation: &WorthQueryApplicationReadObservation,
    ) -> Result<
        WorthQuerySelectedProductOperation<'_, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    > {
        let execution = match phase.execution_request_for(&self.runtime.product_runtime) {
            Ok(execution) => execution,
            Err(_) => {
                return Err(WorthQueryProductBranchAdmissionDenial::Refused(
                    AdmissionRefusal::ForeignPhase,
                ))
            }
        };
        self.select_application_read_observation_with_request(&execution, observation)
    }

    pub(in crate::domain_computation) fn select_application_read_observation_with_request(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observation: &WorthQueryApplicationReadObservation,
    ) -> Result<
        WorthQuerySelectedProductOperation<'_, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    > {
        if observation.runtime_authority() != self.runtime.authority_identity().as_u64()
            || observation.schema_binding() != &self.installed_schema.binding_identity()
        {
            return Err(WorthQueryProductBranchAdmissionDenial::Refused(
                AdmissionRefusal::ForeignOwner,
            ));
        }
        let product = self
            .runtime
            .product_runtime
            .lease_from_observation(observation.product_observation().clone())?;
        self.on_product(execution, product)
            .map(|selected| selected.with_original_settled_producer(observation.producer_arc()))
    }
}
