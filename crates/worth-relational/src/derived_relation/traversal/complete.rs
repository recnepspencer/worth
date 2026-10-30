//! Complete source support from the same frontier used by Native components.
use crate::derived_relation::evaluation::SourceRead;
use crate::derived_relation::{
    DerivedRelationAdjacency as Adjacency, DerivedRelationData as Data,
    DerivedRelationDenial as DefinitionDenial, DerivedRelationHandle as Handle,
    DerivedRelationReadDenial as Denial, DerivedRelationWalkStep,
};
use crate::graph_member::NativeMemberKey;
use crate::identity::data::{EntityId, KindId};
use crate::schema::data::RelationalSchemaRegistry;
use crate::validation::data::StructuralRelationRecord;

#[expect(
    clippy::disallowed_methods,
    reason = "Comparison uses declared semantic keys; type names label diagnostics only"
)]
pub(in crate::derived_relation) fn acquire(
    registry: &RelationalSchemaRegistry,
    definition: &Handle,
    root: NativeMemberKey,
    reverse: bool,
    undirected: bool,
    read: &mut impl SourceRead,
) -> Result<(Data, Vec<DerivedRelationWalkStep>), Denial> {
    let mut traversal =
        super::observed_walk(registry, definition, root, reverse, undirected, read)?;
    let mut counts = [0usize; 3];
    for step in &traversal.support {
        read.checkpoint(1)?;
        for (count, length) in counts.iter_mut().zip([
            step.data.entities.len(),
            step.data.relations.len(),
            step.data.adjacency.len(),
        ]) {
            *count = count
                .checked_add(length)
                .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
        }
    }
    let bytes = counts[0]
        .checked_mul(size_of::<(EntityId, KindId)>())
        .and_then(|n| n.checked_add(counts[1].checked_mul(size_of::<StructuralRelationRecord>())?))
        .and_then(|n| n.checked_add(counts[2].checked_mul(size_of::<Adjacency>())?))
        .and_then(|n| {
            n.checked_add(
                traversal
                    .members
                    .len()
                    .checked_mul(size_of::<NativeMemberKey>())?,
            )
        })
        .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
    read.retain(bytes)?;
    let mut entities = Vec::with_capacity(counts[0]);
    let mut relations = Vec::with_capacity(counts[1]);
    let mut adjacency = Vec::with_capacity(counts[2]);
    for step in traversal.support {
        read.checkpoint(1)?;
        entities.extend(step.data.entities);
        relations.extend(step.data.relations);
        adjacency.extend(step.data.adjacency);
    }
    // These are fixed-width physical fact keys. Preserve every support until
    // the complete traversal has succeeded, then canonicalize only the roster.
    for count in counts {
        let levels = (usize::BITS - count.max(1).leading_zeros()) as usize;
        read.checkpoint(
            count
                .checked_mul(levels)
                .and_then(|n| n.checked_mul(11))
                .ok_or(DefinitionDenial::WorkBudgetExceeded)?,
        )?;
    }
    entities.sort_unstable();
    entities.dedup();
    relations.sort_unstable_by_key(|record| record.relation_id);
    relations.dedup_by_key(|record| record.relation_id);
    adjacency.sort_unstable();
    adjacency.dedup();
    let count = traversal.steps.len();
    let levels = (usize::BITS - count.max(1).leading_zeros()) as usize;
    read.checkpoint(
        count
            .checked_mul(levels)
            .and_then(|n| n.checked_mul(11))
            .ok_or(DefinitionDenial::WorkBudgetExceeded)?,
    )?;
    for step in &traversal.steps {
        crate::derived_relation::evaluation::member_work(read, &step.member, count)?;
    }
    // Minimum distances come from the genuine frontier. Canonical order within
    // a distance layer is independent of which parent discovered a child first.
    traversal.steps.sort_unstable_by(|left, right| {
        left.distance
            .cmp(&right.distance)
            .then_with(|| left.member.cmp(&right.member))
    });
    Ok((
        Data {
            endpoints: traversal.members.into_iter().collect(),
            entities,
            relations,
            adjacency,
            reads: read.reads(),
        },
        traversal.steps,
    ))
}
