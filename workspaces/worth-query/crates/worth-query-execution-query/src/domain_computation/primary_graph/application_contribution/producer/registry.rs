use crate::equality_type_id::EqualityTypeId as TypeId;
use std::any::Any;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_declaration::facade::application_operation::ApplicationMutationOutputRoleFamilyDescriptor;
use worth_query_installation::facade::ApplicationSchema;

pub mod checkpoint;
mod codec_transition;
mod coupled_selection;
mod declaration;
mod seal_refusal;
pub use seal_refusal::RefusedProducerSeal;

use super::scheduling;
use super::InstalledProducerExecutor;
use super::WorthQueryApplicationProducerBinding;
use super::WorthQueryInstalledOutputProducerRoutes;
use super::WorthQueryOutputDemandDenial;
use super::WorthQueryOutputDemandDenialKind;
use super::WorthQueryProducerApplicability;
use super::WorthQueryProducerInvariantRequirement;
use super::WorthQueryProducerOutputFamily;
use super::WorthQuerySelectedApplicationProducer;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind as DenialKind;

/// Selection work on the request's meter; running out is the selection's
/// work denial.

#[derive(Clone)]
pub struct DeclaredProducerBinding
{
    pub owner: String,
    pub identity: String,
    pub source_selector:
        String,
    pub output_family:
        String,
    pub output_family_type:
        TypeId,
    pub output_role_descriptors:
        Vec<worth_query_declaration::facade::application_operation::ApplicationMutationOutputRoleDescriptor>,
    pub output_role_families:
        Vec<ApplicationMutationOutputRoleFamilyDescriptor>,
    pub output_role: String,
    pub operation: String,
    pub handler_identity:
        String,
    pub provider_identity:
        String,
    pub applicability:
        Vec<WorthQueryProducerApplicability>,
    pub supported:
        Vec<WorthQueryProducerApplicability>,
    pub required_invariants:
        Vec<WorthQueryProducerInvariantRequirement>,
    pub reuse_policy: String,
    pub ownership:
        worth_foundational::facade::ProducerPublicationOwnership,
    pub owned_scope_entity: Option<String>,
    pub output_equivalence:
        Option<super::super::native_field_contract::NativeFieldBindingExpectation>,
    pub installed_output_equivalence:
        Option<Arc<super::output_equivalence_contract::InstalledProducerOutputEquivalence>>,
    pub predecessor_reads: Option<Vec<super::observation_reads::ProducerObservationReadMeaning>>,
    pub original_reads: Option<Vec<super::observation_reads::ProducerObservationReadMeaning>>,
    pub input_reuse:
        Option<super::WorthQueryProducerInputReuseContract>,
    pub input_capture:
        Option<super::input_capture::InputCaptureMeaning>,
    pub binding_type: TypeId,
    pub source_type: TypeId,
    pub operation_binding_type:
        TypeId,
    /// The operation the producer's mutation binding runs.
    pub operation_type: TypeId,
    pub output_contract_type:
        TypeId,
    pub provider_type:
        TypeId,
}

mod history_group;
mod implementation;
pub mod operation_binding_uniqueness;
pub mod output_family_inventory;
mod pending;
mod pending_factory;
pub mod scope_ownership;
pub mod semantic_edition;
pub mod semantic_edition_ext;

use operation_binding_uniqueness::duplicate_operation_binding;
pub use pending::PendingProducerRegistry;
pub use semantic_edition::InstalledProducerEdition;

pub struct InstalledProducerProvider<Schema> {
    pub(crate) declaration:
        DeclaredProducerBinding,
    pub(crate) edition:
        InstalledProducerEdition,
    value: Arc<dyn Any + Send + Sync>,
    pub(crate) executor: Arc<dyn InstalledProducerExecutor<Schema>>,
}

impl<S> InstalledProducerProvider<S> {
    pub fn coupled_executor(
        &self,
    ) -> &dyn super::execution::InstalledCoupledProducerExecutor<S> {
        self.executor.as_ref()
    }
}

pub struct WorthQueryInstalledApplicationProducerRegistry<Schema> {
    pub(crate) entries: BTreeMap<String, Arc<InstalledProducerProvider<Schema>>>,
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

impl<Schema> WorthQueryInstalledApplicationProducerRegistry<Schema> {
    pub fn preflight_home_retirement(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryHomeRetirementDenial> {
        use crate::domain_computation::primary_graph::{
            WorthQueryHomeRetirementDenial as Denial, WorthQueryHomeRetirementOwner as Owner,
        };
        for entry in self.entries.values_mut() {
            request.consult().map_err(Denial::Execution)?;
            let original = Arc::get_mut(entry).ok_or(Denial::Busy(Owner::InstalledProducers))?;
            Arc::get_mut(&mut original.executor).ok_or(Denial::Busy(Owner::InstalledProducers))?;
        }
        Ok(())
    }
}

impl<Schema> WorthQueryInstalledApplicationProducerRegistry<Schema>
where
    Schema: ApplicationSchema,
{
    // Ordinary producer selection retains its established installation/read
    // class. Required Fresh supplies its request meter, so each reached
    // installed predicate and the chosen identity copy are paid before use.
    pub fn select_entry<Family>(
        &self,

        accepts: impl FnMut(&Arc<InstalledProducerProvider<Schema>>) -> bool,
        missing: WorthQueryOutputDemandDenialKind,
    ) -> Result<&Arc<InstalledProducerProvider<Schema>>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        self.select_matching_entry(accepts, missing, Family::IDENTITY)
    }
    fn select_matching_entry(
        &self,
        mut accepts: impl FnMut(&Arc<InstalledProducerProvider<Schema>>) -> bool,
        missing: WorthQueryOutputDemandDenialKind,
        subject: &'static str,
    ) -> Result<&Arc<InstalledProducerProvider<Schema>>, WorthQueryOutputDemandDenial> {
        let mut selected = None;
        for entry in self.entries.values() {
            if accepts(entry) {
                if selected.is_some() {
                    return Err(WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::AmbiguousApplicableProducer,
                        subject,
                    ));
                }
                selected = Some(entry);
            }
        }
        let selected =
            selected.ok_or_else(|| WorthQueryOutputDemandDenial::new(missing, subject))?;

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

    pub fn family_output_bindings<
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

    pub fn family_output_role<
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

    pub fn select_exact<Family>(
        &self,
        output_binding: TypeId,
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
                retained_read_posture: super::demand::RetainedOutputReadPosture::Ordinary,

                retained_idempotency_key: None,
                retained_output_binding: None,
                reuses_live_output_only: !selected.declaration.applicability.iter().any(
                    |applicability| {
                        applicability.lifecycle()
                            == super::WorthQueryProducerLifecyclePosture::Preserve
                    },
                ),
            },
            selected,
        ))
    }

    pub fn install_signal_routes(
        &self,
        builder: &mut worth_runtime_bridge::facade::BridgeConditionalRuntimeBuilder,
    ) -> Result<WorthQueryInstalledOutputProducerRoutes, WorthQueryPrimaryGraphInstallationDenial>
    {
        scheduling::install_output_producer_routes(
            self.entries.values().map(|entry| {
                (
                    entry.declaration.identity.as_str(),
                    entry
                        .declaration
                        .installed_output_equivalence
                        .as_ref()
                        .map(|field| (field, entry.declaration.output_role.as_str())),
                )
            }),
            builder,
        )
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[doc(hidden)]
    pub fn installed_identities_for_test(&self) -> Vec<&str> {
        self.entries.keys().map(String::as_str).collect()
    }
}

impl<Schema: 'static> WorthQueryInstalledApplicationProducerRegistry<Schema> {
    #[doc(hidden)]
    pub fn retain_entry_for_test(&self) -> Option<Box<dyn Any>> {
        self.entries
            .values()
            .next()
            .map(|entry| Box::new(entry.clone()) as Box<dyn Any>)
    }
    #[doc(hidden)]
    pub fn retain_weak_entry_for_test(&self) -> Option<Box<dyn Any>> {
        self.entries
            .values()
            .next()
            .map(|entry| Box::new(Arc::downgrade(entry)) as Box<dyn Any>)
    }
    #[doc(hidden)]
    pub fn retain_executor_for_test(&self) -> Option<Box<dyn Any>> {
        self.entries
            .values()
            .next()
            .map(|entry| Box::new(entry.executor.clone()) as Box<dyn Any>)
    }
    #[doc(hidden)]
    pub fn retain_weak_executor_for_test(&self) -> Option<Box<dyn Any>> {
        self.entries
            .values()
            .next()
            .map(|entry| Box::new(Arc::downgrade(&entry.executor)) as Box<dyn Any>)
    }
}

fn denial(
    kind: DenialKind,
    subject: impl Into<String>,
) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(kind, subject)
}

mod court_doors_for_test;
