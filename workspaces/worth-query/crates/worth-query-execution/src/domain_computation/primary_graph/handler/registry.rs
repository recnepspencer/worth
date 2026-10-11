use std::any::{Any, TypeId};
use std::collections::{BTreeMap, HashMap};
use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_declaration::facade::application_operation::ApplicationCandidateRequirements;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_program::{
    ApplicationComputationExecution, DeterminismContract,
};
use worth_query_installation::facade::{
    ApplicationSchema, ApplicationSchemaBindingIdentity,
    WorthQueryInstalledApplicationMutationBinding, WorthQueryInstalledApplicationSchema,
};

use super::super::{
    application_entry::mutation::{MutationHandlerExecutionDenial, OperationHandler},
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenial,
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

#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) struct InstalledManagedComputationOwner {
    pub feature_type: TypeId,
    pub computation_type: TypeId,
    pub output_artifact_type: TypeId,
}

pub(in crate::domain_computation::primary_graph) struct PendingMutationHandlerRegistry<Schema> {
    entries: BTreeMap<String, PendingMutationHandler>,
    computations: HashMap<TypeId, InstalledManagedComputationOwner>,
    _schema: PhantomData<fn() -> Schema>,
}

pub(in crate::domain_computation::primary_graph) struct InstalledMutationHandlerRegistry<Schema> {
    entries: BTreeMap<String, PendingMutationHandler>,
    computations: HashMap<TypeId, InstalledManagedComputationOwner>,
    _schema: PhantomData<fn() -> Schema>,
}

#[derive(Clone, Copy)]
struct ManagedComputationExpectation {
    composition_instance: &'static str,
    feature_identity: &'static str,
    identity: &'static str,
    feature_type: TypeId,
    computation_type: TypeId,
    output_artifact_type: TypeId,
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
    /// Records the one owner of a managed computation. `binding` is the
    /// execution posture the owner's binding serves: the declared posture must
    /// be that one. A declared equivalence predicate is refused because no
    /// installed predicate is visible here.
    pub(in crate::domain_computation::primary_graph) fn record_computation_owner<
        Feature,
        Computation,
    >(
        &mut self,
        binding: ApplicationComputationExecution,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>
    where
        Feature: worth_query_declaration::facade::application_program::ApplicationFeature<Schema>,
        Computation:
            worth_query_declaration::facade::application_program::ApplicationManagedComputation<
                Schema,
                Feature,
            >,
    {
        if Computation::EXECUTION != binding {
            return Err(denial(
                DenialKind::ManagedComputationOwnerBindingMismatch,
                Computation::IDENTITY,
            ));
        }
        if let DeterminismContract::ContractEquivalent(_) = Computation::DETERMINISM {
            return Err(denial(
                DenialKind::ManagedComputationEquivalenceUnavailable,
                Computation::IDENTITY,
            ));
        }
        let computation_type = TypeId::of::<Computation>();
        if self.computations.contains_key(&computation_type) {
            return Err(denial(
                DenialKind::DuplicateManagedComputationOwner,
                Computation::IDENTITY,
            ));
        }
        self.computations.insert(
            computation_type,
            InstalledManagedComputationOwner {
                feature_type: TypeId::of::<Feature>(),
                computation_type,
                output_artifact_type: TypeId::of::<Computation::Output>(),
            },
        );
        Ok(())
    }

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
        if Binding::WORKFLOW_CONTROL {
            return Err(denial(
                DenialKind::WorkflowControlHandler,
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
        let handled = || {
            installed_schema
                .installed_mutation_binding_inventory()
                .filter(|descriptor| !descriptor.workflow_control())
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
            computations: self.computations.clone(),
            _schema: PhantomData,
        })
    }
}

impl<Schema> InstalledMutationHandlerRegistry<Schema> {
    pub(in crate::domain_computation::primary_graph) fn validate_managed_computations(
        &self,
        features: &[worth_query_declaration::facade::application_program::ApplicationFeatureDeclaration],
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        let mut declared = Vec::new();
        let mut positions = HashMap::<TypeId, usize>::new();
        for (feature, computation) in features.iter().flat_map(|feature| {
            feature
                .managed_computations()
                .iter()
                .map(move |computation| (feature, computation))
        }) {
            let entry = (
                computation.computation_type(),
                ManagedComputationExpectation {
                    composition_instance: feature.composition_instance(),
                    feature_identity: feature.identity(),
                    identity: computation.identity(),
                    feature_type: computation.feature_type(),
                    computation_type: computation.computation_type(),
                    output_artifact_type: computation.output_artifact_type(),
                },
            );
            if let Some(position) = positions.get(&entry.0) {
                declared[*position] = entry;
            } else {
                positions.insert(entry.0, declared.len());
                declared.push(entry);
            }
        }
        validate_managed_computation_inventory(&self.computations, &declared)
    }
}

fn validate_managed_computation_inventory(
    installed: &HashMap<TypeId, InstalledManagedComputationOwner>,
    declared: &[(TypeId, ManagedComputationExpectation)],
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let mut computations: Vec<_> = declared.iter().collect();
    computations.sort_by_key(|(_, computation)| {
        (
            computation.composition_instance,
            computation.feature_identity,
            computation.identity,
        )
    });
    for (computation_type, computation) in computations {
        let owner = installed.get(computation_type).ok_or_else(|| {
            denial(
                DenialKind::MissingManagedComputationOwner,
                computation.identity,
            )
        })?;
        if computation.computation_type != *computation_type
            || owner.computation_type != *computation_type
            || owner.feature_type != computation.feature_type
            || owner.output_artifact_type != computation.output_artifact_type
        {
            return Err(denial(
                DenialKind::ManagedComputationOwnerMeaningMismatch,
                computation.identity,
            ));
        }
    }
    if installed.len() != declared.len() {
        return Err(denial(
            DenialKind::ForeignManagedComputationOwner,
            "managed computation owner inventory",
        ));
    }
    Ok(())
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
    /// The installed handler for `Binding`.
    ///
    /// A workflow control binding has no handler and is refused as
    /// `WorkflowControl`; a binding this runtime installed no handler for is
    /// refused as `HandlerNotInstalled`. Both refusals are deterministic, so
    /// no retry of the same request can succeed.
    pub(in crate::domain_computation::primary_graph) fn mutation_handler_for_attempt<Binding>(
        &self,
    ) -> Result<
        (
            Arc<dyn OperationHandler<Schema, Binding>>,
            ApplicationCandidateRequirements,
        ),
        MutationHandlerExecutionDenial,
    >
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

#[cfg(test)]
#[path = "registry_tests.rs"]
mod tests;
