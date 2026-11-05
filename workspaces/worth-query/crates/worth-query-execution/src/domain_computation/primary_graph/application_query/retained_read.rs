use crate::basis::WorthQueryProductBranchAdmissionRefusalKind as AdmissionRefusal;
use crate::domain_computation::primary_graph::product_operation::ContextScopedOps;
use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchema;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use crate::basis::WorthQueryProductBranchAdmissionDenial;
use crate::basis::WorthQueryProductObservationLease;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use super::super::WorthQuerySelectedProductOperation;

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
    pub(in crate::domain_computation::primary_graph) fn with_original_producer(
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

    pub(in crate::domain_computation::primary_graph) fn original_settled_producer(
        &self,
    ) -> Option<&super::super::application_output_demand::OriginalSettledProducerObservation> {
        self.producer.as_deref()
    }

    pub(in crate::domain_computation::primary_graph) fn retained_product_lease(
        &self,
    ) -> WorthQueryProductObservationLease {
        self.product.retained_clone()
    }
    pub(in crate::domain_computation::primary_graph) fn product_observation(
        &self,
    ) -> &worth_runtime_world::facade::ProductBranchObservation {
        self.product.observation()
    }

    pub(in crate::domain_computation::primary_graph) fn same_application_occurrence(
        &self,
        other: &Self,
    ) -> bool {
        self.runtime_authority == other.runtime_authority
            && self.schema_binding == other.schema_binding
            && self.branch_identity() == other.branch_identity()
            && self.branch_incarnation() == other.branch_incarnation()
    }

    pub(in crate::domain_computation::primary_graph) fn from_product<Schema>(
        runtime: &dyn crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>,
        product: WorthQueryProductObservationLease,
    ) -> Arc<Self>
    where
        Schema: ApplicationSchema,
    {
        Self::from_product_with_memory(runtime, product, None)
    }

    pub(in crate::domain_computation::primary_graph) fn from_product_with_memory<
        Schema: ApplicationSchema,
    >(
        runtime: &dyn crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>,
        product: WorthQueryProductObservationLease,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Arc<Self> {
        Arc::new(Self {
            runtime_authority: runtime.authority_identity().as_u64(),
            schema_binding: runtime.installed_schema().binding_identity(),
            product,
            producer: None,
            _memory: memory,
        })
    }

    pub fn branch_identity(&self) -> &worth_runtime_world::facade::ProductBranchIdentity {
        self.product.branch_identity()
    }

    pub(in crate::domain_computation::primary_graph) fn from_settled_product<
        Schema: ApplicationSchema,
    >(
        runtime: &dyn crate::domain_computation::primary_graph::application_output_demand::host::OutputDemandHost<Schema>,
        producer: Arc<super::super::application_output_demand::OriginalSettledProducerObservation>,
        product: WorthQueryProductObservationLease,
    ) -> Arc<Self> {
        Arc::new(Self {
            runtime_authority: runtime.authority_identity().as_u64(),
            schema_binding: runtime.installed_schema().binding_identity(),
            producer: producer
                .matches(runtime, product.observation())
                .then_some(producer),
            product,
            _memory: None,
        })
    }

    pub fn branch_incarnation(&self) -> worth_runtime_world::facade::ProductBranchIncarnation {
        self.product.observation().lifecycle_incarnation()
    }

    pub fn selected_commit(&self) -> &worth_runtime_world::facade::CompositeCommitIdentity {
        self.product.selected_commit()
    }

    /// Historical program-read custody may precede the currently selected
    /// head. It can constrain a fresh required continuation to its original
    /// application and branch occurrence without authorizing the new read.
    pub(in crate::domain_computation::primary_graph) fn belongs_to_selected_occurrence<Schema>(
        &self,
        runtime: &dyn crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>,
        selected: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> bool
    where
        Schema: ApplicationSchema,
    {
        self.runtime_authority == runtime.authority_identity().as_u64()
            && self.schema_binding == runtime.installed_schema().binding_identity()
            && self.product.observation().branch_identity() == selected.branch_identity()
            && self.product.observation().lifecycle_incarnation()
                == selected.lifecycle_incarnation()
    }
}

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

pub trait RetainedReadOps<Schema>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
where
    Schema: ApplicationSchema,
{
    fn select_application_read_observation(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        observation: &WorthQueryApplicationReadObservation,
    ) -> Result<
        WorthQuerySelectedProductOperation<'_, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    > {
        let execution = match phase.execution_request_for(self.product_runtime()) {
            Ok(execution) => execution,
            Err(_) => {
                return Err(WorthQueryProductBranchAdmissionDenial::Refused(
                    AdmissionRefusal::ForeignPhase,
                ))
            }
        };
        self.select_application_read_observation_with_request(&execution, observation)
    }
}

impl<
        Schema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > RetainedReadOps<Schema> for HostRuntime
where
    Schema: ApplicationSchema,
{
}

pub(in crate::domain_computation) trait RetainedReadScopedOps<Schema>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
where
    Schema: ApplicationSchema,
{
    fn select_application_read_observation_with_request(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        observation: &WorthQueryApplicationReadObservation,
    ) -> Result<
        WorthQuerySelectedProductOperation<'_, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    > {
        if observation.runtime_authority != self.authority_identity().as_u64()
            || observation.schema_binding != self.installed_schema().binding_identity()
        {
            return Err(WorthQueryProductBranchAdmissionDenial::Refused(
                AdmissionRefusal::ForeignOwner,
            ));
        }
        let product = self
            .product_runtime()
            .lease_from_observation(observation.product.observation().clone())?;
        self.on_product(execution, product)
            .map(|selected| selected.with_original_settled_producer(observation.producer.clone()))
    }
}

impl<
        Schema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > RetainedReadScopedOps<Schema> for HostRuntime
where
    Schema: ApplicationSchema,
{
}

mod court_doors_for_test;
