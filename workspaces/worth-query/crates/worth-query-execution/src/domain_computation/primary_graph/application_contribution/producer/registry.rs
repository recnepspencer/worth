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
#[cfg(test)]
mod tests;

use operation_binding_uniqueness::duplicate_operation_binding;

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
            .filter(|entry| entry.declaration.output_family == Family::IDENTITY)
            .map(|entry| entry.declaration.operation_binding_type)
            .collect()
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

pub(in crate::domain_computation::primary_graph::application_contribution) struct PendingProducerRegistry<
    Schema,
> {
    declared: BTreeMap<String, DeclaredProducerBinding>,
    providers: BTreeMap<
        String,
        (
            Arc<dyn Any + Send + Sync>,
            Arc<dyn InstalledProducerExecutor<Schema>>,
        ),
    >,
    marker: PhantomData<fn() -> Schema>,
}

impl<Schema> PendingProducerRegistry<Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph::application_contribution) fn new(
        declared: BTreeMap<String, DeclaredProducerBinding>,
    ) -> Self {
        Self {
            declared,
            providers: BTreeMap::new(),
            marker: PhantomData,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_contribution) fn register<
        Binding,
    >(
        &mut self,
        owner: &str,
        provider: Binding::Provider,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        Binding: WorthQueryApplicationProducerBinding<Schema>,
    {
        let declared = self
            .declared
            .get(Binding::IDENTITY)
            .ok_or_else(|| denial(DenialKind::ForeignProducerBinding, Binding::IDENTITY))?;
        if declared.owner != owner {
            return Err(denial(
                DenialKind::ForeignProducerBinding,
                Binding::IDENTITY,
            ));
        }
        if !declared.meaning_matches::<Schema, Binding>() {
            return Err(denial(
                DenialKind::ProducerBindingMeaningMismatch,
                Binding::IDENTITY,
            ));
        }
        if self.providers.contains_key(Binding::IDENTITY) {
            return Err(denial(
                DenialKind::DuplicateProducerBinding,
                Binding::IDENTITY,
            ));
        }
        let provider = Arc::new(provider);
        self.providers.insert(
            Binding::IDENTITY.to_owned(),
            (
                provider.clone(),
                Arc::new(TypedInstalledProducer::<Schema, Binding>::new(provider)),
            ),
        );
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph::application_contribution) fn seal(
        self,
    ) -> Result<
        WorthQueryInstalledApplicationProducerRegistry<Schema>,
        WorthQueryPrimaryGraphInstallationDenial,
    > {
        if let Some((left, right)) = duplicate_operation_binding(&self.declared) {
            return Err(denial(
                DenialKind::DuplicateProducerOperationBinding,
                format!("{left} / {right}"),
            ));
        }
        let missing = self
            .declared
            .keys()
            .find(|identity| !self.providers.contains_key(*identity));
        if let Some(identity) = missing {
            return Err(denial(DenialKind::MissingProducerProvider, identity));
        }
        let entries = self
            .declared
            .into_iter()
            .map(|(identity, declaration)| {
                let (value, executor) = self
                    .providers
                    .get(&identity)
                    .expect("complete provider inventory checked")
                    .clone();
                (
                    identity,
                    InstalledProducerProvider {
                        declaration,
                        value,
                        executor,
                    },
                )
            })
            .collect();
        Ok(WorthQueryInstalledApplicationProducerRegistry {
            entries,
            marker: PhantomData,
        })
    }

    pub(in crate::domain_computation::primary_graph::application_contribution) fn validate_complete(
        &self,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        if let Some(identity) = self
            .declared
            .keys()
            .find(|identity| !self.providers.contains_key(*identity))
        {
            Err(denial(DenialKind::MissingProducerProvider, identity))
        } else {
            Ok(())
        }
    }
}

fn denial(
    kind: DenialKind,
    subject: impl Into<String>,
) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(kind, subject)
}
