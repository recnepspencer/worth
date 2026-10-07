use std::sync::Arc;

use worth_query_installation::facade::{ApplicationSchema, ApplicationSchemaBindingIdentity};

use crate::basis::{WorthQueryProductBranchAdmissionDenial, WorthQueryProductObservationLease};

use super::super::{WorthQueryPrimaryGraphApplicationRuntime, WorthQuerySelectedProductOperation};

/// Owner-retained authority for reading one exact World product occurrence.
///
/// This value carries no publication source and cannot authorize mutation.
pub struct WorthQueryApplicationReadObservation {
    runtime_authority: u64,
    schema_binding: ApplicationSchemaBindingIdentity,
    product: WorthQueryProductObservationLease,
    _required_custody:
        Option<super::super::application_output_demand::RequiredOutputCustodyCapacity>,
}

impl WorthQueryApplicationReadObservation {
    pub(in crate::domain_computation::primary_graph) fn from_product<Schema>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        product: WorthQueryProductObservationLease,
    ) -> Arc<Self>
    where
        Schema: ApplicationSchema,
    {
        Arc::new(Self {
            runtime_authority: runtime.runtime.authority_identity().as_u64(),
            schema_binding: runtime.installed_schema.binding_identity(),
            product,
            _required_custody: None,
        })
    }

    pub(in crate::domain_computation::primary_graph) fn from_product_funded<Schema>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        product: WorthQueryProductObservationLease,
        custody: super::super::application_output_demand::RequiredOutputCustodyCapacity,
    ) -> Arc<Self>
    where
        Schema: ApplicationSchema,
    {
        Arc::new(Self {
            runtime_authority: runtime.runtime.authority_identity().as_u64(),
            schema_binding: runtime.installed_schema.binding_identity(),
            product,
            _required_custody: Some(custody),
        })
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

    /// Historical program-read custody may precede the currently selected
    /// head. It can constrain a fresh required continuation to its original
    /// application and branch occurrence without authorizing the new read.
    pub(in crate::domain_computation::primary_graph) fn belongs_to_selected_occurrence<Schema>(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        selected: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> bool
    where
        Schema: ApplicationSchema,
    {
        self.runtime_authority == runtime.runtime.authority_identity().as_u64()
            && self.schema_binding == runtime.installed_schema.binding_identity()
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
        let (runtime, product, application_basis) = self.into_parts();
        drop(application_basis);
        WorthQueryApplicationReadObservation::from_product(runtime, product.into_read_lease())
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn select_application_read_observation(
        &self,
        observation: &WorthQueryApplicationReadObservation,
    ) -> Result<
        WorthQuerySelectedProductOperation<'_, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    > {
        if observation.runtime_authority != self.runtime.authority_identity().as_u64()
            || observation.schema_binding != self.installed_schema.binding_identity()
        {
            return Err(WorthQueryProductBranchAdmissionDenial::ForeignOwner);
        }
        let product = self
            .product_runtime
            .lease_from_observation(observation.product.observation().clone())?;
        self.on_product(product)
    }
}
