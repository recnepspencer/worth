//! Completed effect and normalization addresses, paired with the current body.
use super::super::retained_decision_facts::{StorageControl, StoreDenial};
use super::super::WorthQueryApplicationObservedFact as Fact;
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload;
use std::collections::BTreeMap;
use std::{mem::size_of, sync::Arc};
use worth_execution::ExecutionMemoryReservation;
use worth_foundational::facade::AspectFieldLocator;
use worth_foundational::facade::{
    prepare_aspect_value_identity_basis, CanonicalAspectValueIdentityBasis,
};
use worth_relational::facade::identity::{EntityId, KindId, RelationId};
use worth_relational::facade::indexes::DerivedIndexId;

type FieldOrdinals = BTreeMap<EntityId, BTreeMap<AspectFieldLocator, Vec<usize>>>;
pub(in crate::domain_computation::primary_graph) struct FactRoutes {
    ordinals: FieldOrdinals,
    entities: BTreeMap<EntityId, usize>,
    relations: BTreeMap<RelationId, usize>,
    selections: BTreeMap<
        (
            DerivedIndexId,
            KindId,
            AspectFieldLocator,
            CanonicalAspectValueIdentityBasis,
        ),
        usize,
    >,
    bytes: u64,
    _memory: Option<ExecutionMemoryReservation>,
}

fn address(fact: &Fact) -> Option<(EntityId, &AspectFieldLocator)> {
    match fact {
        Fact::Field {
            entity_id, locator, ..
        }
        | Fact::AbsentField {
            entity_id, locator, ..
        }
        | Fact::OptionalAspectAbsent {
            entity_id, locator, ..
        } => Some((*entity_id, locator)),
        _ => None,
    }
}

impl FactRoutes {
    pub(in crate::domain_computation::primary_graph) fn build(
        body: &FactPayload,
        control: StorageControl<'_, '_>,
    ) -> Result<Arc<Self>, StoreDenial> {
        let mut bytes = body
            .retained_resident_bytes()
            .and_then(|bytes| bytes.checked_mul(16))
            .and_then(|bytes| bytes.checked_add(u64::try_from(body.len()).ok()?.checked_mul(1024)?))
            .and_then(|bytes| {
                bytes.checked_add((size_of::<Self>() + 2 * size_of::<usize>()) as u64)
            })
            .ok_or(StoreDenial::Representability)?;
        for fact in body.iter() {
            control.check_live()?;
            super::super::super::fact_verification_observation::effect_index_fact();
            super::super::super::fact_verification_observation::content_field_index();
            if matches!(fact, Fact::IndexedEntitySelection { .. }) {
                bytes = bytes
                    .checked_add(
                        fact.dependency_locator_byte_bound()
                            .and_then(|n| n.checked_mul(4))
                            .ok_or(StoreDenial::Representability)?,
                    )
                    .ok_or(StoreDenial::Representability)?;
            }
        }
        let memory = control.policy().reserve_memory(bytes);
        control.check_live()?;
        let memory = memory?;
        Self::build_iter(body.iter(), bytes, memory, control)
    }
    pub(in crate::domain_computation::primary_graph) fn build_iter<'a>(
        facts: impl Iterator<Item = &'a Fact>,
        bytes: u64,
        memory: Option<ExecutionMemoryReservation>,
        control: StorageControl<'_, '_>,
    ) -> Result<Arc<Self>, StoreDenial> {
        let mut routes = Self {
            ordinals: BTreeMap::new(),
            entities: BTreeMap::new(),
            relations: BTreeMap::new(),
            selections: BTreeMap::new(),
            bytes,
            _memory: memory,
        };
        for (ordinal, fact) in facts.enumerate() {
            control.check_live()?;
            super::super::super::fact_verification_observation::effect_index_fact();
            super::super::super::fact_verification_observation::content_field_index();
            if let Some((entity, locator)) = address(fact) {
                routes
                    .ordinals
                    .entry(entity)
                    .or_default()
                    .entry(locator.clone())
                    .or_default()
                    .push(ordinal);
            }
            match fact {
                Fact::Entity { entity_id, .. } => {
                    routes.entities.entry(*entity_id).or_insert(ordinal);
                }
                Fact::Relation {
                    matching_relations, ..
                } => {
                    for id in matching_relations {
                        routes.relations.entry(*id).or_insert(ordinal);
                    }
                }
                Fact::Adjacency { relations, .. } => {
                    for relation in relations {
                        routes
                            .relations
                            .entry(relation.relation_id)
                            .or_insert(ordinal);
                    }
                }
                Fact::IndexedEntitySelection {
                    index_id,
                    entity_kind,
                    locator,
                    value,
                    ..
                } => {
                    routes
                        .selections
                        .entry((
                            *index_id,
                            *entity_kind,
                            locator.clone(),
                            prepare_aspect_value_identity_basis(value),
                        ))
                        .or_insert(ordinal);
                }
                _ => {}
            }
        }
        Ok(Arc::new(routes))
    }
    pub(in crate::domain_computation::primary_graph) fn field(
        &self,
        entity: EntityId,
        locator: &AspectFieldLocator,
    ) -> Option<usize> {
        self.ordinals(entity, locator).first().copied()
    }
    pub(in crate::domain_computation::primary_graph) fn entity(
        &self,
        id: EntityId,
    ) -> Option<usize> {
        self.entities.get(&id).copied()
    }
    pub(in crate::domain_computation::primary_graph) fn relation(
        &self,
        id: RelationId,
    ) -> Option<usize> {
        self.relations.get(&id).copied()
    }
    pub(in crate::domain_computation::primary_graph) fn selection(
        &self,
        key: &(
            DerivedIndexId,
            KindId,
            AspectFieldLocator,
            CanonicalAspectValueIdentityBasis,
        ),
    ) -> Option<usize> {
        self.selections.get(key).copied()
    }
    pub(in crate::domain_computation::primary_graph) fn ordinals(
        &self,
        entity: EntityId,
        locator: &AspectFieldLocator,
    ) -> &[usize] {
        self.ordinals
            .get(&entity)
            .and_then(|fields| fields.get(locator))
            .map_or(&[], Vec::as_slice)
    }
    pub(in crate::domain_computation::primary_graph) fn addresses_match(
        old: Option<&Fact>,
        new: Option<&Fact>,
        control: StorageControl<'_, '_>,
    ) -> Result<bool, StoreDenial> {
        super::super::super::fact_verification_observation::effect_index_fact();
        super::super::super::fact_verification_observation::content_field_index();
        if old.and_then(address) != new.and_then(address) {
            return Ok(false);
        }
        let matches = match (old, new) {
            (Some(Fact::Entity { entity_id: a, .. }), Some(Fact::Entity { entity_id: b, .. })) => {
                a == b
            }
            (
                Some(Fact::Relation {
                    matching_relations: a,
                    ..
                }),
                Some(Fact::Relation {
                    matching_relations: b,
                    ..
                }),
            ) => a == b,
            (
                Some(Fact::Adjacency { relations: a, .. }),
                Some(Fact::Adjacency { relations: b, .. }),
            ) => a == b,
            (
                Some(Fact::IndexedEntitySelection {
                    index_id: a,
                    entity_kind: ak,
                    locator: al,
                    value: av,
                    ..
                }),
                Some(Fact::IndexedEntitySelection {
                    index_id: b,
                    entity_kind: bk,
                    locator: bl,
                    value: bv,
                    ..
                }),
            ) => {
                if (a, ak, al) != (b, bk, bl) {
                    return Ok(false);
                }
                let bytes = old
                    .and_then(Fact::dependency_locator_byte_bound)
                    .and_then(|n| n.checked_add(new.and_then(Fact::dependency_locator_byte_bound)?))
                    .ok_or(StoreDenial::Representability)?;
                let memory = control.policy().reserve_memory(bytes);
                control.check_live()?;
                let _memory = memory?;
                prepare_aspect_value_identity_basis(av) == prepare_aspect_value_identity_basis(bv)
            }
            (
                Some(
                    Fact::Entity { .. }
                    | Fact::Relation { .. }
                    | Fact::Adjacency { .. }
                    | Fact::IndexedEntitySelection { .. },
                ),
                _,
            )
            | (
                _,
                Some(
                    Fact::Entity { .. }
                    | Fact::Relation { .. }
                    | Fact::Adjacency { .. }
                    | Fact::IndexedEntitySelection { .. },
                ),
            ) => false,
            _ => true,
        };
        control.check_live()?;
        Ok(matches)
    }
    pub(in crate::domain_computation::primary_graph) fn bytes(&self) -> u64 {
        self.bytes
    }
}
