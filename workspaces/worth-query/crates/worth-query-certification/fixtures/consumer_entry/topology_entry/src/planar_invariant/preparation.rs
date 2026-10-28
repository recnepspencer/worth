//! Prepare affected geometry triples from one complete Native component read.
use super::*;
use worth_query_decl::facade::application_schema::RelationDirection;

impl<Schema: TopologyModelBinding> PositiveTurnRule<Schema> {
    pub(super) fn prepare_with_budget(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Schema>,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<
        <Self as WorthQueryApplicationInvariantRule<Schema>>::Scope,
        WorthQueryApplicationInvariantPreparationError,
    > {
        let outcome = self.prepare_inner(planner, budget);
        self.observe(PlanarInvariantReadEvent::Prepared {
            reads: planner.reads(),
            succeeded: outcome.is_ok(),
        });
        outcome
    }

    fn prepare_inner(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Schema>,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<
        <Self as WorthQueryApplicationInvariantRule<Schema>>::Scope,
        WorthQueryApplicationInvariantPreparationError,
    > {
        preparation_checkpoint(budget, 0)?;
        self.probe.fetch_add(1, Ordering::SeqCst);
        let view = planner.proposed();
        let mode = self.mode.load(Ordering::SeqCst);
        let touched = view.touched_entities(&self.x).map_err(preparation_error)?;
        for entity in &touched {
            preparation_checkpoint(budget, PLANAR_CANDIDATE_WORK)?;
            self.observe(PlanarInvariantReadEvent::Touched {
                entity: entity.entity_id(),
            });
            // Negative controls deliberately prepare no relation scope.
            if mode == INVARIANT_PROBE_UNDECLARED_ACCESS {
                return match view.relations_from(&self.undeclared, entity) {
                    Err(error)
                        if error.kind()
                            == WorthQueryInvariantAccessDenialKind::OutsideDeclaredAccess =>
                    {
                        Err(preparation_error(error))
                    }
                    _ => Ok(Vec::new()),
                };
            }
            if mode == INVARIANT_PROBE_EVALUATION_EXPANSION {
                return triples_storage::<Schema>(1, budget).map(|mut triples| {
                    triples.push([entity.clone(), entity.clone(), entity.clone()]);
                    triples
                });
            }
        }
        if touched.is_empty() {
            return Ok(Vec::new());
        }
        let before = planner.reads();
        let walk = view
            .closure(&self.successor, RelationDirection::Reverse, &touched)
            .map_err(preparation_error)?;
        self.observe(PlanarInvariantReadEvent::CompleteWalk {
            before,
            after: planner.reads(),
            visited: walk.len(),
            physical_relations: walk.relations().len(),
        });
        // Native commit-boundary degree laws run before this custom rule. Its
        // ordinary reached component is a ring. Keep explicit missing/duplicate
        // lookup refusals for callers that do not inherit that ordering.
        let successors = successors::Successors::from_walk(&walk, budget)?;
        let mut root_count = 0usize;
        for step in &walk {
            preparation_checkpoint(budget, PLANAR_CANDIDATE_WORK)?;
            if step.distance() <= 2 {
                root_count = root_count.checked_add(1).ok_or_else(|| {
                    WorthQueryApplicationInvariantPreparationError::new(
                        "planar root count overflow",
                    )
                })?;
            }
        }
        let mut triples = triples_storage::<Schema>(root_count, budget)?;
        for step in &walk {
            preparation_checkpoint(budget, PLANAR_CANDIDATE_WORK)?;
            // Two is the geometric window, never a Native traversal limit.
            if step.distance() > 2 {
                continue;
            }
            let root = step.entity();
            let middle = successors.next(root, budget)?;
            let next = successors.next(&middle, budget)?;
            triples.push([root.clone(), middle, next]);
        }
        preparation_checkpoint(budget, 0)?;
        Ok(triples)
    }
}

fn triples_storage<Schema: TopologyModelBinding>(
    count: usize,
    budget: Option<&dyn CustomInvariantLeaseBudget>,
) -> Result<
    Vec<[WorthQueryApplicationInvariantEntity<Schema, Body>; 3]>,
    WorthQueryApplicationInvariantPreparationError,
> {
    let bytes = count
        .checked_mul(std::mem::size_of::<
            [WorthQueryApplicationInvariantEntity<Schema, Body>; 3],
        >())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(|| {
            WorthQueryApplicationInvariantPreparationError::new("planar triangle storage overflow")
        })?;
    if budget.is_some_and(|budget| !budget.claim_result(bytes)) {
        return Err(WorthQueryApplicationInvariantPreparationError::new(
            "planar triangle storage refused",
        ));
    }
    let mut triples = Vec::new();
    triples.try_reserve_exact(count).map_err(|_| {
        WorthQueryApplicationInvariantPreparationError::new("planar triangle allocation refused")
    })?;
    Ok(triples)
}

pub(super) fn preparation_checkpoint(
    budget: Option<&dyn CustomInvariantLeaseBudget>,
    work: u64,
) -> Result<(), WorthQueryApplicationInvariantPreparationError> {
    if budget.is_some_and(|budget| !budget.checkpoint(work)) {
        return Err(WorthQueryApplicationInvariantPreparationError::new(
            "planar preparation refused",
        ));
    }
    Ok(())
}
fn preparation_error(
    error: WorthQueryInvariantAccessDenial,
) -> WorthQueryApplicationInvariantPreparationError {
    WorthQueryApplicationInvariantPreparationError::new(error.to_string())
}
