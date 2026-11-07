//! Finalize lineage and recovery only after service publication has returned.
use super::*;
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
        decoded_checkpoint,
        interim_continuation,
    } = completion.consume();
    let execution = phase.execution_request();
    application.installed_producers = producers;
    application.installed_conditionals = installed_conditionals;
    application
        .runtime
        .primary_provider
        .graph
        .output_lineage
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .bind_implementation_build(build_identity);
    if let Some(support) = admitted_program_support {
        application
            .runtime
            .product_runtime
            .activations
            .require_program_coordination();
        application.program_support = Some(
            crate::domain_computation::primary_graph::program_occurrence::WorthQueryInstalledProgramSupport::installed(
                support.roster,
                activation,
            ),
        );
    }
    if let Some(checkpoint) = decoded_checkpoint {
        let origin = checkpoint.origin.as_ref().ok_or_else(|| Denial::Graph(
            WorthQueryPrimaryGraphInstallationDenial::new(crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::InterimCheckpointCaptureOriginMissing,
                "interim checkpoint capture origin was lost")))?;
        application
            .restore_interim_registry(origin, interim_continuation.as_ref(), &execution)
            .map_err(Denial::Graph)?;
        application.runtime.recovered_outputs =
            crate::domain_computation::primary_graph::application_output_demand::WorthQueryRecoveredOutputs::from_checkpoint(
                checkpoint,
                &execution,
                |accepted| {
                    application.installed_producers.readmit_checkpoint_output(
                        &application.installed_schema,
                        accepted,
                        &execution,
                    )
                },
            )
            .map_err(Denial::Graph)?;
    }
    crate::domain_computation::primary_graph::application_installation::checkpoint_lineage::restore(&application, &execution).map_err(Denial::Graph)?;
    Ok(application)
}
