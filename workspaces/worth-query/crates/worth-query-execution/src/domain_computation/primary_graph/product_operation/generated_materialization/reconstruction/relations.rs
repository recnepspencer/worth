use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_installation::facade::{
    ApplicationEntityRef, ApplicationRelationRef, ApplicationSchema,
};
use worth_relational::facade::identity::EntityId;

use super::{
    WorthQueryGeneratedEntity, WorthQueryGeneratedOutputReconstruction,
    WorthQueryGeneratedOutputReconstructionDenial, WorthQueryRetainedGeneratedOutputEntity,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationProducerBinding;

impl<Schema, Producer> WorthQueryGeneratedOutputReconstruction<'_, Schema, Producer>
where
    Schema: ApplicationSchema,
    Producer: WorthQueryApplicationProducerBinding<Schema>,
{
    pub fn retained_relation_sources<Relation, From, To>(
        &self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        source: ApplicationEntityRef<Schema, From>,
        target: &WorthQueryGeneratedEntity<Schema, To>,
    ) -> Result<
        Vec<WorthQueryRetainedGeneratedOutputEntity<Schema, From>>,
        WorthQueryGeneratedOutputReconstructionDenial,
    > {
        self.validate_handle(target)?;
        let expected = self
            .layout
            .relation(relation.name())
            .ok_or(WorthQueryGeneratedOutputReconstructionDenial::MissingRelation)?;
        let source_kind = self
            .layout
            .entity_kind(source.name())
            .ok_or(WorthQueryGeneratedOutputReconstructionDenial::RetainedEntityKindMismatch)?;
        if expected.from != source_kind {
            return Err(WorthQueryGeneratedOutputReconstructionDenial::RetainedEntityKindMismatch);
        }
        Ok(self
            .retained_endpoints(
                expected.kind,
                |candidate| candidate.target == target.identity,
                |candidate| candidate.source,
            )
            .into_iter()
            .map(|identity| self.retained_handle(identity))
            .collect())
    }

    pub fn retained_relation_source<Relation, From, To>(
        &self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        source: ApplicationEntityRef<Schema, From>,
        target: &WorthQueryGeneratedEntity<Schema, To>,
    ) -> Result<
        WorthQueryRetainedGeneratedOutputEntity<Schema, From>,
        WorthQueryGeneratedOutputReconstructionDenial,
    > {
        self.validate_handle(target)?;
        let expected = self
            .layout
            .relation(relation.name())
            .ok_or(WorthQueryGeneratedOutputReconstructionDenial::MissingRelation)?;
        let source_kind = self
            .layout
            .entity_kind(source.name())
            .ok_or(WorthQueryGeneratedOutputReconstructionDenial::RetainedEntityKindMismatch)?;
        if expected.from != source_kind {
            return Err(WorthQueryGeneratedOutputReconstructionDenial::RetainedEntityKindMismatch);
        }
        let identity = self.unique_retained_endpoint(
            expected.kind,
            |candidate| candidate.target == target.identity,
            |candidate| candidate.source,
        )?;
        Ok(self.retained_handle(identity))
    }

    pub fn retained_relation_target<Relation, From, To>(
        &self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        source: &WorthQueryGeneratedEntity<Schema, From>,
        target: ApplicationEntityRef<Schema, To>,
    ) -> Result<
        WorthQueryRetainedGeneratedOutputEntity<Schema, To>,
        WorthQueryGeneratedOutputReconstructionDenial,
    > {
        self.validate_handle(source)?;
        let expected = self
            .layout
            .relation(relation.name())
            .ok_or(WorthQueryGeneratedOutputReconstructionDenial::MissingRelation)?;
        let target_kind = self
            .layout
            .entity_kind(target.name())
            .ok_or(WorthQueryGeneratedOutputReconstructionDenial::RetainedEntityKindMismatch)?;
        if expected.to != target_kind {
            return Err(WorthQueryGeneratedOutputReconstructionDenial::RetainedEntityKindMismatch);
        }
        let identity = self.unique_retained_endpoint(
            expected.kind,
            |candidate| candidate.source == source.identity,
            |candidate| candidate.target,
        )?;
        Ok(self.retained_handle(identity))
    }

    pub fn retained_relation_targets<Relation, From, To>(
        &self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        source: &WorthQueryGeneratedEntity<Schema, From>,
        target: ApplicationEntityRef<Schema, To>,
    ) -> Result<
        Vec<WorthQueryRetainedGeneratedOutputEntity<Schema, To>>,
        WorthQueryGeneratedOutputReconstructionDenial,
    > {
        self.validate_handle(source)?;
        let expected = self
            .layout
            .relation(relation.name())
            .ok_or(WorthQueryGeneratedOutputReconstructionDenial::MissingRelation)?;
        let target_kind = self
            .layout
            .entity_kind(target.name())
            .ok_or(WorthQueryGeneratedOutputReconstructionDenial::RetainedEntityKindMismatch)?;
        if expected.to != target_kind {
            return Err(WorthQueryGeneratedOutputReconstructionDenial::RetainedEntityKindMismatch);
        }
        Ok(self
            .retained_endpoints(
                expected.kind,
                |candidate| candidate.source == source.identity,
                |candidate| candidate.target,
            )
            .into_iter()
            .map(|identity| self.retained_handle(identity))
            .collect())
    }

    pub fn relation<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        source: &WorthQueryGeneratedEntity<Schema, From>,
        target: &WorthQueryGeneratedEntity<Schema, To>,
    ) -> Result<(), WorthQueryGeneratedOutputReconstructionDenial> {
        self.validate_handle(source)?;
        self.validate_handle(target)?;
        self.claim_relation(relation.name(), source.identity, target.identity)
    }

    pub fn relation_from_retained<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        source: &WorthQueryRetainedGeneratedOutputEntity<Schema, From>,
        target: &WorthQueryGeneratedEntity<Schema, To>,
    ) -> Result<(), WorthQueryGeneratedOutputReconstructionDenial> {
        self.validate_retained_handle(source)?;
        self.validate_handle(target)?;
        self.claim_relation(relation.name(), source.identity, target.identity)
    }

    pub fn relation_to_retained<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        source: &WorthQueryGeneratedEntity<Schema, From>,
        target: &WorthQueryRetainedGeneratedOutputEntity<Schema, To>,
    ) -> Result<(), WorthQueryGeneratedOutputReconstructionDenial> {
        self.validate_handle(source)?;
        self.validate_retained_handle(target)?;
        self.claim_relation(relation.name(), source.identity, target.identity)
    }

    fn claim_relation(
        &mut self,
        name: &str,
        source: EntityId,
        target: EntityId,
    ) -> Result<(), WorthQueryGeneratedOutputReconstructionDenial> {
        let expected = self
            .layout
            .relation(name)
            .ok_or(WorthQueryGeneratedOutputReconstructionDenial::MissingRelation)?;
        let matching = self
            .relations
            .iter()
            .enumerate()
            .filter(|(_, candidate)| {
                candidate.kind == expected.kind
                    && candidate.source == source
                    && candidate.target == target
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        match matching.as_slice() {
            [] => Err(WorthQueryGeneratedOutputReconstructionDenial::WrongRelationEndpoint),
            [index] if self.relations[*index].claimed => {
                Err(WorthQueryGeneratedOutputReconstructionDenial::DuplicateRelationClaim)
            }
            [index] => {
                self.relations[*index].claimed = true;
                Ok(())
            }
            _ => Err(WorthQueryGeneratedOutputReconstructionDenial::AmbiguousRelation),
        }
    }

    fn unique_retained_endpoint(
        &self,
        kind: worth_relational::facade::identity::KindId,
        matches_anchor: impl Fn(&super::ReconstructionRelation) -> bool,
        endpoint: impl Fn(&super::ReconstructionRelation) -> EntityId,
    ) -> Result<EntityId, WorthQueryGeneratedOutputReconstructionDenial> {
        let mut matches = self
            .relations
            .iter()
            .filter(|candidate| candidate.kind == kind && matches_anchor(candidate))
            .map(endpoint)
            .filter(|identity| !self.entities.contains_key(identity));
        let first = matches
            .next()
            .ok_or(WorthQueryGeneratedOutputReconstructionDenial::WrongRelationEndpoint)?;
        if matches.next().is_some() {
            return Err(WorthQueryGeneratedOutputReconstructionDenial::AmbiguousRelation);
        }
        Ok(first)
    }

    fn retained_endpoints(
        &self,
        kind: worth_relational::facade::identity::KindId,
        matches_anchor: impl Fn(&super::ReconstructionRelation) -> bool,
        endpoint: impl Fn(&super::ReconstructionRelation) -> EntityId,
    ) -> Vec<EntityId> {
        self.relations
            .iter()
            .filter(|candidate| candidate.kind == kind && matches_anchor(candidate))
            .map(endpoint)
            .filter(|identity| !self.entities.contains_key(identity))
            .collect()
    }

    fn retained_handle<Entity>(
        &self,
        identity: EntityId,
    ) -> WorthQueryRetainedGeneratedOutputEntity<Schema, Entity> {
        WorthQueryRetainedGeneratedOutputEntity {
            identity,
            session: Arc::clone(&self.session),
            _marker: PhantomData,
        }
    }
}
