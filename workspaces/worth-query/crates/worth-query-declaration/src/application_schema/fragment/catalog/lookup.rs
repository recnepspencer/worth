//! Type markers extract identities; concrete lookups walk the installed tables once.
use super::*;
impl FragmentDefinition {
    pub fn aspects(&self) -> impl Iterator<Item = &AspectDefinition> {
        self.aspects.values()
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    #[inline(always)]
    pub fn relation_aspect<R: super::super::Relation, A: Aspect>(
        &self,
    ) -> Option<&AspectDefinition> {
        self.aspect_by_identity(TypeId::of::<R>(), TypeId::of::<A>())
    }
    #[inline(always)]
    pub fn case_aspect<E: Entity>(
        &self,
        case: &super::super::CaseDefinition,
    ) -> Option<&AspectDefinition> {
        self.case_aspect_by_identity(TypeId::of::<E>(), case)
    }
    #[doc(hidden)]
    pub fn case_aspect_by_identity(
        &self,
        entity: TypeId,
        case: &super::super::CaseDefinition,
    ) -> Option<&AspectDefinition> {
        self.aspect_by_identity(entity, case.marker())
    }
    #[inline(always)]
    pub fn entity<E: Entity>(&self) -> Option<&str> {
        self.entity_by_identity(TypeId::of::<E>())
    }
    #[doc(hidden)]
    pub fn entity_by_identity(&self, entity: TypeId) -> Option<&str> {
        self.entities.get(&entity).map(String::as_str)
    }
    #[inline(always)]
    pub fn aspect<E: Entity, A: Aspect>(&self) -> Option<&AspectDefinition> {
        self.aspect_by_identity(TypeId::of::<E>(), TypeId::of::<A>())
    }
    #[doc(hidden)]
    pub fn aspect_by_identity(&self, entity: TypeId, aspect: TypeId) -> Option<&AspectDefinition> {
        self.aspects.get(&(entity, aspect))
    }
    #[inline(always)]
    pub fn group<E: Entity, G: super::super::OneOf>(&self) -> Option<&GroupDefinition> {
        self.group_by_identity(TypeId::of::<E>(), TypeId::of::<G>())
    }
    #[doc(hidden)]
    pub fn group_by_identity(&self, entity: TypeId, group: TypeId) -> Option<&GroupDefinition> {
        self.groups.get(&(entity, group))
    }
    #[inline(always)]
    pub fn relation<E: Entity, R: super::super::Relation>(&self) -> Option<&RelationDefinition> {
        self.relation_by_identity(TypeId::of::<E>(), TypeId::of::<R>())
    }
    #[doc(hidden)]
    pub fn relation_by_identity(
        &self,
        entity: TypeId,
        relation: TypeId,
    ) -> Option<&RelationDefinition> {
        self.relations.get(&(entity, relation))
    }
    #[inline(always)]
    pub fn case_group<E: Entity, A: Aspect>(&self) -> Option<&GroupDefinition> {
        self.case_group_by_identity(TypeId::of::<E>(), TypeId::of::<A>())
    }
    #[doc(hidden)]
    pub fn case_group_by_identity(
        &self,
        entity: TypeId,
        aspect: TypeId,
    ) -> Option<&GroupDefinition> {
        self.groups
            .iter()
            .filter(|((owner, _), _)| *owner == entity)
            .map(|(_, group)| group)
            .find(|group| group.cases().iter().any(|case| case.marker() == aspect))
    }
}
