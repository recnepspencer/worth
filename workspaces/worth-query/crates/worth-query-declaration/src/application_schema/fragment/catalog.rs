//! Compiler-local fragment descriptors travel with the existing schema provenance.

use super::super::{
    ApplicationFieldBindingRecipe, ApplicationRelationIntegrity,
    ApplicationSchemaDeclarationDenial as Denial,
};
use super::one_of::GroupDefinition;
use super::record::Column;
use super::{Aspect, Entity, Fragment, Record, RecordBinding};
use std::any::TypeId;
use std::collections::BTreeMap;
mod record_access;
pub(crate) use record_access::{expand_record_access, RecordAccess};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FragmentCatalog {
    definitions: BTreeMap<TypeId, FragmentDefinition>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FragmentDefinition {
    path: String,
    entities: BTreeMap<TypeId, String>,
    aspects: BTreeMap<(TypeId, TypeId), AspectDefinition>,
    groups: BTreeMap<(TypeId, TypeId), GroupDefinition>,
    relations: BTreeMap<(TypeId, TypeId), RelationDefinition>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AspectDefinition {
    path: String,
    entity: String,
    name: String,
    binding: RecordBinding,
    fields: Vec<RecordFieldDefinition>,
    optional: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordFieldDefinition {
    recipe: ApplicationFieldBindingRecipe,
    writable: bool,
    equality: bool,
    unique: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationDefinition {
    pub name: String,
    pub from: String,
    pub to: String,
    pub integrity: ApplicationRelationIntegrity,
}

impl FragmentCatalog {
    pub fn fragment<F: Fragment>(&self, _: F) -> Option<&FragmentDefinition> {
        self.definitions.get(&TypeId::of::<F>())
    }

    pub(crate) fn register<F: Fragment>(&mut self, path: &str) -> Result<(), Denial> {
        if self.definitions.contains_key(&TypeId::of::<F>()) {
            return Err(Denial::DuplicateFragmentMember {
                path: path.to_owned(),
            });
        }
        self.definitions.insert(
            TypeId::of::<F>(),
            FragmentDefinition {
                path: path.to_owned(),
                entities: BTreeMap::new(),
                aspects: BTreeMap::new(),
                groups: BTreeMap::new(),
                relations: BTreeMap::new(),
            },
        );
        Ok(())
    }

    fn owner(&mut self, entity_path: &str) -> Result<&mut FragmentDefinition, Denial> {
        let (fragment, _) = entity_path.rsplit_once('.').ok_or(Denial::MissingEntity)?;
        self.definitions
            .values_mut()
            .find(|definition| definition.path == fragment)
            .ok_or(Denial::MissingEntity)
    }

    pub(crate) fn entity<E: Entity>(&mut self, entity_path: &str) -> Result<(), Denial> {
        self.owner(entity_path)?
            .entities
            .insert(TypeId::of::<E>(), E::NAME.to_owned());
        Ok(())
    }

    pub(crate) fn aspect<Set: Record<A::Row>, A: Aspect>(
        &mut self,
        entity_path: &str,
        entity: &str,
        columns: &[Column],
        optional: bool,
    ) -> Result<AspectDefinition, Denial> {
        let owner = self.owner(entity_path)?;
        let entity_type = owner
            .entities
            .iter()
            .find(|(_, name)| name.as_str() == entity)
            .map(|(key, _)| *key)
            .ok_or(Denial::MissingEntity)?;
        let fields = columns
            .iter()
            .map(|column| RecordFieldDefinition {
                recipe: (column.recipe)(super::super::ApplicationFieldBindingLocus::new(
                    entity,
                    A::NAME,
                    &column.path,
                )),
                writable: column.writable,
                equality: column.equality,
                unique: column.unique,
            })
            .collect();
        let definition = AspectDefinition {
            path: format!("{entity_path}.{}", A::NAME),
            entity: entity.to_owned(),
            name: A::NAME.to_owned(),
            binding: RecordBinding::record::<Set, A::Row>(),
            fields,
            optional,
        };
        owner
            .aspects
            .insert((entity_type, TypeId::of::<A>()), definition.clone());
        Ok(definition)
    }

    pub(crate) fn group(
        &mut self,
        entity_path: &str,
        entity: &str,
        group: GroupDefinition,
    ) -> Result<(), Denial> {
        let mut rows = std::collections::BTreeSet::new();
        for case in group.cases() {
            if !rows.insert(case.row_type()) {
                return Err(Denial::DuplicateOneOfRow {
                    entity: entity_path.to_owned(),
                    group: group.name(),
                    row: case.row_name(),
                });
            }
        }
        let owner = self.owner(entity_path)?;
        let entity_type = owner
            .entities
            .iter()
            .find(|(_, name)| name.as_str() == entity)
            .map(|(key, _)| *key)
            .ok_or(Denial::MissingEntity)?;
        owner.groups.insert((entity_type, group.marker()), group);
        Ok(())
    }

    pub(crate) fn relation<R: super::Relation>(
        &mut self,
        entity_path: &str,
        entity: &str,
    ) -> Result<(), Denial> {
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
                integrity: super::declaration::relation_integrity::<R>(),
            },
        );
        Ok(())
    }
}

impl FragmentDefinition {
    pub fn case_aspect<E: Entity>(
        &self,
        case: &super::CaseDefinition,
    ) -> Option<&AspectDefinition> {
        self.aspects.get(&(TypeId::of::<E>(), case.marker()))
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn entity<E: Entity>(&self) -> Option<&str> {
        self.entities.get(&TypeId::of::<E>()).map(String::as_str)
    }
    pub fn aspect<E: Entity, A: Aspect>(&self) -> Option<&AspectDefinition> {
        self.aspects.get(&(TypeId::of::<E>(), TypeId::of::<A>()))
    }
    pub fn group<E: Entity, G: super::OneOf>(&self) -> Option<&GroupDefinition> {
        self.groups.get(&(TypeId::of::<E>(), TypeId::of::<G>()))
    }
    pub fn relation<E: Entity, R: super::Relation>(&self) -> Option<&RelationDefinition> {
        self.relations.get(&(TypeId::of::<E>(), TypeId::of::<R>()))
    }
    pub fn case_group<E: Entity, A: Aspect>(&self) -> Option<&GroupDefinition> {
        self.groups
            .iter()
            .filter(|((entity, _), _)| *entity == TypeId::of::<E>())
            .map(|(_, group)| group)
            .find(|group| group.contains::<A>())
    }
}

impl AspectDefinition {
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn entity(&self) -> &str {
        &self.entity
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn binding(&self) -> &RecordBinding {
        &self.binding
    }
    pub fn fields(&self) -> &[RecordFieldDefinition] {
        &self.fields
    }
    pub fn optional(&self) -> bool {
        self.optional
    }
}

impl RecordFieldDefinition {
    pub fn recipe(&self) -> &ApplicationFieldBindingRecipe {
        &self.recipe
    }
    pub fn writable(&self) -> bool {
        self.writable
    }
    pub fn equality(&self) -> bool {
        self.equality
    }
    pub(crate) fn unique(&self) -> bool {
        self.unique
    }
}
