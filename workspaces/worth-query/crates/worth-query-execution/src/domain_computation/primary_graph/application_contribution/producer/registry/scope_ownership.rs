//! Install ownership only when the producer's actual source is its entity scope.
use super::*;
use worth_query_declaration::facade::{
    application_operation::*, application_query::ApplicationQueryScopeResolutionMode,
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationMutationBinding, WorthQueryInstalledApplicationQueryBinding,
};

pub(super) fn validate<S, B>(
    mutation: &WorthQueryInstalledApplicationMutationBinding<S, B::Operation>,
    source: &WorthQueryInstalledApplicationQueryBinding<
        S,
        super::super::ProducerSourceBinding<S, B>,
    >,
) -> Result<Option<String>, WorthQueryPrimaryGraphInstallationDenial>
where
    S: ApplicationSchema,
    B: WorthQueryApplicationProducerBinding<S>,
{
    use worth_foundational::facade::ProducerPublicationOwnership;
    if B::OWNERSHIP == ProducerPublicationOwnership::Independent {
        return Ok(None);
    }
    let query = source.query();
    let graph = query.read_graph();
    let scope = source.scope_contract();
    let operation = mutation.scope_contract();
    let same_entity_source = graph.root_entity() == query.scope_entity()
        && graph.root_paths().is_empty()
        && graph.component_selection().is_none()
        && graph.orbit().is_none()
        && graph.managed_index_selection().is_none()
        && scope.mode() == ApplicationQueryScopeResolutionMode::InputField
        && operation.mode() == ApplicationMutationScopeResolutionMode::InputField
        && scope.field() == operation.field();
    if !same_entity_source {
        return Err(denial(
            DenialKind::ProducerOwnershipRequiresEntitySourceScope,
            B::IDENTITY,
        ));
    }
    Ok(Some(query.scope_entity().to_owned()))
}

impl<S: ApplicationSchema> WorthQueryInstalledApplicationProducerRegistry<S> {
    pub(in crate::domain_computation::primary_graph) fn source_scope_owned_producers(
        &self,
    ) -> impl Iterator<Item = (&str, &str)> {
        self.entries.values().filter_map(|entry| {
            entry
                .declaration
                .owned_scope_entity
                .as_deref()
                .map(|entity| (entry.declaration.identity.as_str(), entity))
        })
    }
}

impl<S: ApplicationSchema> PendingProducerRegistry<S> {
    pub(in crate::domain_computation::primary_graph) fn owns_source_scopes(&self) -> bool {
        self.declared
            .values()
            .any(|entry| entry.owned_scope_entity.is_some())
    }
}

pub fn require_family_agreement(
    declarations: &std::collections::BTreeMap<String, DeclaredProducerBinding>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let mut families = std::collections::BTreeMap::new();
    for declaration in declarations.values() {
        let meaning = (
            declaration.ownership,
            declaration.owned_scope_entity.as_deref(),
        );
        if let Some((prior, identity)) =
            families.insert(&declaration.output_family, (meaning, &declaration.identity))
            && prior != meaning
        {
            return Err(denial(
                DenialKind::ProducerFamilyOwnershipMismatch,
                format!("{identity} / {}", declaration.identity),
            ));
        }
    }
    Ok(())
}
