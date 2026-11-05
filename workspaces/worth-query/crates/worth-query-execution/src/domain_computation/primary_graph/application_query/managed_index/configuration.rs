//! Immutable installed callbacks, separate from the existing live view registry.
use super::{
    adapter::{ErasedIndexExecutor, InstalledIndexProvider, TypedIndexProvider},
    WorthQueryManagedIndexProvider,
};
use crate::{
    domain_computation::primary_graph::{
        WorthQueryPrimaryGraphInstallationDenial as Denial,
        WorthQueryPrimaryGraphInstallationDenialKind as Kind,
    },
    equality_type_id::EqualityTypeId,
};
use std::{any::Any, collections::HashMap, marker::PhantomData, sync::Arc};
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding,
    application_schema::{ApplicationSchema, ApplicationSchemaMember},
};
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

struct ConfiguredProvider<S> {
    name: String,
    refinement: bool,
    adapter: Box<dyn Any + Send + Sync>,
    belongs_to: fn(&dyn Any, &WorthQueryInstalledApplicationSchema<S>) -> bool,
    executor: Arc<dyn ErasedIndexExecutor<S>>,
    source: Arc<dyn super::source::service::SourceFactory>,
}

struct ConfiguredProviders<S> {
    entries: HashMap<EqualityTypeId, ConfiguredProvider<S>>,
    registration_order: Vec<EqualityTypeId>,
}

pub struct ManagedIndexProviderCatalog<S> {
    providers: Arc<ConfiguredProviders<S>>,
    marker: PhantomData<fn() -> S>,
}
impl<S> Default for ManagedIndexProviderCatalog<S> {
    fn default() -> Self {
        Self {
            providers: Arc::new(ConfiguredProviders {
                entries: HashMap::new(),
                registration_order: Vec::new(),
            }),
            marker: PhantomData,
        }
    }
}

impl<S> Clone for ManagedIndexProviderCatalog<S> {
    fn clone(&self) -> Self {
        Self {
            providers: Arc::clone(&self.providers),
            marker: PhantomData,
        }
    }
}

impl<S: ApplicationSchema> ManagedIndexProviderCatalog<S> {
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn configured_instances_for_test(
        &self,
    ) -> Vec<*const ()> {
        self.in_registration_order()
            .map(|entry| Arc::as_ptr(&entry.executor).cast::<()>())
            .collect()
    }

    fn in_registration_order(&self) -> impl Iterator<Item = &ConfiguredProvider<S>> {
        self.providers
            .registration_order
            .iter()
            .map(|identity| &self.providers.entries[identity])
    }

    pub(super) fn is_empty(&self) -> bool {
        self.providers.entries.is_empty()
    }
    pub(super) fn source_factory(
        &self,
        definition: &worth_query_declaration::facade::application_query::ErasedApplicationManagedIndexDefinition,
    ) -> Option<Arc<dyn super::source::service::SourceFactory>> {
        self.in_registration_order()
            .find(|entry| entry.source.definition() == definition)
            .map(|entry| Arc::clone(&entry.source))
    }
    pub(in crate::domain_computation::primary_graph) fn take_complete(
        &mut self,
        schema: &WorthQueryInstalledApplicationSchema<S>,
    ) -> Result<Self, Denial> {
        self.validate_complete(schema)?;
        Ok(std::mem::take(self))
    }

    pub(in crate::domain_computation::primary_graph) fn validate_complete(
        &self,
        schema: &WorthQueryInstalledApplicationSchema<S>,
    ) -> Result<(), Denial> {
        for entry in self.in_registration_order() {
            if !(entry.belongs_to)(entry.adapter.as_ref(), schema) {
                return Err(Denial::new(
                    Kind::ManagedIndexProviderMeaningMismatch,
                    &entry.name,
                ));
            }
        }
        for member in schema.installed_declaration().members() {
            if let ApplicationSchemaMember::ApplicationQuery { definition } = member
                && let Some(selection) = definition.managed_index()
                && selection.posture() == worth_query_declaration::facade::application_query::ApplicationManagedIndexMatchPosture::Refined
                && !self.in_registration_order().any(|entry| entry.name == selection.parts().index && entry.refinement) {
                return Err(Denial::new(Kind::MissingManagedIndexRefinement, &selection.parts().index));
            }

            if let ApplicationSchemaMember::ManagedIndex { definition } = member
                && !self
                    .in_registration_order()
                    .any(|entry| entry.name == definition.parts().name)
            {
                return Err(Denial::new(
                    Kind::MissingManagedIndexProvider,
                    &definition.parts().name,
                ));
            }
        }
        Ok(())
    }
}

impl<S: ApplicationSchema + 'static> ManagedIndexProviderCatalog<S> {
    pub(in crate::domain_computation::primary_graph) fn register<I, P>(
        &mut self,
        schema: &WorthQueryInstalledApplicationSchema<S>,
        owner: &str,
        provider: P,
        refinement: Option<super::refinement::RefinementCallbacks<S, I, P>>,
    ) -> Result<(), Denial>
    where
        I: ApplicationManagedIndexBinding<S>,
        P: WorthQueryManagedIndexProvider<S, I>,
    {
        self.register_shared::<I, P>(schema, owner, Arc::new(provider), None, refinement)
    }

    pub(in crate::domain_computation::primary_graph) fn register_predicate<I, P>(
        &mut self,
        schema: &WorthQueryInstalledApplicationSchema<S>,
        owner: &str,
        provider: P,
        refinement: Option<super::refinement::RefinementCallbacks<S, I, P>>,
    ) -> Result<(), Denial>
    where
        I: ApplicationManagedIndexBinding<S>,
        P: super::WorthQueryManagedIndexPredicateProvider<S, I>,
    {
        let provider = Arc::new(provider);
        let installed = schema
            .installed_managed_index::<I>()
            .map_err(|_| Denial::new(Kind::ManagedIndexProviderMeaningMismatch, I::NAME))?;
        let predicate = Arc::new(super::predicate::TypedPredicateFactory {
            installed,
            provider: Arc::clone(&provider),
            bindings: schema.value_bindings().clone(),
            refinement,
        });
        self.register_shared::<I, P>(schema, owner, provider, Some(predicate), refinement)
    }

    fn register_shared<I, P>(
        &mut self,
        schema: &WorthQueryInstalledApplicationSchema<S>,
        owner: &str,
        provider: Arc<P>,
        predicate: Option<Arc<dyn super::predicate::PredicateFactory>>,
        refinement: Option<super::refinement::RefinementCallbacks<S, I, P>>,
    ) -> Result<(), Denial>
    where
        I: ApplicationManagedIndexBinding<S>,
        P: WorthQueryManagedIndexProvider<S, I>,
    {
        if P::IDENTITY != I::PROVIDER || P::VERSION != I::PROVIDER_VERSION {
            return Err(Denial::new(
                Kind::ManagedIndexProviderMeaningMismatch,
                I::NAME,
            ));
        }
        let installed = schema
            .installed_managed_index::<I>()
            .map_err(|_| Denial::new(Kind::ManagedIndexProviderMeaningMismatch, I::NAME))?;
        if refinement.is_some_and(|actual| {
            installed.definition().refinement() != Some(&actual.meaning.meaning())
        }) {
            return Err(Denial::new(
                Kind::ManagedIndexProviderMeaningMismatch,
                I::NAME,
            ));
        }
        let contribution = schema
            .contributions()
            .get(owner)
            .ok_or_else(|| Denial::new(Kind::ContributionMemberMismatch, owner))?;
        if !contribution.members().any(|member| matches!(member,
            ApplicationSchemaMember::ManagedIndex { definition } if definition == installed.definition())) {
            return Err(Denial::new(Kind::ContributionMemberMismatch, I::NAME));
        }
        let index_type = EqualityTypeId::of::<I>();
        if self.providers.entries.contains_key(&index_type)
            || self
                .in_registration_order()
                .any(|entry| entry.name == I::NAME)
        {
            return Err(Denial::new(Kind::DuplicateManagedIndexProvider, I::NAME));
        }
        let concrete = Arc::new(TypedIndexProvider {
            installed,
            provider,
            predicate,
            refinement,
            coverage: crate::domain_computation::primary_graph::access_strategy::coverage::CoverageVerifier::new(),
            bindings: schema.value_bindings().clone(),
        });
        let adapter: Arc<dyn InstalledIndexProvider<S, I>> = concrete.clone();
        let executor: Arc<dyn ErasedIndexExecutor<S>> = concrete.clone();
        let source: Arc<dyn super::source::service::SourceFactory> = concrete;
        let providers = Arc::get_mut(&mut self.providers)
            .ok_or_else(|| Denial::new(Kind::ManagedIndexProviderConfigurationSealed, I::NAME))?;
        providers.entries.insert(
            index_type,
            ConfiguredProvider {
                name: I::NAME.to_owned(),
                refinement: refinement.is_some(),
                adapter: Box::new(adapter),
                belongs_to: configured_in::<S, I>,
                executor,
                source,
            },
        );
        providers.registration_order.push(index_type);
        Ok(())
    }

    pub(super) fn executor(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        association: &worth_query_installation::facade::WorthQueryInstalledManagedIndexParent,
        allocation: &Arc<
            crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation,
        >,
    ) -> Result<
        Option<Arc<dyn ErasedIndexExecutor<S>>>,
        worth_relational::facade::transactions::RelationalExecutionDenialCause,
    > {
        use crate::domain_computation::primary_graph::application_query::one_shot::outcome::material::MaterialCompletion;
        use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
        let work = self
            .providers
            .entries
            .len()
            .checked_add(1)
            .and_then(|n| n.checked_mul(size_of::<ConfiguredProvider<S>>()))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Cause::WorkCounterOverflow)?;
        MaterialCompletion::new(execution, allocation).work(work)?;
        Ok(self
            .in_registration_order()
            .find(|entry| entry.executor.admits(association))
            .map(|entry| Arc::clone(&entry.executor)))
    }

    pub(super) fn get<I>(&self) -> Option<Arc<dyn InstalledIndexProvider<S, I>>>
    where
        I: ApplicationManagedIndexBinding<S>,
    {
        self.providers
            .entries
            .get(&EqualityTypeId::of::<I>())?
            .adapter
            .downcast_ref::<Arc<dyn InstalledIndexProvider<S, I>>>()
            .map(Arc::clone)
    }

    pub(super) fn get_admitted<I>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        allocation: &Arc<
            crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation,
        >,
    ) -> Result<
        Option<Arc<dyn InstalledIndexProvider<S, I>>>,
        worth_relational::facade::transactions::RelationalExecutionDenialCause,
    >
    where
        I: ApplicationManagedIndexBinding<S>,
    {
        use crate::domain_computation::primary_graph::application_query::one_shot::outcome::material::MaterialCompletion;
        use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
        let work = self
            .providers
            .entries
            .len()
            .checked_add(1)
            .and_then(|n| n.checked_mul(size_of::<EqualityTypeId>().checked_add(1)?))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Cause::WorkCounterOverflow)?;
        MaterialCompletion::new(execution, allocation).work(work)?;
        Ok(self.get::<I>())
    }
}

fn configured_in<S, I>(adapter: &dyn Any, schema: &WorthQueryInstalledApplicationSchema<S>) -> bool
where
    S: ApplicationSchema + 'static,
    I: ApplicationManagedIndexBinding<S>,
{
    adapter
        .downcast_ref::<Arc<dyn InstalledIndexProvider<S, I>>>()
        .is_some_and(|actual| actual.installed().is_installed_in(schema))
}

mod court_doors_for_test;
