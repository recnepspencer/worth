pub(in crate::domain_computation::primary_graph) mod route_binding;
use super::operation_invocation::WorthQueryTemporalOperationExecution;
use super::reconstruction_authority::WorthQueryTemporalReconstructionAccess;
use crate::domain_computation::primary_graph::application_runtime::installation::ApplicationRuntimePublication;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::equality_type_id::EqualityTypeId as TypeId;
use std::{collections::BTreeSet, marker::PhantomData, sync::Arc};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryConditionalNodeLocation,
    WorthQueryHostConditionalPredicateProvider, WorthQueryInstalledTemporalConditionalOperation,
    WorthQueryNamedClock, WorthQueryNamedClockSource,
    WorthQueryPortableApplicationConditionalOperationBinding, WorthQueryTemporalIntentProjector,
};
pub(crate) mod clock_handle;
pub(in crate::domain_computation::primary_graph) use clock_handle::ConditionalClockLease;
pub use clock_handle::WorthQueryConditionalClockHandle;
mod denial;
use denial::foreign_binding_denial;
pub use denial::{
    WorthQueryConditionalRuntimeInstallationDenial,
    WorthQueryConditionalRuntimeInstallationDenialKind,
};
pub(in crate::domain_computation::primary_graph) mod home_program;
mod pending_operation;
mod publication_admission;
pub(in crate::domain_computation::primary_graph) use pending_operation::WorthQueryPendingConditionalOperation;
pub(in crate::domain_computation::primary_graph) use publication_admission::RefusedConditionalRuntimeInstallation;
mod application_binding_scope;
mod definition_authoring;
mod output_producer_installation;
mod output_readiness;
mod temporal_binding;
mod temporal_binding_validation;
type InstalledOutputProducers<Schema> =
    super::super::application_contribution::WorthQueryInstalledApplicationProducerRegistry<Schema>;
struct ApplicationConditionalBindingScope {
    binding: WorthQueryPortableApplicationConditionalOperationBinding,
    operation_type: TypeId,
    target: WorthQueryConditionalNodeLocation,
    initial_binding_count: usize,
    initial_readiness_count: usize,
    initial_definition_count: usize,
    required_producers: Vec<String>,
}
/// Builder for an application runtime with conditional operations.
///
/// Bind each temporal operation and output readiness, then `publish` to get
/// the runtime. Nothing is visible until publication succeeds.
pub struct WorthQueryConditionalApplicationRuntimeInstallation<Schema> {
    publication: ApplicationRuntimePublication<Schema>,
    output_producers: InstalledOutputProducers<Schema>,
    binding_identities: BTreeSet<Arc<str>>,
    bindings: Vec<Box<dyn WorthQueryPendingConditionalOperation<Schema>>>,
    definitions: Vec<Box<dyn super::PendingDefinitionDeclaration>>,
    output_readiness:
        Vec<Box<dyn super::super::application_contribution::PendingOutputReadiness<Schema>>>,
    application_binding_scope: Option<ApplicationConditionalBindingScope>,
}
impl<Schema> WorthQueryConditionalApplicationRuntimeInstallation<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// Admission witness for validation; the staged installation retains facts only.
    pub(in crate::domain_computation::primary_graph) fn new(
        _phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        publication: ApplicationRuntimePublication<Schema>,
    ) -> Result<Self, RefusedConditionalRuntimeInstallation<Schema>> {
        if let Err(cause) =
            super::super::application_runtime::installation::validate_application_schema(
                &publication.runtime,
                &publication.installed_schema,
            )
        {
            return Err(RefusedConditionalRuntimeInstallation {
                publication,
                cause: super::publication::publication_denial(cause),
            });
        }
        Ok(Self {
            publication,
            output_producers: Default::default(),
            binding_identities: BTreeSet::new(),
            bindings: Vec::new(),
            definitions: Vec::new(),
            output_readiness: Vec::new(),
            application_binding_scope: None,
        })
    }
    pub fn publish(
        self,
        phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    ) -> Result<
        WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        WorthQueryConditionalRuntimeInstallationDenial,
    > {
        // Direct native temporal installation binds each portable node once.
        let expected = self
            .publication
            .runtime
            .installed_packages()
            .installed_conditional_node_count_for_schema(
                self.publication.installed_schema.owner(),
                self.publication.installed_schema.schema_name(),
            );
        self.publish_with_inventory(phase, expected)
    }

    pub(in crate::domain_computation::primary_graph) fn publish_contributions(
        self,
        phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        conditionals: &super::super::application_contribution::WorthQueryInstalledApplicationConditionalRegistry<Schema>,
    ) -> Result<
        WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        WorthQueryConditionalRuntimeInstallationDenial,
    > {
        // Issued only after exact configuration/declaration matching and each
        // singleton binding scope's successful one-route completion. Several
        // such bindings may instantiate one identical portable node contract.
        self.publish_with_inventory(phase, conditionals.binding_count())
    }

    fn publish_with_inventory(
        self,
        phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        expected: usize,
    ) -> Result<
        WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        WorthQueryConditionalRuntimeInstallationDenial,
    > {
        super::super::application_runtime::installation::publish_application_runtime_with_conditionals(phase,
            self.publication,
            self.bindings,
            self.definitions,
            self.output_readiness,
            expected,
            &self.output_producers,
        )
    }
}
