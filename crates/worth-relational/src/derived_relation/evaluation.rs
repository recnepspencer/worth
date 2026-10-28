//! One iterative endpoint evaluator for physical and projected member domains.
mod memo;
mod navigation;
mod physical;

use super::read::{DerivedRelationAdjacency as Adjacency, DerivedRelationData as Data};
use super::read::{DerivedRelationReadDenial as Denial, DerivedRelationReads};
use super::{DerivedRelationDenial as DefinitionDenial, DerivedRelationHandle as Handle};
use crate::graph_member::NativeMemberKey as Member;
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::schema::data::RelationalSchemaRegistry;
use crate::validation::data::StructuralRelationRecord;
use memo::Key;
use navigation::Frame;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) trait SourceRead {
    fn checkpoint(&mut self, units: usize) -> Result<(), Denial>;
    fn retain(&mut self, bytes: usize) -> Result<(), Denial>;
    fn entity_kind(&mut self, id: EntityId) -> Result<KindId, Denial>;
    fn adjacency(&mut self, probe: Adjacency) -> Result<Vec<StructuralRelationRecord>, Denial>;
    fn reads(&self) -> DerivedRelationReads;
}

struct Evaluation<'a, Port> {
    registry: &'a RelationalSchemaRegistry,
    read: &'a mut Port,
    pending: Vec<Frame>,
    values: BTreeMap<Key, BTreeSet<Member>>,
    entities: BTreeMap<EntityId, KindId>,
    relations: BTreeMap<RelationId, StructuralRelationRecord>,
    adjacency: BTreeSet<Adjacency>,
}

/// The actual registry lookup is paid before any source acquisition.
pub(super) fn admit_definition(
    registry: &RelationalSchemaRegistry,
    handle: &Handle,
    read: &mut impl SourceRead,
) -> Result<(), Denial> {
    // This exact catalog lookup is also reached by canonical affectedness,
    // without the custom-view admission lookup's separate checkpoint.
    let names = handle
        .parts()
        .identity
        .schema
        .0
        .len()
        .checked_add(handle.parts().identity.name.len())
        .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
    let levels = (usize::BITS - registry.derived_relations.len().max(1).leading_zeros()) as usize;
    read.checkpoint(
        names
            .checked_mul(11)
            .and_then(|units| units.checked_mul(levels))
            .ok_or(DefinitionDenial::WorkBudgetExceeded)?,
    )?;
    if !registry.admits_derived_relation(handle) {
        return Err(DefinitionDenial::ForeignDefinition.into());
    }
    Ok(())
}

/// Native canonical affectedness pulls back an exact registered subexpression;
/// it never constructs a new recipe or stored relation ID for the prefix.
pub(crate) fn evaluate_position(
    registry: &RelationalSchemaRegistry,
    handle: &Handle,
    node: usize,
    reverse: bool,
    root: Member,
    read: &mut impl SourceRead,
) -> Result<Data, Denial> {
    admit_definition(registry, handle, read)?;
    let key = Key {
        definition: handle.clone(),
        node,
        anchor: root,
        reverse,
    };
    let mut evaluation = Evaluation {
        registry,
        read,
        pending: Vec::new(),
        values: BTreeMap::new(),
        entities: BTreeMap::new(),
        relations: BTreeMap::new(),
        adjacency: BTreeSet::new(),
    };
    let endpoints = handle
        .entry
        .endpoints
        .get(node)
        .ok_or(DefinitionDenial::InvalidExpression)?;
    evaluation.member(
        &key.anchor,
        if reverse { &endpoints.1 } else { &endpoints.0 },
    )?;
    evaluation.push(Frame::Enter(key.clone()))?;
    while let Some(frame) = evaluation.pending.pop() {
        evaluation.read.checkpoint(1)?;
        evaluation.step(frame)?;
    }
    evaluation.memo_key_work(&key)?;
    let endpoints = evaluation
        .values
        .remove(&key)
        .expect("complete root evaluation");
    let output_bytes = endpoints
        .len()
        .checked_mul(size_of::<Member>())
        .and_then(|n| {
            n.checked_add(
                evaluation
                    .entities
                    .len()
                    .checked_mul(size_of::<(EntityId, KindId)>())?,
            )
        })
        .and_then(|n| {
            n.checked_add(
                evaluation
                    .relations
                    .len()
                    .checked_mul(size_of::<StructuralRelationRecord>())?,
            )
        })
        .and_then(|n| {
            n.checked_add(
                evaluation
                    .adjacency
                    .len()
                    .checked_mul(size_of::<Adjacency>())?,
            )
        })
        .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
    evaluation.read.retain(output_bytes)?;
    Ok(Data {
        endpoints: endpoints.into_iter().collect(),
        entities: evaluation.entities.into_iter().collect(),
        relations: evaluation.relations.into_values().collect(),
        adjacency: evaluation.adjacency.into_iter().collect(),
        reads: evaluation.read.reads(),
    })
}

/// Fixed-width physical keys retain their one-unit insertion cost. Semantic
/// names add actual comparison bytes at the existing source checkpoint.
pub(in crate::derived_relation) fn member_work(
    read: &mut impl SourceRead,
    member: &Member,
    count: usize,
) -> Result<(), Denial> {
    let work = member
        .comparison_work(count)
        .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
    read.checkpoint(work)
}

pub(in crate::derived_relation) fn insert_member(
    read: &mut impl SourceRead,
    targets: &mut BTreeSet<Member>,
    member: Member,
) -> Result<(), Denial> {
    member_work(read, &member, targets.len())?;
    if !targets.contains(&member) {
        read.retain(size_of::<Member>() + 4 * size_of::<usize>())?;
        member_work(read, &member, targets.len())?;
        targets.insert(member);
    }
    Ok(())
}
