//! Publish the already completed Native bootstrap through its one Query owner.
use super::native_preparation::PreparedMemoryApplication;
use super::*;
use crate::domain_computation::primary_graph::bootstrap::BootstrapPublicationStorage;
pub(super) fn publish<Schema: ApplicationSchemaComposition>(
    phase: &super::super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    prepared: BootstrapPublicationStorage<PreparedMemoryApplication<Schema>>,
) -> Result<WorthQueryPrimaryGraphApplicationRuntime<Schema>, WorthQueryInMemoryApplicationDenial> {
    use WorthQueryInMemoryApplicationDenial as Denial;
    let PreparedMemoryApplication {
        graph,
        runtime,
        authority,
        installed,
        producers,
        conditionals,
        build_identity,
        conditional_budget,
        authorization_time_source,
        admitted_program_support,
        activation,
        decoded_checkpoint,
    } = prepared.consume();
    let execution = phase.execution_request();
    let (mut application, installed_conditionals) =
        if conditionals.is_empty() && producers.is_empty() {
            let application = match authorization_time_source {
                Some(source) => graph.publish_application_runtime_with_authorization_time_source(
                    phase,
                    runtime,
                    authority,
                    installed,
                    conditional_budget,
                    source,
                ),
                None => graph.publish_application_runtime(
                    phase,
                    runtime,
                    authority,
                    installed,
                    conditional_budget,
                ),
            }
            .map_err(Denial::Publication)?;
            (application, Default::default())
        } else {
            let mut publication = match authorization_time_source {
                Some(source) => graph
                    .conditional_application_runtime_installation_with_authorization_time_source(
                        phase,
                        runtime,
                        authority,
                        installed,
                        conditional_budget,
                        source,
                    ),
                None => graph.conditional_application_runtime_installation(
                    phase,
                    runtime,
                    authority,
                    installed,
                    conditional_budget,
                ),
            }
            .map_err(Denial::ConditionalPublication)?;
            publication.install_output_producers(producers.clone());
            let installed_conditionals = conditionals
                .install_all(&producers, &mut publication)
                .map_err(Denial::ConditionalPublication)?;
            let application = publication
                .publish_contributions(phase, &installed_conditionals)
                .map_err(Denial::ConditionalPublication)?;
            (application, installed_conditionals)
        };
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
            super::super::super::program_occurrence::WorthQueryInstalledProgramSupport::installed(
                support.roster,
                activation,
            ),
        );
    }
    if let Some(checkpoint) = decoded_checkpoint {
        application.runtime.recovered_outputs =
            super::super::super::application_output_demand::WorthQueryRecoveredOutputs::from_checkpoint(
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
    super::super::checkpoint_lineage::restore(&application, &execution).map_err(Denial::Graph)?;
    Ok(application)
}
