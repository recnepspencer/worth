use crate::equality_type_id::EqualityTypeId as TypeId;
use std::any::Any;
use std::collections::{BTreeMap, HashMap};
use std::marker::PhantomData;
use std::sync::Arc;
mod operation_body;
mod registration;
use registration::HandlerOrigin;

use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_program::{
    ApplicationComputationExecution, DeterminismContract,
};
use worth_query_installation::facade::{
    ApplicationSchema, ApplicationSchemaBindingIdentity,
    WorthQueryInstalledApplicationMutationBinding, WorthQueryInstalledApplicationSchema,
};

mod collections;
mod computations;
use computations::InstalledManagedComputationOwner;

use super::super::{
    application_entry::mutation::{MutationHandlerExecutionDenial, OperationHandler},
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind as DenialKind,
};

#[derive(Clone)]
struct PendingMutationHandler {
    origin: HandlerOrigin,
    identity: String,
    handler_identity: String,
    decision_type: TypeId,
    denial_type: TypeId,
    value: Arc<dyn Any + Send + Sync>,
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
    computations: HashMap<TypeId, InstalledManagedComputationOwner>,
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
            computations: HashMap::new(),
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
        let handled = || {
            installed_schema
                .installed_mutation_binding_inventory()
                .filter(|descriptor| {
                    !descriptor.workflow_control()
                        || self
                            .entries
                            .get(descriptor.identity())
                            .is_some_and(|entry| entry.origin == HandlerOrigin::CompiledBody)
                })
        };
        if handled().count() != self.entries.len() {
            return Err(denial(
                DenialKind::MissingMutationHandler,
                handled()
                    .find(|descriptor| !self.entries.contains_key(descriptor.identity()))
                    .map_or("mutation handler inventory", |descriptor| {
                        descriptor.identity()
                    }),
            ));
        }
        for descriptor in handled() {
            let entry = self
                .entries
                .get(descriptor.identity())
                .ok_or_else(|| denial(DenialKind::MissingMutationHandler, descriptor.identity()))?;
            if entry.identity != descriptor.identity()
                || entry.handler_identity != descriptor.handler().identity()
                || entry.decision_type != TypeId::from(descriptor.handler().decision_type())
                || entry.denial_type != TypeId::from(descriptor.handler().denial_type())
            {
                return Err(denial(
                    DenialKind::MutationHandlerMeaningMismatch,
                    descriptor.identity(),
                ));
            }
        }
        self.require_operation_body_coverage(installed_schema)?;
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
    fn get<Binding>(&self) -> Option<Arc<dyn OperationHandler<Schema, Binding>>>
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
                    .map(|slot| Arc::clone(&slot.handler))
            })
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    /// The installed handler for `Binding`.
    ///
    /// A workflow control binding has no handler and is refused as
    /// `WorkflowControl`; a binding this runtime installed no handler for is
    /// refused as `HandlerNotInstalled`. Both refusals are deterministic, so
    /// no retry of the same request can succeed.
    pub(in crate::domain_computation::primary_graph) fn mutation_handler_for_attempt<Binding>(
        &self,
    ) -> Result<Arc<dyn OperationHandler<Schema, Binding>>, MutationHandlerExecutionDenial>
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        if Binding::WORKFLOW_CONTROL {
            return Err(MutationHandlerExecutionDenial::WorkflowControl);
        }
        self.mutation_handlers
            .get::<Binding>()
            .ok_or(MutationHandlerExecutionDenial::HandlerNotInstalled)
    }
}

fn denial(
    kind: DenialKind,
    subject: impl Into<String>,
) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(kind, subject)
}
