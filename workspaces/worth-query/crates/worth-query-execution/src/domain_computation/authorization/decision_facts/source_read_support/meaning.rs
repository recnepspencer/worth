//! Descriptive support meaning under the caller's exact compiled Source contract.
use super::*;
use worth_relational::facade::authorization::{
    RelationalAuthorizationObservationEvidence as Evidence,
    RelationalAuthorizationTraversalDirection as Direction,
};
use worth_relational::facade::identity::EntityId;

impl SourceReadSupport {
    /// The caller must first compare the actual compiled Source binding. That
    /// immutable contract and this registry fix the complete conventional plan.
    pub(in crate::domain_computation) fn same_conventional_support<E>(
        &self,
        other: &Self,
        admit: &mut impl FnMut(u64) -> Result<(), E>,
    ) -> Result<bool, E> {
        admit(1)?;
        if !Arc::ptr_eq(&self.registry, &other.registry) {
            return Ok(false);
        }
        let (Some(left), Some(right)) = (self.observations(), other.observations()) else {
            return Ok(false);
        };
        if !self.visit_support(&mut |bytes| admit(bytes.len() as u64))?
            || !other.visit_support(&mut |bytes| admit(bytes.len() as u64))?
        {
            return Ok(false);
        }
        Ok(self.principal_facts == other.principal_facts
            && left.len() == right.len()
            && left.iter().zip(right).all(|(left, right)| {
                let (Some(left), Some(right)) =
                    (left.source_observation(), right.source_observation())
                else {
                    return false;
                };
                left.principal() == right.principal()
                    && left.scope() == right.scope()
                    && left.paths() == right.paths()
            }))
    }

    fn observations(&self) -> Option<&[super::super::WorthQueryAuthorizationDecisionFact]> {
        match &self.authorization {
            WorthQueryRetainedAuthorizationDecisionFacts::Principal(_) => Some(&[]),
            WorthQueryRetainedAuthorizationDecisionFacts::Abilities { decisions, .. } => {
                Some(decisions)
            }
            WorthQueryRetainedAuthorizationDecisionFacts::Capability(_) => None,
        }
    }

    pub(in crate::domain_computation) fn visit_support<E>(
        &self,
        visit: &mut impl FnMut(&[u8]) -> Result<(), E>,
    ) -> Result<bool, E> {
        let Some(observations) = self.observations() else {
            return Ok(false);
        };
        visit(b"worth-query:conventional-source-support:v1")?;
        visit(&(self.principal_facts.len() as u64).to_be_bytes())?;
        for fact in &self.principal_facts {
            principal_fact(fact, visit)?;
        }
        visit(&(observations.len() as u64).to_be_bytes())?;
        for observation in observations {
            let Some(observation) = observation.source_observation() else {
                return Ok(false);
            };
            evidence(observation, visit)?;
        }
        Ok(true)
    }
}

fn evidence<E>(value: &Evidence, visit: &mut impl FnMut(&[u8]) -> Result<(), E>) -> Result<(), E> {
    entity(value.principal(), visit)?;
    entity(value.scope(), visit)?;
    visit(&(value.paths().len() as u64).to_be_bytes())?;
    for path in value.paths() {
        visit(&[
            u8::from(path.matched()),
            u8::from(path.exhaustive()),
            u8::from(path.witness().is_some()),
        ])?;
        entities(
            path.witness().map_or(&[], |witness| witness.entities()),
            visit,
        )?;
        entities(path.entities(), visit)?;
        visit(&(path.relations().len() as u64).to_be_bytes())?;
        for relation in path.relations() {
            visit(&relation.partition_value().to_be_bytes())?;
            visit(&relation.local_slot_value().to_be_bytes())?;
            visit(&relation.generation_value().to_be_bytes())?;
        }
        visit(&(path.adjacency_lists().len() as u64).to_be_bytes())?;
        for adjacency in path.adjacency_lists() {
            entity(adjacency.entity(), visit)?;
            visit(&adjacency.relation_kind().as_u32().to_be_bytes())?;
            visit(&[match adjacency.direction() {
                Direction::Forward => 0,
                Direction::Reverse => 1,
            }])?;
        }
        visit(&(path.fields().len() as u64).to_be_bytes())?;
        for (owner, field) in path.fields() {
            entity(*owner, visit)?;
            locator(field, visit)?;
        }
    }
    Ok(())
}
fn entities<E>(
    values: &[EntityId],
    visit: &mut impl FnMut(&[u8]) -> Result<(), E>,
) -> Result<(), E> {
    visit(&(values.len() as u64).to_be_bytes())?;
    for value in values {
        entity(*value, visit)?;
    }
    Ok(())
}
fn entity<E>(value: EntityId, visit: &mut impl FnMut(&[u8]) -> Result<(), E>) -> Result<(), E> {
    visit(&value.partition_value().to_be_bytes())?;
    visit(&value.local_slot_value().to_be_bytes())?;
    visit(&value.generation_value().to_be_bytes())
}
fn locator<E>(
    value: &worth_foundational::facade::AspectFieldLocator,
    visit: &mut impl FnMut(&[u8]) -> Result<(), E>,
) -> Result<(), E> {
    use worth_foundational::facade::LocatorAuthority as Authority;
    visit(&[match value.aspect().authority() {
        Authority::Authoritative => 0,
        Authority::Derived => 1,
        Authority::Projected => 2,
        Authority::SupportOnly => 3,
        Authority::Planned => 4,
        Authority::ReceiptBearing => 5,
    }])?;
    text(value.aspect().aspect_key().as_str(), visit)?;
    visit(&(value.field_path().fields().len() as u64).to_be_bytes())?;
    for field in value.field_path().fields() {
        text(field.as_str(), visit)?;
    }
    Ok(())
}
fn text<E>(value: &str, visit: &mut impl FnMut(&[u8]) -> Result<(), E>) -> Result<(), E> {
    visit(&(value.len() as u64).to_be_bytes())?;
    visit(value.as_bytes())
}
fn principal_fact<E>(
    fact: &WorthQueryApplicationObservedFact,
    visit: &mut impl FnMut(&[u8]) -> Result<(), E>,
) -> Result<(), E> {
    use WorthQueryApplicationObservedFact as Fact;
    match fact {
        Fact::Entity { entity_id, kind } => {
            visit(&[0])?;
            entity(*entity_id, visit)?;
            visit(&kind.as_u32().to_be_bytes())?;
        }
        Fact::SourceFieldRevision {
            entity_id,
            locator: field,
            native_revision,
        } => {
            visit(&[1])?;
            entity(*entity_id, visit)?;
            locator(field, visit)?;
            visit(&[u8::from(native_revision.is_some())])?;
            if let Some(revision) = native_revision {
                visit(&revision.version().0.to_be_bytes())?;
                use worth_relational::facade::runtime::RelationalFieldPresence as Presence;
                visit(&[match revision.presence() {
                    Presence::Present => 0,
                    Presence::Absent => 1,
                }])?;
            }
        }
        Fact::SourceAdjacencyRevision {
            relation_kind,
            anchor,
            direction,
            native_revision,
            comparison_work_limit,
            endpoints,
        } => {
            debug_assert!(endpoints.is_empty());
            visit(&[2])?;
            entity(*anchor, visit)?;
            visit(&relation_kind.as_u32().to_be_bytes())?;
            use worth_relational::facade::runtime::RelationalAdjacencyDirection as D;
            visit(&[
                match direction {
                    D::Outgoing => 0,
                    D::Incoming => 1,
                },
                u8::from(native_revision.is_some()),
            ])?;
            if let Some(revision) = native_revision {
                visit(&revision.0.to_be_bytes())?;
            }
            visit(&(*comparison_work_limit as u64).to_be_bytes())?;
        }
        _ => unreachable!("the principal issuer emits only entity, field and adjacency facts"),
    }
    Ok(())
}

impl SourceReadSupport {
    pub(in crate::domain_computation) fn visit_loci<E>(
        &self,
        visit: &mut impl FnMut(SourceAuthorizationLocus<'_>) -> Result<(), E>,
    ) -> Result<bool, E> {
        let Some(observations) = self.observations() else {
            return Ok(false);
        };
        for fact in &self.principal_facts {
            use WorthQueryApplicationObservedFact as Fact;
            match fact {
                Fact::Entity { entity_id, .. } => {
                    visit(SourceAuthorizationLocus::Entity(*entity_id))?
                }
                Fact::SourceFieldRevision {
                    entity_id, locator, ..
                } => visit(SourceAuthorizationLocus::Field(*entity_id, locator))?,
                Fact::SourceAdjacencyRevision {
                    relation_kind,
                    anchor,
                    direction,
                    ..
                } => {
                    visit(SourceAuthorizationLocus::Entity(*anchor))?;
                    visit(SourceAuthorizationLocus::Adjacency(*anchor, *relation_kind, match direction {
                        worth_relational::facade::runtime::RelationalAdjacencyDirection::Outgoing => Direction::Forward,
                        worth_relational::facade::runtime::RelationalAdjacencyDirection::Incoming => Direction::Reverse,
                    }))?;
                }
                _ => unreachable!("principal issuer's closed fact family"),
            }
        }
        for decision in observations {
            let Some(observation) = decision.source_observation() else {
                return Ok(false);
            };
            for path in observation.paths() {
                for entity in path.entities() {
                    visit(SourceAuthorizationLocus::Entity(*entity))?;
                }
                for relation in path.relations() {
                    visit(SourceAuthorizationLocus::Relation(*relation))?;
                }
                for adjacency in path.adjacency_lists() {
                    visit(SourceAuthorizationLocus::Entity(adjacency.entity()))?;
                    visit(SourceAuthorizationLocus::Adjacency(
                        adjacency.entity(),
                        adjacency.relation_kind(),
                        adjacency.direction(),
                    ))?;
                }
                for (entity, field) in path.fields() {
                    visit(SourceAuthorizationLocus::Field(*entity, field))?;
                }
            }
        }
        Ok(true)
    }
}

pub(in crate::domain_computation) enum SourceAuthorizationLocus<'a> {
    Entity(EntityId),
    Relation(worth_relational::facade::identity::RelationId),
    Adjacency(
        EntityId,
        worth_relational::facade::identity::KindId,
        Direction,
    ),
    Field(EntityId, &'a worth_foundational::facade::AspectFieldLocator),
}
