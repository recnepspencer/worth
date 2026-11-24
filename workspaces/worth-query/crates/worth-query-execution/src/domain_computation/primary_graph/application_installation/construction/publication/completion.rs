//! Finalize lineage and recovery only after service publication has returned.
use super::*;
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;
#[inline(never)]
pub(super) fn complete<S: ApplicationSchemaComposition>(
    phase: &crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<'_>,
    published: BootstrapPublicationStorage<PublishedMemoryService<S>>,
) -> Result<WorthQueryPrimaryGraphApplicationRuntime<S>, WorthQueryApplicationPreparationDenial> {
    use WorthQueryApplicationPreparationDenial as Denial;
    let PublishedMemoryService {
        application,
        installed_conditionals,
        completion,
    } = published.consume();
    let mut application = application.consume();
    let MemoryPublicationCompletion {
        producers,
        build_identity,
        admitted_program_support,
        activation,
    } = completion.consume();
    let execution = phase.execution_request();
    application.inner.installed_producers = producers;
    application.inner.installed_conditionals = installed_conditionals;
    application
        .inner
        .runtime
        .primary_provider_ref()
        .graph_ref()
        .output_lineage_ref()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .bind_implementation_build(build_identity);
    if let Some(support) = admitted_program_support {
        application
            .inner
            .runtime
            .product_runtime_ref()
            .activations_ref()
            .require_program_coordination();
        application.inner.program_support = Some(
            crate::domain_computation::primary_graph::program_occurrence::WorthQueryInstalledProgramSupport::installed(
                support.roster,
                activation,
            ),
        );
    }
    crate::domain_computation::primary_graph::application_installation::checkpoint_lineage::restore(&application, &execution).map_err(Denial::Graph)?;
    Ok(application)
}
