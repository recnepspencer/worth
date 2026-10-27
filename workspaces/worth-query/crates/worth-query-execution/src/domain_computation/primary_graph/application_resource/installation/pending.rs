use super::super::provider::WorthQueryApplicationResourceProvider;
use super::clock::{ClockCorrespondence, TypedResourceClock};
use super::*;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial as Denial;
use worth_query_declaration::facade::application_schema::ApplicationSchemaMember;
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationResource, WorthQueryInstalledApplicationSchema,
    WorthQueryNamedClock, WorthQueryNamedClockSource,
};

trait PendingResource<Schema> {
    fn install(
        self: Box<Self>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        builder: &mut worth_runtime_bridge::facade::BridgeConditionalRuntimeBuilder,
        support: &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot,
    ) -> Result<(TypeId, InstalledResourceEntry), Denial>;
}

struct PendingBinding<Schema: ApplicationSchema, Binding: ApplicationResourceBinding<Schema>> {
    installed: WorthQueryInstalledApplicationResource<Schema, Binding>,
    provider:
        Arc<dyn WorthQueryApplicationResourceProvider<Schema, Binding, Role = Binding::Provider>>,
    clock: Arc<dyn ResourceClock>,
}

pub(in crate::domain_computation::primary_graph) struct PendingResourceRegistry<Schema> {
    entries: BTreeMap<String, Box<dyn PendingResource<Schema>>>,
    clock: Option<ClockCorrespondence>,
}

impl<Schema> Default for PendingResourceRegistry<Schema> {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
            clock: None,
        }
    }
}

impl<Schema: ApplicationSchema + 'static> PendingResourceRegistry<Schema> {
    pub(in crate::domain_computation::primary_graph) fn register<Binding, Provider, Clock, Source>(
        &mut self,
        schema: &WorthQueryInstalledApplicationSchema<Schema>,
        owner: &str,
        provider: Provider,
        source: Arc<Source>,
    ) -> Result<(), Denial>
    where
        Binding: ApplicationResourceBinding<Schema>,
        Provider: WorthQueryApplicationResourceProvider<Schema, Binding, Role = Binding::Provider>,
        Clock: WorthQueryNamedClock,
        Source: WorthQueryNamedClockSource<Clock>,
    {
        let installed = schema
            .installed_resource::<Binding>()
            .map_err(|cause| denial(format!("{}: {cause:?}", Binding::IDENTITY)))?;
        let contribution = schema
            .contributions()
            .get(owner)
            .ok_or_else(|| denial(owner))?;
        if !contribution.members().any(|member| {
            matches!(member,
            ApplicationSchemaMember::ApplicationResource { definition }
                if definition == installed.definition())
        }) {
            return Err(denial(
                "resource provider does not belong to this contribution",
            ));
        }
        if self.entries.contains_key(Binding::IDENTITY) {
            return Err(denial("duplicate resource provider"));
        }
        let clock = ClockCorrespondence::of::<Clock, Source>(&source);
        if self
            .clock
            .as_ref()
            .is_some_and(|existing| !existing.matches(&clock))
        {
            return Err(denial("all resource pools in this cell require the same installed clock instance and timeline"));
        }
        self.clock.get_or_insert(clock);
        self.entries.insert(
            Binding::IDENTITY.to_owned(),
            Box::new(PendingBinding::<Schema, Binding> {
                installed,
                provider: Arc::new(provider),
                clock: Arc::new(TypedResourceClock::<Clock, Source>::new(source)),
            }),
        );
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn validate_complete(
        &self,
        schema: &WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), Denial> {
        if schema
            .installed_declaration()
            .members()
            .iter()
            .any(|member| {
                matches!(member,
            ApplicationSchemaMember::ApplicationResource { definition }
                if !self.entries.contains_key(&definition.parts().name))
            })
        {
            return Err(denial("declared resource has no configured provider"));
        }
        Ok(())
    }

    /// The whole batch is the final graph mutation before seal. No unrelated
    /// raw-tick producer may already occupy this same Native temporal cell.
    pub(in crate::domain_computation::primary_graph) fn install(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        builder: &mut worth_runtime_bridge::facade::BridgeConditionalRuntimeBuilder,
        support: &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot,
    ) -> Result<InstalledResourceRegistry<Schema>, Denial> {
        if !self.entries.is_empty() && builder.owned_async_declaration_count() != 0 {
            return Err(denial(
                "resource pools cannot share a cell with raw owned-async clock producers",
            ));
        }
        let mut entries = HashMap::new();
        for pending in self.entries.into_values() {
            let (identity, resource) = pending.install(execution, builder, support)?;
            entries.insert(identity, resource);
        }
        Ok(InstalledResourceRegistry {
            entries,
            marker: PhantomData,
        })
    }
}

impl<Schema: ApplicationSchema + 'static, Binding: ApplicationResourceBinding<Schema>>
    PendingResource<Schema> for PendingBinding<Schema, Binding>
{
    fn install(
        self: Box<Self>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        builder: &mut worth_runtime_bridge::facade::BridgeConditionalRuntimeBuilder,
        support: &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot,
    ) -> Result<(TypeId, InstalledResourceEntry), Denial> {
        let resource = InstalledResourceBinding::install(
            execution,
            builder,
            support,
            self.installed,
            self.provider,
            self.clock,
        )?;
        let storage = Arc::clone(&resource.storage);
        Ok((
            TypeId::of::<Binding>(),
            InstalledResourceEntry {
                binding: Arc::new(resource),
                storage,
            },
        ))
    }
}
