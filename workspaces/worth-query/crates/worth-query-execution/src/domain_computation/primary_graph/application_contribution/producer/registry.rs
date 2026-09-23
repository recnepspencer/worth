use std::any::{Any, TypeId};
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_declaration::facade::application_operation::ApplicationMutationOutputRoleFamilyDescriptor;
use worth_query_installation::facade::ApplicationSchema;

mod checkpoint;
mod declaration;

use super::{
    scheduling, InstalledProducerExecutor, TypedInstalledProducer,
    WorthQueryApplicationProducerBinding, WorthQueryInstalledOutputProducerRoutes,
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryProducerApplicability, WorthQueryProducerInvariantRequirement,
    WorthQueryProducerOutputFamily, WorthQuerySelectedApplicationProducer,
};
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind as DenialKind,
};

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph::application_contribution) struct DeclaredProducerBinding
{
    pub(in crate::domain_computation::primary_graph::application_contribution) owner: String,
    pub(in crate::domain_computation::primary_graph::application_contribution) identity: String,
    pub(in crate::domain_computation::primary_graph::application_contribution) source_selector:
        String,
    pub(in crate::domain_computation::primary_graph::application_contribution) output_family:
        String,
    pub(in crate::domain_computation::primary_graph::application_contribution) output_family_type:
        TypeId,
    pub(in crate::domain_computation::primary_graph::application_contribution) output_roles:
        Vec<String>,
    pub(in crate::domain_computation::primary_graph::application_contribution) output_role_descriptors:
        Vec<worth_query_declaration::facade::application_operation::ApplicationMutationOutputRoleDescriptor>,
    pub(in crate::domain_computation::primary_graph::application_contribution) output_role_families:
        Vec<ApplicationMutationOutputRoleFamilyDescriptor>,
    pub(in crate::domain_computation::primary_graph::application_contribution) output_role: String,
    pub(in crate::domain_computation::primary_graph::application_contribution) operation: String,
    pub(in crate::domain_computation::primary_graph::application_contribution) provider_identity:
        String,
    pub(in crate::domain_computation::primary_graph::application_contribution) applicability:
        Vec<WorthQueryProducerApplicability>,
    pub(in crate::domain_computation::primary_graph::application_contribution) supported:
        Vec<WorthQueryProducerApplicability>,
    pub(in crate::domain_computation::primary_graph::application_contribution) required_invariants:
        Vec<WorthQueryProducerInvariantRequirement>,
    pub(in crate::domain_computation::primary_graph::application_contribution) resource_policy:
        String,
    pub(in crate::domain_computation::primary_graph::application_contribution) reuse_policy: String,
    pub(in crate::domain_computation::primary_graph::application_contribution) binding_type: TypeId,
    pub(in crate::domain_computation::primary_graph::application_contribution) source_type: TypeId,
    pub(in crate::domain_computation::primary_graph::application_contribution) operation_binding_type:
        TypeId,
    pub(in crate::domain_computation::primary_graph::application_contribution) provider_type:
        TypeId,
}

mod operation_binding_uniqueness;
mod output_family_inventory;
mod pending;
#[cfg(test)]
mod tests;

use operation_binding_uniqueness::duplicate_operation_binding;
pub(in crate::domain_computation::primary_graph::application_contribution) use pending::PendingProducerRegistry;

pub(super) struct InstalledProducerProvider<Schema> {
    pub(super) declaration: DeclaredProducerBinding,
    value: Arc<dyn Any + Send + Sync>,
    pub(super) executor: Arc<dyn InstalledProducerExecutor<Schema>>,
}

impl<Schema> Clone for InstalledProducerProvider<Schema> {
    fn clone(&self) -> Self {
        Self {
            declaration: self.declaration.clone(),
            value: Arc::clone(&self.value),
            executor: Arc::clone(&self.executor),
        }
    }
}

pub struct WorthQueryInstalledApplicationProducerRegistry<Schema> {
    pub(super) entries: BTreeMap<String, InstalledProducerProvider<Schema>>,
    marker: PhantomData<fn() -> Schema>,
}

impl<Schema> Clone for WorthQueryInstalledApplicationProducerRegistry<Schema> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            marker: PhantomData,
        }
    }
}

impl<Schema> Default for WorthQueryInstalledApplicationProducerRegistry<Schema> {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
            marker: PhantomData,
        }
    }
}

impl<Schema> WorthQueryInstalledApplicationProducerRegistry<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn provider<Binding>(&self) -> Option<Arc<Binding::Provider>>
    where
        Binding: WorthQueryApplicationProducerBinding<Schema>,
    {
        self.entries
            .get(Binding::IDENTITY)
            .filter(|entry| entry.declaration.meaning_matches::<Schema, Binding>())
            .and_then(|entry| {
                Arc::clone(&entry.value)
                    .downcast::<Binding::Provider>()
                    .ok()
            })
    }

    pub(in crate::domain_computation::primary_graph::application_contribution) fn family_output_bindings<
        Family,
    >(
        &self,
    ) -> Vec<TypeId>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        self.entries
            .values()
            .filter(|entry| {
                entry.declaration.output_family == Family::IDENTITY
                    && entry.declaration.output_family_type == TypeId::of::<Family>()
            })
            .map(|entry| entry.declaration.operation_binding_type)
            .collect()
    }

    pub(in crate::domain_computation::primary_graph::application_contribution) fn family_output_role<
        Family,
    >(
        &self,
        output_binding: TypeId,
    ) -> Result<&str, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let mut matching = self.entries.values().filter(|entry| {
            entry.declaration.output_family == Family::IDENTITY
                && entry.declaration.output_family_type == TypeId::of::<Family>()
                && entry.declaration.operation_binding_type == output_binding
        });
        let role = matching.next().ok_or_else(|| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ProducerUnavailable,
                Family::IDENTITY,
            )
        })?;
        if matching.any(|entry| entry.declaration.output_role != role.declaration.output_role) {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::AmbiguousApplicableProducer,
                Family::IDENTITY,
            ));
        }
        Ok(&role.declaration.output_role)
    }

    pub(in crate::domain_computation::primary_graph::application_contribution) fn select_exact<
        Family,
    >(
        &self,
        output_binding: TypeId,
    ) -> Result<WorthQuerySelectedApplicationProducer, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let mut matching = self.entries.values().filter(|entry| {
            entry.declaration.output_family == Family::IDENTITY
                && entry.declaration.output_family_type == TypeId::of::<Family>()
                && entry.declaration.operation_binding_type == output_binding
        });
        let selected = matching.next().ok_or_else(|| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ProducerUnavailable,
                Family::IDENTITY,
            )
        })?;
        if matching.next().is_some() {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::AmbiguousApplicableProducer,
                Family::IDENTITY,
            ));
        }
        Ok(WorthQuerySelectedApplicationProducer {
            identity: selected.declaration.identity.clone(),
            applicability: selected.declaration.applicability[0],
        })
    }

    pub(in crate::domain_computation::primary_graph) fn install_signal_routes(
        &self,
        builder: &mut worth_runtime_bridge::facade::BridgeConditionalRuntimeBuilder,
    ) -> Result<WorthQueryInstalledOutputProducerRoutes, WorthQueryPrimaryGraphInstallationDenial>
    {
        scheduling::install_output_producer_routes(
            self.entries
                .values()
                .map(|entry| entry.declaration.identity.as_str()),
            builder,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

fn denial(
    kind: DenialKind,
    subject: impl Into<String>,
) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(kind, subject)
}
