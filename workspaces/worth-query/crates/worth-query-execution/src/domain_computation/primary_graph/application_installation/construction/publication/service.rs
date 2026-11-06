//! Publish the configured service with no reconstruction state on its stack.
use super::*;
#[inline(never)]
pub(super) fn publish<S: ApplicationSchemaComposition>(
    phase: &crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<'_>,
    prepared: BootstrapPublicationStorage<ConfiguredMemoryService<S>>,
) -> Result<
    BootstrapPublicationStorage<PublishedMemoryService<S>>,
    WorthQueryInMemoryApplicationDenial,
> {
    use WorthQueryInMemoryApplicationDenial as Denial;
    let storage = BootstrapPublicationReservation::prepare(&phase.execution_request())
        .map_err(Denial::Graph)?;
    let ConfiguredMemoryService {
        route,
        installed_conditionals,
        completion,
    } = prepared.consume();
    let application = match route {
        MemoryServiceRoute::Plain(input) => publish_plain(phase, input)?,
        MemoryServiceRoute::Conditional(publication) => {
            publish_conditional(phase, publication, &installed_conditionals)?
        }
    };
    Ok(storage.fill(PublishedMemoryService {
        application,
        installed_conditionals,
        completion,
    }))
}
#[inline(never)]
fn publish_plain<S: ApplicationSchemaComposition>(
    phase: &crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<'_>,
    input: BootstrapPublicationStorage<PlainMemoryService<S>>,
) -> Result<WorthQueryPrimaryGraphApplicationRuntime<S>, WorthQueryInMemoryApplicationDenial> {
    use WorthQueryInMemoryApplicationDenial as Denial;
    let PlainMemoryService {
        graph,
        runtime,
        authority,
        installed,
        conditional_budget,
        authorization_time_source,
    } = input.consume();
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
    Ok(application)
}

#[inline(never)]
fn publish_conditional<S: ApplicationSchemaComposition>(
    phase: &crate::domain_computation::primary_graph::WorthQueryBootstrapAdvancementPhase<'_>,
    input: BootstrapPublicationStorage<crate::domain_computation::primary_graph::conditional_operation::WorthQueryConditionalApplicationRuntimeInstallation<S>>,
    conditionals: &crate::domain_computation::primary_graph::application_contribution::WorthQueryInstalledApplicationConditionalRegistry<S>,
) -> Result<WorthQueryPrimaryGraphApplicationRuntime<S>, WorthQueryInMemoryApplicationDenial> {
    input
        .consume()
        .publish_contributions(phase, conditionals)
        .map_err(WorthQueryInMemoryApplicationDenial::ConditionalPublication)
}
