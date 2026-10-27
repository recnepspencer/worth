//! A derived-law failure carries actual endpoints and physical source records.
use super::{DerivedRelationLaw as Law, DerivedRelationLawViolation as Violation};
use crate::derived_relation::evaluation::{member_work, SourceRead};
use crate::derived_relation::{
    DerivedRelationData as Data, DerivedRelationDenial as DefinitionDenial,
    DerivedRelationReadDenial as Denial,
};
use crate::graph_member::NativeMemberKey as Member;
use crate::identity::data::{EntityId, RelationId};
use std::collections::BTreeSet;

pub(super) fn violation<'data>(
    law: Law,
    source: &Member,
    target: Option<&Member>,
    count: u64,
    limit: u64,
    support: impl IntoIterator<Item = &'data Data>,
    read: &mut impl SourceRead,
) -> Result<Violation, Denial> {
    member_work(read, source, 1)?;
    if let Some(target) = target {
        member_work(read, target, 1)?;
    }
    let mut entities = BTreeSet::new();
    let mut relations = BTreeSet::new();
    for data in support {
        for (entity, _) in data.entity_dependencies() {
            read.checkpoint(1)?;
            if !entities.contains(entity) {
                read.retain(size_of::<EntityId>() + 4 * size_of::<usize>())?;
                entities.insert(*entity);
            }
        }
        for relation in data.physical_relations() {
            read.checkpoint(1)?;
            if !relations.contains(&relation.relation_id) {
                read.retain(size_of::<RelationId>() + 4 * size_of::<usize>())?;
                relations.insert(relation.relation_id);
            }
        }
    }
    let bytes = entities
        .len()
        .checked_mul(size_of::<EntityId>())
        .and_then(|n| {
            relations
                .len()
                .checked_mul(size_of::<RelationId>())
                .and_then(|r| n.checked_add(r))
        })
        .and_then(|n| n.checked_add(size_of::<Violation>()))
        .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
    read.retain(bytes)?;
    Ok(Violation {
        law,
        member: source.clone(),
        counterpart: target.cloned(),
        count,
        limit,
        physical_entities: entities.into_iter().collect(),
        physical_relations: relations.into_iter().collect(),
    })
}
