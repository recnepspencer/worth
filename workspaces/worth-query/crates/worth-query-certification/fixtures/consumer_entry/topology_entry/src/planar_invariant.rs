use super::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_query_host::facade::application_invariants::*;

pub const INVARIANT_PROBE_STANDARD: usize = 0;
pub const INVARIANT_PROBE_UNDECLARED_ACCESS: usize = 1;
pub const INVARIANT_PROBE_EVALUATION_EXPANSION: usize = 2;

pub(crate) fn resolve_rule<Schema: TopologySchemaBinding>(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, Schema>,
    probe: Arc<AtomicUsize>,
    mode: Arc<AtomicUsize>,
) -> Result<PositiveTurnRule<Schema>, String> {
    let declared_successor = PlanarSuccessor::reference::<Schema>();
    let forged_endpoints = worth_query_decl::facade::application_schema::ApplicationRelationRef::<
        Schema,
        PlanarSuccessor,
        Body,
        Body,
    >::from_schema_identifiers(
        declared_successor.name(),
        "NotBody",
        declared_successor.to(),
        declared_successor.integrity(),
    );
    if resolver.typed_relation(forged_endpoints).is_some() {
        return Err("relation binding accepted forged endpoint names".to_owned());
    }
    let x = resolver
        .typed_field(PositionX::reference::<Schema>())
        .ok_or("missing position x")?;
    let y = resolver
        .typed_field(PositionY::reference::<Schema>())
        .ok_or("missing position y")?;
    let successor = resolver
        .typed_relation(declared_successor)
        .ok_or("missing successor")?;
    let undeclared = resolver
        .typed_relation(UndeclaredPlanarRelation::reference::<Schema>())
        .ok_or("missing undeclared-access probe relation")?;
    Ok(PositiveTurnRule {
        successor,
        undeclared,
        x,
        y,
        probe,
        mode,
    })
}

pub struct PositiveTurnRule<Schema: TopologySchemaBinding> {
    successor: WorthQueryApplicationInvariantRelationBinding<Schema, PlanarSuccessor, Body, Body>,
    undeclared:
        WorthQueryApplicationInvariantRelationBinding<Schema, UndeclaredPlanarRelation, Body, Body>,
    x: WorthQueryApplicationInvariantFieldBinding<Schema, Body, PositiveLength>,
    y: WorthQueryApplicationInvariantFieldBinding<Schema, Body, PositiveLength>,
    probe: Arc<AtomicUsize>,
    mode: Arc<AtomicUsize>,
}

impl<Schema: TopologySchemaBinding> WorthQueryApplicationInvariantRule<Schema>
    for PositiveTurnRule<Schema>
{
    type Scope = Vec<[WorthQueryApplicationInvariantEntity<Schema, Body>; 3]>;

    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Schema>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError> {
        self.probe.fetch_add(1, Ordering::SeqCst);
        let view = planner.proposed();
        let mode = self.mode.load(Ordering::SeqCst);
        let mut roots = Vec::new();
        for entity in view.touched_entities(&self.x).map_err(preparation_error)? {
            if mode == INVARIANT_PROBE_UNDECLARED_ACCESS {
                return match view.relations_from(&self.undeclared, &entity) {
                    Err(error)
                        if error.kind()
                            == WorthQueryInvariantAccessDenialKind::OutsideDeclaredAccess =>
                    {
                        Err(preparation_error(error))
                    }
                    Ok(_) => Ok(Vec::new()),
                    Err(_) => Ok(Vec::new()),
                };
            }
            if mode == INVARIANT_PROBE_EVALUATION_EXPANSION {
                return Ok(vec![[entity.clone(), entity.clone(), entity]]);
            }
            insert_distinct(&mut roots, entity.clone());
            let mut previous = entity;
            for _ in 0..2 {
                let predecessor = exactly_one(
                    view.relations_to(&self.successor, &previous)
                        .map_err(preparation_error)?,
                    "predecessor",
                )?;
                previous = predecessor.from().clone();
                insert_distinct(&mut roots, previous.clone());
            }
        }
        let mut triples = Vec::with_capacity(roots.len());
        for root in roots {
            let first = exactly_one(
                view.relations_from(&self.successor, &root)
                    .map_err(preparation_error)?,
                "successor",
            )?;
            let middle = first.to().clone();
            let second = exactly_one(
                view.relations_from(&self.successor, &middle)
                    .map_err(preparation_error)?,
                "successor",
            )?;
            triples.push([root, middle, second.to().clone()]);
        }
        Ok(triples)
    }

    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Schema>,
        scope: &Self::Scope,
    ) -> Result<WorthQueryApplicationInvariantVerdict, WorthQueryApplicationInvariantExecutionError>
    {
        let proposed = context.proposed();
        if self.mode.load(Ordering::SeqCst) == INVARIANT_PROBE_EVALUATION_EXPANSION {
            let Some(entity) = scope.first().map(|triple| &triple[0]) else {
                return Ok(WorthQueryApplicationInvariantVerdict::Pass);
            };
            return match proposed.relations_from(&self.successor, entity) {
                Err(error)
                    if error.kind()
                        == WorthQueryInvariantAccessDenialKind::OutsidePreparedScope =>
                {
                    Err(WorthQueryApplicationInvariantExecutionError::new(
                        error.to_string(),
                    ))
                }
                Ok(_) => Ok(WorthQueryApplicationInvariantVerdict::Pass),
                Err(_) => Ok(WorthQueryApplicationInvariantVerdict::Pass),
            };
        }
        for triple in scope {
            let mut points = [(0i128, 0i128); 3];
            for (index, entity) in triple.iter().enumerate() {
                points[index] = (
                    i128::from(PositiveLength::get(&required_field(
                        &proposed,
                        &self.x,
                        entity,
                        "position x",
                    )?)),
                    i128::from(PositiveLength::get(&required_field(
                        &proposed,
                        &self.y,
                        entity,
                        "position y",
                    )?)),
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

fn insert_distinct<T: PartialEq>(values: &mut Vec<T>, value: T) {
    if !values.contains(&value) {
        values.push(value);
    }
}

fn exactly_one<Schema: TopologySchemaBinding>(
    relations: Vec<WorthQueryApplicationInvariantRelation<Schema, PlanarSuccessor, Body, Body>>,
    direction: &str,
) -> Result<
    WorthQueryApplicationInvariantRelation<Schema, PlanarSuccessor, Body, Body>,
    WorthQueryApplicationInvariantPreparationError,
> {
    let mut relations = relations.into_iter();
    let Some(relation) = relations.next() else {
        return Err(WorthQueryApplicationInvariantPreparationError::new(
            format!("planar {direction} absent"),
        ));
    };
    if relations.next().is_some() {
        return Err(WorthQueryApplicationInvariantPreparationError::new(
            format!("multiple planar {direction} relations"),
        ));
    }
    Ok(relation)
}

fn required_field<Schema: TopologySchemaBinding>(
    view: &WorthQueryApplicationInvariantReadView<'_, Schema>,
    field: &WorthQueryApplicationInvariantFieldBinding<Schema, Body, PositiveLength>,
    entity: &WorthQueryApplicationInvariantEntity<Schema, Body>,
    name: &str,
) -> Result<PositiveLength, WorthQueryApplicationInvariantExecutionError> {
    view.field(field, entity)
        .map_err(|error| WorthQueryApplicationInvariantExecutionError::new(error.to_string()))?
        .ok_or_else(|| WorthQueryApplicationInvariantExecutionError::new(format!("{name} absent")))
}

fn preparation_error(
    error: WorthQueryInvariantAccessDenial,
) -> WorthQueryApplicationInvariantPreparationError {
    WorthQueryApplicationInvariantPreparationError::new(error.to_string())
}
