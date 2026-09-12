use super::*;
use std::collections::BTreeSet;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_foundational::facade::ContractValidatedAspectValueView;
use worth_query_host::facade::application_invariants::*;

pub(crate) fn resolve_rule<Schema: TopologySchemaBinding>(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, Schema>,
    probe: Arc<AtomicUsize>,
) -> Result<PositiveTurnRule, String> {
    let (body, x) = resolver
        .field(PositionX::reference::<Schema>())
        .ok_or("missing position x")?;
    let (_, y) = resolver
        .field(PositionY::reference::<Schema>())
        .ok_or("missing position y")?;
    let (relation, _, _) = resolver
        .relation(PlanarSuccessor::reference::<Schema>())
        .ok_or("missing successor")?;
    Ok(PositiveTurnRule {
        body,
        relation,
        x,
        y,
        probe,
    })
}
pub struct PositiveTurnRule {
    body: KindId,
    relation: KindId,
    x: AspectFieldLocator,
    y: AspectFieldLocator,
    probe: Arc<AtomicUsize>,
}
impl WorthQueryApplicationInvariantRule for PositiveTurnRule {
    type Scope = Vec<[EntityId; 3]>;
    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError> {
        self.probe.fetch_add(1, Ordering::SeqCst);
        let mut roots = BTreeSet::new();
        let relations = planner.relations();
        for entity in planner.touched().visible_entity_ids() {
            if relations.entity_kind(*entity) != Some(self.body) {
                continue;
            }
            roots.insert(*entity);
            // A changed vertex affects its own turn and the two predecessor turns.
            let mut previous = *entity;
            for _ in 0..2 {
                let incoming = relations.incoming_relations_for_entity(previous)?;
                let predecessor = incoming
                    .iter().copied()
                    .filter_map(|id| relations.relation(id))
                    .find(|relation| relation.kind_id == self.relation)
                    .ok_or_else(|| {
                        WorthQueryApplicationInvariantPreparationError::new(
                            format!("planar predecessor absent: entity={previous:?}, expected_kind={:?}, incoming_count={}, received={:?}", self.relation, incoming.len(), incoming.iter().map(|id| (*id,relations.relation(*id).map(|record|record.kind_id))).collect::<Vec<_>>()),
                        )
                    })?;
                roots.insert(predecessor.source);
                previous = predecessor.source;
            }
        }
        let mut triples = Vec::with_capacity(roots.len());
        for root in roots {
            let mut triple = [root; 3];
            for index in 1..3 {
                let outgoing = relations.outgoing_relations_for_entity(triple[index - 1])?;
                let successor = outgoing
                    .iter().copied()
                    .filter_map(|id| relations.relation(id))
                    .find(|relation| relation.kind_id == self.relation)
                    .ok_or_else(|| {
                        WorthQueryApplicationInvariantPreparationError::new(
                            format!("planar successor absent: entity={:?}, expected_kind={:?}, outgoing_count={}, received={:?}", triple[index-1], self.relation, outgoing.len(), outgoing.iter().map(|id| (*id,relations.relation(*id).map(|record|record.kind_id))).collect::<Vec<_>>()),
                        )
                    })?;
                triple[index] = successor.target;
            }
            triples.push(triple);
        }
        Ok(triples)
    }
    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_>,
        scope: &Self::Scope,
    ) -> Result<WorthQueryApplicationInvariantVerdict, WorthQueryApplicationInvariantExecutionError>
    {
        for triple in scope {
            let mut points = [(0i128, 0i128); 3];
            for (index, entity) in triple.iter().enumerate() {
                let states = context.aspect_states();
                let state = states.entity_aspect_state(*entity).ok_or_else(|| {
                    WorthQueryApplicationInvariantExecutionError::new(
                        "planar candidate entity absent",
                    )
                })?;
                let coordinate = |locator: &AspectFieldLocator| {
                    let value = state.get(locator.aspect().aspect_key())?;
                    let ContractValidatedAspectValueView::Struct(fields) = value.view() else {
                        return None;
                    };
                    let AspectValue::UInt64(value) =
                        fields.get(locator.field_path().fields().first()?)?
                    else {
                        return None;
                    };
                    Some(i128::from(*value))
                };
                points[index] = (
                    coordinate(&self.x).ok_or_else(|| {
                        WorthQueryApplicationInvariantExecutionError::new("position x absent")
                    })?,
                    coordinate(&self.y).ok_or_else(|| {
                        WorthQueryApplicationInvariantExecutionError::new("position y absent")
                    })?,
                );
            }
            let [a, b, c] = points;
            let turn = (b.0 - a.0).checked_mul(c.1 - b.1).and_then(|left| {
                (b.1 - a.1)
                    .checked_mul(c.0 - b.0)
                    .and_then(|right| left.checked_sub(right))
            });
            let turn = turn.ok_or_else(|| {
                WorthQueryApplicationInvariantExecutionError::new(
                    "coordinate determinant exceeds supported arithmetic",
                )
            })?;
            if turn <= 0 {
                return Ok(WorthQueryApplicationInvariantVerdict::Violation);
            }
        }
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
}
