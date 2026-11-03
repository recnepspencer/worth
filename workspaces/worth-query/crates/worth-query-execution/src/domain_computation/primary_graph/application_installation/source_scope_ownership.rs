//! Bind the installed producer declarations to the active Native publication handle.
use super::super::{
    application_contribution::WorthQueryInstalledApplicationProducerRegistry,
    WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial as Denial,
    WorthQueryPrimaryGraphInstallationDenialKind as Kind,
};
use worth_query_installation::facade::ApplicationSchema;

pub(super) fn seal<S: ApplicationSchema>(
    graph: &mut WorthQueryPrimaryGraphBootstrap<S>,
    producers: &WorthQueryInstalledApplicationProducerRegistry<S>,
) -> Result<(), Denial> {
    let declarations = producers
        .source_scope_owned_producers()
        .map(|(producer, entity)| {
            graph
                .graph
                .layout
                .entity_kind(entity)
                .map(|kind| (producer.to_owned(), kind))
                .ok_or_else(|| {
                    Denial::new(Kind::ProducerOwnershipRequiresEntitySourceScope, producer)
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if declarations.is_empty() {
        return Ok(());
    }
    let publisher = graph.graph.application_publication.take().ok_or_else(|| {
        Denial::new(
            Kind::ProducerOwnershipRequiresEntitySourceScope,
            declarations[0].0.clone(),
        )
    })?;
    let publisher = std::sync::Arc::try_unwrap(publisher).map_err(|_| {
        Denial::new(
            Kind::AlreadyInstalled,
            "source-scope ownership must be sealed before publisher sharing",
        )
    })?;
    graph.graph.application_publication = Some(std::sync::Arc::new(
        super::super::root::application_publication::ApplicationPublication::recovered(
            publisher.owner,
            publisher
                .native
                .with_source_scope_owned_producers(declarations),
        ),
    ));
    Ok(())
}
