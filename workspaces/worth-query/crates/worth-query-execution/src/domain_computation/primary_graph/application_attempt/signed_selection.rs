//! One candidate-adjacency comparison serves both native minting and settled supersession.
use super::{
    WorthQueryApplicationObservedRelation as Edge, WorthQueryApplicationRealizedEffect as Effect,
};
use worth_query_declaration::facade::application_query::ApplicationQueryRootPathDirection as Direction;
use worth_relational::facade::{
    identity::{EntityId, KindId},
    transactions::EntityReference,
};

pub(super) fn preserves_endpoint(
    edges: &[Edge],
    effects: &[Effect],
    anchor: EntityId,
    kind: KindId,
    direction: Direction,
    selected: EntityId,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<bool, worth_execution::MapKernelStop> {
    let scan = u64::try_from(effects.len())
        .unwrap_or(u64::MAX)
        .saturating_add(1);
    context.checkpoint(scan)?;
    if deleted(effects, anchor) || deleted(effects, selected) {
        return Ok(false);
    }
    let mut present = false;
    for edge in edges {
        context.checkpoint(scan)?;
        let target = match direction {
            Direction::Forward => edge.to,
            Direction::Reverse => edge.from,
        };
        if effects.iter().any(|effect| matches!(effect, Effect::DeleteRelation {relation_id} if *relation_id == edge.relation_id)) || deleted(effects, target) {continue;}
        if target != selected {
            return Ok(false);
        }
        present = true;
    }
    for effect in effects {
        context.checkpoint(1)?;
        if let Effect::CreateRelation {
            kind: written,
            from,
            to,
            ..
        } = effect
            && *written == kind
        {
            let (parent, target) = match direction {
                Direction::Forward => (from, to),
                Direction::Reverse => (to, from),
            };
            if !matches!(parent, EntityReference::Existing(entity) if *entity == anchor) {
                continue;
            }
            match target {
                EntityReference::Existing(target) => {
                    context.checkpoint(scan)?;
                    if deleted(effects, *target) {
                        continue;
                    }
                    if *target != selected {
                        return Ok(false);
                    }
                    present = true;
                }
                EntityReference::Created(_) => return Ok(false),
            }
        }
    }
    Ok(present)
}
fn deleted(effects: &[Effect], entity: EntityId) -> bool {
    effects
        .iter()
        .any(|effect| matches!(effect, Effect::DeleteEntity {entity_id} if *entity_id == entity))
}
