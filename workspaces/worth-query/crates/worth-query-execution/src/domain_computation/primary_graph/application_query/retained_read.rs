use std::sync::Arc;

use worth_query_installation::facade::{ApplicationSchema, ApplicationSchemaBindingIdentity};

use crate::basis::{
    WorthQueryProductBranchAdmissionDenial, WorthQueryProductBranchLease,
    WorthQueryProductObservationLease,
};

use super::super::{WorthQueryPrimaryGraphApplicationRuntime, WorthQuerySelectedProductOperation};

/// Owner-retained authority for reading one exact World product occurrence.
///
/// This value carries no publication source and cannot authorize mutation.
pub struct WorthQueryApplicationReadObservation {
    runtime_authority: u64,
    schema_binding: ApplicationSchemaBindingIdentity,
    bridge_snapshot_identity: Option<worth_runtime_bridge::facade::TruthSnapshotIdentity>,
    product: WorthQueryProductObservationLease,
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
            bridge_snapshot_identity: None,
            product,
        })
    }

    fn from_selected_product<Schema>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        product: WorthQueryProductBranchLease,
    ) -> Arc<Self>
    where
        Schema: ApplicationSchema,
    {
        let bridge_snapshot_identity = product.bridge_snapshot_identity().clone();
        Arc::new(Self {
            runtime_authority: runtime.runtime.authority_identity().as_u64(),
            schema_binding: runtime.installed_schema.binding_identity(),
            bridge_snapshot_identity: Some(bridge_snapshot_identity),
            product: product.into_read_lease(),
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

    pub(crate) fn bridge_snapshot_identity(
        &self,
    ) -> Option<&worth_runtime_bridge::facade::TruthSnapshotIdentity> {
        self.bridge_snapshot_identity.as_ref()
    }
}

impl<'runtime, Schema> WorthQuerySelectedProductOperation<'runtime, Schema>
where
    Schema: ApplicationSchema,
{
    pub fn retain_application_read(self) -> Arc<WorthQueryApplicationReadObservation> {
        let (runtime, product, application_basis) = self.into_parts();
        drop(application_basis);
        WorthQueryApplicationReadObservation::from_selected_product(runtime, product)
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
