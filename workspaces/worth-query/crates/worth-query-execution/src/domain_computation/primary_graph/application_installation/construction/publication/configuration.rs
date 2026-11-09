//! Configure executors before retaining a plan for real service publication.
use super::*;
#[inline(never)]
pub(super) fn prepare<Schema: ApplicationSchemaComposition>(
    phase: &crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<'_>,
    prepared: BootstrapPublicationStorage<PreparedMemoryApplication<Schema>>,
) -> Result<
    BootstrapPublicationStorage<ConfiguredMemoryService<Schema>>,
    WorthQueryApplicationPreparationDenial,
> {
    use WorthQueryApplicationPreparationDenial as Denial;
    let storage = BootstrapPublicationReservation::prepare(&phase.execution_request())
        .map_err(Denial::Graph)?;
    let completion = BootstrapPublicationReservation::prepare(&phase.execution_request())
        .map_err(Denial::Graph)?;
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
        interim_continuation,
    } = prepared.consume();
    let (route, installed_conditionals) = if conditionals.is_empty() && producers.is_empty() {
        let route = BootstrapPublicationReservation::prepare(&phase.execution_request())
            .map_err(Denial::Graph)?;
        (
            MemoryServiceRoute::Plain(route.fill(PlainMemoryService {
                graph,
                runtime,
                authority,
                installed,
                conditional_budget,
                authorization_time_source,
            })),
            Default::default(),
        )
    } else {
        let route = BootstrapPublicationReservation::prepare(&phase.execution_request())
            .map_err(Denial::Graph)?;
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
            .map_err(|refused| Denial::ConditionalPublication(refused.into_cause()))?;
        (
            MemoryServiceRoute::Conditional(route.fill(publication)),
            installed_conditionals,
        )
    };
    let completion = completion.fill(MemoryPublicationCompletion {
        producers,
        build_identity,
        admitted_program_support,
        activation,
        decoded_checkpoint,
        interim_continuation,
    });
    Ok(storage.fill(ConfiguredMemoryService {
        route,
        installed_conditionals,
        completion,
    }))
}
