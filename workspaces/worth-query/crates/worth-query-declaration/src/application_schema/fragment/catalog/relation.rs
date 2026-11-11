//! Outgoing relation declarations retain their source and extension eligibility.
use super::*;
impl FragmentCatalog {
    pub(crate) fn relation<R: super::super::Relation>(
        &mut self,
        entity_path: &str,
        entity: &str,
    ) -> Result<(), Denial> {
        let path = format!("{entity_path}.{}", R::NAME);
        let key = (entity.to_owned(), R::NAME.to_owned());
        let extension = self.extension_paths.contains(entity_path);
        if extension && let Some(first) = self.member_names.get(&key) {
            return Err(Denial::DuplicateExtensionMember {
                first: first.clone(),
                path,
            });
        }
        let cardinality = R::CARDINALITY;
        let positive = |minimum: Option<u64>| minimum.is_some_and(|minimum| minimum > 0);
        let fragment = entity_path.rsplit_once('.').ok_or(Denial::MissingEntity)?.0;
        if extension
            && (positive(cardinality.source_min)
                || ((positive(cardinality.target_min) || positive(cardinality.pair_min))
                    && !self.declares_within::<R::To>(fragment)))
        {
            return Err(Denial::ExtensionRelationMinimum { path });
        }
        self.member_names.insert(key, path);
        let owner = self.owner(entity_path)?;
        let entity_type = owner
            .entities
            .iter()
            .find(|(_, name)| name.as_str() == entity)
            .map(|(key, _)| *key)
            .ok_or(Denial::MissingEntity)?;
        owner.relations.insert(
            (entity_type, TypeId::of::<R>()),
            RelationDefinition {
                name: R::NAME.to_owned(),
                from: entity.to_owned(),
                to: R::To::NAME.to_owned(),
                integrity: super::super::declaration::relation_integrity::<R>(),
            },
        );
        Ok(())
    }
    fn declares_within<E: Entity>(&self, fragment: &str) -> bool {
        self.definitions.values().any(|definition| {
            definition.path.strip_prefix(fragment)
                .is_some_and(|suffix| suffix.is_empty() || suffix.starts_with('.'))
                && definition.entities.get(&TypeId::of::<E>()).map(String::as_str) == Some(E::NAME)
                // Extension imports retain an entity reference, not its declaration.
                && !self.extension_paths.contains(&format!("{}.{}", definition.path, E::NAME))
        })
    }
}
