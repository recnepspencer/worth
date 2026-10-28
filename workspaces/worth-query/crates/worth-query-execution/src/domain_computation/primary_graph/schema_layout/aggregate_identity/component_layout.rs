//! Retained component metadata comes from the same installed Native declaration.
use super::*;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct ComponentSetLayout {
    pub member: Arc<str>,
    pub member_domain: worth_relational::facade::schema::NativeMemberDomain,
    pub source: Option<worth_relational::facade::schema::DerivedRelationHandle>,
    pub carriers: Arc<[(Arc<str>, KindId)]>,
    pub member_kind: KindId,
    pub membership_kind: KindId,
    pub definition: AggregateDefinitionIdentity,
    pub sources: Arc<[(Arc<str>, KindId)]>,
}

pub(super) fn retain(
    definition: &definitions::Definition<'_>,
    member_kind: KindId,
    membership_kind: KindId,
    registry: &RelationalSchemaRegistry,
    entities: &BTreeMap<String, KindId>,
    relations: &BTreeMap<String, KindId>,
) -> Result<Option<(String, ComponentSetLayout)>, WorthQueryPrimaryGraphInstallationDenial> {
    let Some(meaning) = definition.component.as_ref() else {
        return Ok(None);
    };
    let mut carriers = BTreeMap::new();
    let sources = match meaning {
        definitions::ComponentMeaning::Stored(meaning) => meaning
            .relations()
            .iter()
            .zip(
                definition
                    .source
                    .stored_relations()
                    .expect("stored definition"),
            )
            .map(|(name, kind)| (Arc::from(name.as_str()), *kind))
            .collect(),
        definitions::ComponentMeaning::Derived(_) => {
            let handle = definition
                .native_source
                .as_ref()
                .expect("derived definition");
            let relation_names: BTreeMap<_, _> =
                relations.iter().map(|(name, kind)| (*kind, name)).collect();
            let entity_names: BTreeMap<_, _> =
                entities.iter().map(|(name, kind)| (*kind, name)).collect();
            for domain in handle.projected_domains() {
                let actual = registry
                    .projected_member_domain(domain)
                    .ok_or_else(|| super::super::invalid_member(&domain.name))?;
                let name = entity_names
                    .get(&actual.parts().carrier)
                    .ok_or_else(|| super::super::invalid_member(&domain.name))?;
                carriers.insert(Arc::from(name.as_str()), actual.parts().carrier);
            }
            handle
                .physical_sources()
                .iter()
                .map(|(kind, _, _)| {
                    Ok((
                        Arc::from(
                            relation_names
                                .get(kind)
                                .ok_or_else(|| super::super::invalid_member(meaning.name()))?
                                .as_str(),
                        ),
                        *kind,
                    ))
                })
                .collect::<Result<Arc<[_]>, WorthQueryPrimaryGraphInstallationDenial>>()?
        }
    };
    let member_domain = definition
        .native_source
        .as_ref()
        .map(|source| source.parts().from.clone())
        .unwrap_or(worth_relational::facade::schema::NativeMemberDomain::Entity(member_kind));
    Ok(Some((
        meaning.name().to_owned(),
        ComponentSetLayout {
            member: Arc::from(definition.member.name()),
            member_domain,
            source: definition.native_source.clone(),
            carriers: carriers.into_iter().collect(),
            member_kind,
            membership_kind,
            definition: definition.identity,
            sources,
        },
    )))
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
        let relations = match (&contract.source, &self.source) {
            (AggregateIdentitySource::Components { relations }, None) => relations,
            (AggregateIdentitySource::DerivedComponents { derived_components }, Some(source)) => {
                poll()?;
                return Ok(pin
                    .schema()
                    .derived_relation(derived_components)
                    .is_some_and(|actual| {
                        actual == *source && actual.parts().from == self.member_domain
                    }));
            }
            _ => return Ok(false),
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

impl super::super::WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation::primary_graph) fn component_set(
        &self,
        name: &str,
    ) -> Option<&ComponentSetLayout> {
        self.component_sets.get(name)
    }
}
