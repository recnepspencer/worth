//! Retained component metadata comes from the same installed Native declaration.
use super::*;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct ComponentSetLayout {
    pub member: Arc<str>,
    pub member_kind: KindId,
    pub membership_kind: KindId,
    pub definition: AggregateDefinitionIdentity,
    pub sources: Arc<[(Arc<str>, KindId)]>,
}

pub(super) fn retain(
    definition: &definitions::Definition<'_>,
    member_kind: KindId,
    membership_kind: KindId,
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
            membership_kind,
            definition: definition.identity,
            sources,
        },
    ))
}

impl ComponentSetLayout {
    /// The selected root's borrowed Native contract proves existence even when
    /// the visible member universe is empty. Membership still requires Native reads.
    pub(in crate::domain_computation::primary_graph) fn matches_native_definition<E>(
        &self,
        pin: &worth_relational::facade::runtime::RelationalPinnedRead,
        mut poll: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        poll()?;
        let Ok(registration) = pin.schema().relation_registration(self.membership_kind) else {
            return Ok(false);
        };
        let Some(contract) = registration.relation_integrity.aggregate_identity.as_ref() else {
            return Ok(false);
        };
        if contract.definition != self.definition
            || contract.member_kind != self.member_kind
            || contract.membership_kind != self.membership_kind
        {
            return Ok(false);
        }
        let AggregateIdentitySource::Components { relations } = &contract.source else {
            return Ok(false);
        };
        if relations.len() != self.sources.len() {
            return Ok(false);
        }
        for (_, kind) in self.sources.iter() {
            poll()?;
            if relations.binary_search(kind).is_err() {
                return Ok(false);
            }
        }
        poll()?;
        Ok(true)
    }
}
