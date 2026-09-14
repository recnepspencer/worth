use worth_query_installation::facade::{ApplicationRelationRef, ApplicationSchema};
use worth_relational::facade::identity::EntityId;

use super::{
    WorthQueryGeneratedEntity, WorthQueryGeneratedOutputReconstruction,
    WorthQueryGeneratedOutputReconstructionDenial,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationProducerBinding;

impl<Schema, Producer> WorthQueryGeneratedOutputReconstruction<'_, Schema, Producer>
where
    Schema: ApplicationSchema,
    Producer: WorthQueryApplicationProducerBinding<Schema>,
{
    pub fn relation<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        source: &WorthQueryGeneratedEntity<Schema, From>,
        target: &WorthQueryGeneratedEntity<Schema, To>,
    ) -> Result<(), WorthQueryGeneratedOutputReconstructionDenial> {
        self.validate_handle(source)?;
        self.validate_handle(target)?;
        self.claim_relation(
            relation.name(),
            source.identity,
            target.identity,
            false,
            false,
        )
    }

    pub fn relation_from_retained<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        target: &WorthQueryGeneratedEntity<Schema, To>,
    ) -> Result<(), WorthQueryGeneratedOutputReconstructionDenial> {
        self.validate_handle(target)?;
        self.claim_relation(
            relation.name(),
            target.identity,
            target.identity,
            true,
            false,
        )
    }

    pub fn relation_to_retained<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        source: &WorthQueryGeneratedEntity<Schema, From>,
    ) -> Result<(), WorthQueryGeneratedOutputReconstructionDenial> {
        self.validate_handle(source)?;
        self.claim_relation(
            relation.name(),
            source.identity,
            source.identity,
            false,
            true,
        )
    }

    fn claim_relation(
        &mut self,
        name: &str,
        source: EntityId,
        target: EntityId,
        external_source: bool,
        external_target: bool,
    ) -> Result<(), WorthQueryGeneratedOutputReconstructionDenial> {
        let expected = self
            .layout
            .relation(name)
            .ok_or(WorthQueryGeneratedOutputReconstructionDenial::MissingRelation)?;
        let generated = &self.entities;
        let matching = self
            .relations
            .iter()
            .enumerate()
            .filter(|(_, candidate)| {
                candidate.kind == expected.kind
                    && if external_source {
                        !generated.contains_key(&candidate.source) && candidate.target == target
                    } else if external_target {
                        candidate.source == source && !generated.contains_key(&candidate.target)
                    } else {
                        candidate.source == source && candidate.target == target
                    }
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
}
