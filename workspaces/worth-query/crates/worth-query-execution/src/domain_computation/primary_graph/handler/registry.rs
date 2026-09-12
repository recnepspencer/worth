use std::any::{Any, TypeId};
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_declaration::facade::application_operation::ApplicationCandidateRequirements;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_installation::facade::{
    ApplicationSchema, ApplicationSchemaBindingIdentity,
    WorthQueryInstalledApplicationMutationBinding, WorthQueryInstalledApplicationSchema,
};

use super::super::{
    application_entry::mutation::OperationHandler, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind as DenialKind,
};

#[derive(Clone)]
struct PendingMutationHandler {
    identity: String,
    handler_identity: String,
    decision_type: TypeId,
    denial_type: TypeId,
    value: Arc<dyn Any + Send + Sync>,
    candidates: ApplicationCandidateRequirements,
}

struct TypedHandlerSlot<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    handler: Arc<dyn OperationHandler<Schema, Binding>>,
}

pub(in crate::domain_computation::primary_graph) struct PendingMutationHandlerRegistry<Schema> {
    entries: BTreeMap<String, PendingMutationHandler>,
    _schema: PhantomData<fn() -> Schema>,
}

pub(in crate::domain_computation::primary_graph) struct InstalledMutationHandlerRegistry<Schema> {
    entries: BTreeMap<String, PendingMutationHandler>,
    _schema: PhantomData<fn() -> Schema>,
}

impl<Schema> Default for PendingMutationHandlerRegistry<Schema> {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
            _schema: PhantomData,
        }
    }
}

impl<Schema> WorthQueryPrimaryGraphBootstrap<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn install_handler<Binding, Handler>(
        &mut self,
        installed: &WorthQueryInstalledApplicationMutationBinding<Schema, Binding>,
        handler: Handler,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        Binding: ApplicationMutationBinding<Schema>,
        Handler: OperationHandler<Schema, Binding>,
    {
        self.mutation_handlers
            .register(self.graph.binding_identity(), installed, handler)
    }
}

impl<Schema> PendingMutationHandlerRegistry<Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph) fn register<Binding, Handler>(
        &mut self,
        binding_identity: &ApplicationSchemaBindingIdentity,
        installed: &WorthQueryInstalledApplicationMutationBinding<Schema, Binding>,
        handler: Handler,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        Binding: ApplicationMutationBinding<Schema>,
        Handler: OperationHandler<Schema, Binding>,
    {
        if installed.operation().binding_identity() != binding_identity {
            return Err(denial(
                DenialKind::ForeignMutationHandler,
                Binding::IDENTITY,
            ));
        }
        if installed.identity() != Binding::IDENTITY
            || installed.handler_identity() != Binding::HANDLER_IDENTITY
        {
            return Err(denial(
                DenialKind::MutationHandlerMeaningMismatch,
                Binding::IDENTITY,
            ));
        }
        if self.entries.contains_key(Binding::IDENTITY) {
            return Err(denial(
                DenialKind::DuplicateMutationHandler,
                Binding::IDENTITY,
            ));
        }
        self.entries.insert(
            Binding::IDENTITY.to_owned(),
            PendingMutationHandler {
                identity: Binding::IDENTITY.to_owned(),
                handler_identity: Binding::HANDLER_IDENTITY.to_owned(),
                decision_type: TypeId::of::<Binding::Decision>(),
                denial_type: TypeId::of::<Binding::Denial>(),
                value: Arc::new(TypedHandlerSlot::<Schema, Binding> {
                    handler: Arc::new(handler),
                }),
                candidates: installed.candidate_requirements(),
            },
        );
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph) fn seal(
        &self,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        binding_identity: &ApplicationSchemaBindingIdentity,
    ) -> Result<InstalledMutationHandlerRegistry<Schema>, WorthQueryPrimaryGraphInstallationDenial>
    {
        if &installed_schema.binding_identity() != binding_identity {
            return Err(denial(
                DenialKind::ForeignMutationHandler,
                "installed schema",
            ));
        }
        let expected_count = installed_schema
            .installed_mutation_binding_inventory()
            .len();
        if expected_count != self.entries.len() {
            return Err(denial(
                DenialKind::MissingMutationHandler,
                installed_schema
                    .installed_mutation_binding_inventory()
                    .find(|descriptor| !self.entries.contains_key(descriptor.identity()))
                    .map_or("mutation handler inventory", |descriptor| {
                        descriptor.identity()
                    }),
            ));
        }
        for descriptor in installed_schema.installed_mutation_binding_inventory() {
            let entry = self
                .entries
                .get(descriptor.identity())
                .ok_or_else(|| denial(DenialKind::MissingMutationHandler, descriptor.identity()))?;
            if entry.identity != descriptor.identity()
                || entry.handler_identity != descriptor.handler().identity()
                || entry.decision_type != descriptor.handler().decision_type()
                || entry.denial_type != descriptor.handler().denial_type()
            {
                return Err(denial(
                    DenialKind::MutationHandlerMeaningMismatch,
                    descriptor.identity(),
                ));
            }
        }
        Ok(InstalledMutationHandlerRegistry {
            entries: self.entries.clone(),
            _schema: PhantomData,
        })
    }
}

impl<Schema> InstalledMutationHandlerRegistry<Schema>
where
    Schema: ApplicationSchema,
{
    fn get<Binding>(
        &self,
    ) -> Option<(
        Arc<dyn OperationHandler<Schema, Binding>>,
        ApplicationCandidateRequirements,
    )>
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        self.entries
            .get(Binding::IDENTITY)
            .filter(|entry| entry.handler_identity == Binding::HANDLER_IDENTITY)
            .and_then(|entry| {
                entry
                    .value
                    .downcast_ref::<TypedHandlerSlot<Schema, Binding>>()
                    .map(|slot| (Arc::clone(&slot.handler), entry.candidates))
            })
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph) fn mutation_handler_for_attempt<Binding>(
        &self,
    ) -> (
        Arc<dyn OperationHandler<Schema, Binding>>,
        ApplicationCandidateRequirements,
    )
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        self.mutation_handlers
            .get::<Binding>()
            .expect("runtime publication validated the exact mutation handler inventory")
    }
}

fn denial(
    kind: DenialKind,
    subject: impl Into<String>,
) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(kind, subject)
}
