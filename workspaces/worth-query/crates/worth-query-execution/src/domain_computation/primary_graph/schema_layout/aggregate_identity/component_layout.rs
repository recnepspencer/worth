//! Retained component metadata comes from the same installed Native declaration.
use super::*;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct ComponentSetLayout {
    pub member: Arc<str>,
    pub member_kind: KindId,
    pub definition: AggregateDefinitionIdentity,
    pub sources: Arc<[(Arc<str>, KindId)]>,
}

pub(super) fn retain(
    definition: &definitions::Definition<'_>,
    member_kind: KindId,
) -> Option<(String, ComponentSetLayout)> {
    let meaning = definition.component?;
    let sources = meaning
        .relations()
        .iter()
        .zip(definition.source.relations())
        .map(|(name, kind)| (Arc::from(name.as_str()), *kind))
        .collect();
    Some((
        meaning.name().to_owned(),
        ComponentSetLayout {
            member: Arc::from(meaning.member()),
            member_kind,
            definition: definition.identity,
            sources,
        },
    ))
}
