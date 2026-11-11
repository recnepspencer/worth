//! Concrete memory installation successors retain only their own phase state.
use super::*;
use crate::domain_computation::primary_graph::{
    application_checkpoint::DecodedApplicationCheckpoint,
    application_contribution::{
        WorthQueryInstalledApplicationConditionalRegistry,
        WorthQueryInstalledApplicationProducerRegistry,
    },
    application_installation::program_admission::WorthQueryAdmittedProgramSupport,
    conditional_operation::WorthQueryConditionalApplicationRuntimeInstallation,
    program_occurrence::WorthQueryProgramActivationCell,
};
pub(super) struct MemoryPublicationCompletion<S: ApplicationSchemaComposition> {
    pub(super) producers: WorthQueryInstalledApplicationProducerRegistry<S>,
    pub(super) build_identity: worth_foundational::facade::ApplicationBuildIdentity,
    pub(super) admitted_program_support: Option<WorthQueryAdmittedProgramSupport<S>>,
    pub(super) activation: WorthQueryProgramActivationCell,
    pub(super) decoded_checkpoint: Option<DecodedApplicationCheckpoint>,
    pub(super) interim_continuation:
        Option<worth_relational::facade::durability::NativeInterimBootstrapContinuation>,
}
pub(super) struct PlainMemoryService<S: ApplicationSchemaComposition> {
    pub(super) graph: WorthQueryPrimaryGraphBootstrap<S>,
    pub(super) runtime: crate::domain_computation::execution_runtime::WorthQueryExecutionRuntime,
    pub(super) authority:
        crate::domain_computation::execution_runtime::WorthQueryExecutionInstallationAuthority,
    pub(super) installed: WorthQueryInstalledApplicationSchema<S>,
    pub(super) conditional_budget: worth_signal::facade::runtime::SignalConditionalEvaluationBudget,
    pub(super) authorization_time_source:
        Option<Box<dyn crate::domain_computation::runtime_time::WorthQueryRuntimeTimeSource>>,
}
pub(super) enum MemoryServiceRoute<S: ApplicationSchemaComposition> {
    Plain(BootstrapPublicationStorage<PlainMemoryService<S>>),
    Conditional(
        BootstrapPublicationStorage<WorthQueryConditionalApplicationRuntimeInstallation<S>>,
    ),
}
pub(super) struct ConfiguredMemoryService<S: ApplicationSchemaComposition> {
    pub(super) route: MemoryServiceRoute<S>,
    pub(super) installed_conditionals: WorthQueryInstalledApplicationConditionalRegistry<S>,
    pub(super) completion: BootstrapPublicationStorage<MemoryPublicationCompletion<S>>,
}
pub(super) struct PublishedMemoryService<S: ApplicationSchemaComposition> {
    pub(super) application:
        BootstrapPublicationStorage<WorthQueryPrimaryGraphApplicationRuntime<S>>,
    pub(super) installed_conditionals: WorthQueryInstalledApplicationConditionalRegistry<S>,
    pub(super) completion: BootstrapPublicationStorage<MemoryPublicationCompletion<S>>,
}
