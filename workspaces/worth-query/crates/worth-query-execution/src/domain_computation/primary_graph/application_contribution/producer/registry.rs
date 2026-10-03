use std::any::{Any, TypeId};
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_declaration::facade::application_operation::ApplicationMutationOutputRoleFamilyDescriptor;
use worth_query_installation::facade::ApplicationSchema;

mod checkpoint;
mod declaration;

use super::{
    scheduling, InstalledProducerExecutor, WorthQueryApplicationProducerBinding,
    WorthQueryInstalledOutputProducerRoutes, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind, WorthQueryProducerApplicability,
    WorthQueryProducerInvariantRequirement, WorthQueryProducerOutputFamily,
    WorthQuerySelectedApplicationProducer,
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
    pub(in crate::domain_computation::primary_graph::application_contribution) handler_identity:
        String,
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
    pub(in crate::domain_computation::primary_graph::application_contribution) input_reuse:
        Option<super::WorthQueryProducerInputReuseContract>,
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
mod pending_factory;
mod semantic_edition;
#[cfg(test)]
mod tests;

use operation_binding_uniqueness::duplicate_operation_binding;
pub(in crate::domain_computation::primary_graph) use pending::PendingProducerRegistry;
pub(in crate::domain_computation::primary_graph) use semantic_edition::InstalledProducerEdition;

pub(super) struct InstalledProducerProvider<Schema> {
    pub(super) declaration: DeclaredProducerBinding,
    pub(super) edition: InstalledProducerEdition,
    value: Arc<dyn Any + Send + Sync>,
    pub(super) executor: Arc<dyn InstalledProducerExecutor<Schema>>,
}

pub struct WorthQueryInstalledApplicationProducerRegistry<Schema> {
    pub(super) entries: BTreeMap<String, Arc<InstalledProducerProvider<Schema>>>,
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
    // Ordinary producer selection retains its established installation/read
    // class. Required Fresh supplies its carried remainder, so each reached
    // installed predicate and the chosen identity copy are paid before use.
    pub(super) fn select_entry<Family>(
        &self,
        mut remaining_work: Option<&mut usize>,
        mut accepts: impl FnMut(&Arc<InstalledProducerProvider<Schema>>) -> bool,
        missing: WorthQueryOutputDemandDenialKind,
    ) -> Result<&Arc<InstalledProducerProvider<Schema>>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        if let Some(remaining) = remaining_work.as_deref_mut() {
            *remaining = remaining.checked_sub(3).ok_or_else(|| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                    "",
                )
            })?;
        }
        let mut selected = None;
        for entry in self.entries.values() {
            if let Some(remaining) = remaining_work.as_deref_mut() {
                let headers = std::mem::size_of::<Arc<InstalledProducerProvider<Schema>>>()
                    .checked_add(7)
                    .ok_or_else(|| {
                        WorthQueryOutputDemandDenial::new(
                            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                            "",
                        )
                    })?;
                *remaining = remaining.checked_sub(headers).ok_or_else(|| {
                    WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                        "",
                    )
                })?;
                let predicate = Family::IDENTITY
                    .len()
                    .checked_add(entry.declaration.output_family.len())
                    // Exact selection compares two TypeId pairs; the
                    // applicability case reads both sides of each slot.
                    .and_then(|work| work.checked_add(std::mem::size_of::<TypeId>() * 4))
                    .and_then(|work| {
                        work.checked_add(entry.declaration.applicability.len().checked_mul(
                            std::mem::size_of::<WorthQueryProducerApplicability>() * 2,
                        )?)
                    })
                    .and_then(|work| work.checked_add(3))
                    .ok_or_else(|| {
                        WorthQueryOutputDemandDenial::new(
                            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                            "",
                        )
                    })?;
                *remaining = remaining.checked_sub(predicate).ok_or_else(|| {
                    WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                        "",
                    )
                })?;
            }
            if accepts(entry) {
                if selected.is_some() {
                    return Err(WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::AmbiguousApplicableProducer,
                        Family::IDENTITY,
                    ));
                }
                selected = Some(entry);
            }
        }
        let selected =
            selected.ok_or_else(|| WorthQueryOutputDemandDenial::new(missing, Family::IDENTITY))?;
        if let Some(remaining) = remaining_work.as_deref_mut() {
            let copied = selected
                .declaration
                .identity
                .len()
                .checked_add(std::mem::size_of::<String>())
                .and_then(|work| work.checked_add(2))
                .ok_or_else(|| {
                    WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                        "",
                    )
                })?;
            *remaining = remaining.checked_sub(copied).ok_or_else(|| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                    "",
                )
            })?;
        }
        Ok(selected)
    }

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
        remaining_work: Option<&mut usize>,
    ) -> Result<
        (
            WorthQuerySelectedApplicationProducer,
            &Arc<InstalledProducerProvider<Schema>>,
        ),
        WorthQueryOutputDemandDenial,
    >
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let selected = self.select_entry::<Family>(
            remaining_work,
            |entry| {
                entry.declaration.output_family == Family::IDENTITY
                    && entry.declaration.output_family_type == TypeId::of::<Family>()
                    && entry.declaration.operation_binding_type == output_binding
            },
            WorthQueryOutputDemandDenialKind::ProducerUnavailable,
        )?;
        Ok((
            WorthQuerySelectedApplicationProducer {
                identity: selected.declaration.identity.clone(),
                applicability: selected.declaration.applicability[0],
                exact_retained_output: true,
                retained_resources: None,
                retained_idempotency_key: None,
                retained_output_binding: None,
            },
            selected,
        ))
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
