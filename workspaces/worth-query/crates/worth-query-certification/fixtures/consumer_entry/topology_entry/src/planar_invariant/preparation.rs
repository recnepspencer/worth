//! Prepare the affected planar triples, observing the reads actually made.
use super::*;

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
        let mut roots = Vec::new();
        for entity in view.touched_entities(&self.x).map_err(preparation_error)? {
            preparation_checkpoint(budget, PLANAR_CANDIDATE_WORK)?;
            self.observe(PlanarInvariantReadEvent::Touched {
                entity: entity.entity_id(),
            });
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
                claim_triangle::<Schema>(budget)?;
                return Ok(vec![[entity.clone(), entity.clone(), entity]]);
            }
            insert_distinct(&mut roots, entity.clone(), budget)?;
            let mut previous = entity;
            for _ in 0..2 {
                preparation_checkpoint(budget, PLANAR_CANDIDATE_WORK)?;
                let predecessor = exactly_one(
                    view.relations_to(&self.successor, &previous)
                        .map_err(preparation_error)?,
                    "predecessor",
                )?;
                previous = predecessor.from().clone();
                insert_distinct(&mut roots, previous.clone(), budget)?;
            }
        }
        let mut triples = Vec::new();
        for root in roots {
            preparation_checkpoint(budget, PLANAR_CANDIDATE_WORK)?;
            let before = planner.reads();
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
            self.observe(PlanarInvariantReadEvent::ForwardHops {
                root: root.entity_id(),
                before,
                after: planner.reads(),
            });
            claim_triangle::<Schema>(budget)?;
            triples.reserve_exact(1);
            triples.push([root, middle, second.to().clone()]);
        }
        Ok(triples)
    }
}

fn insert_distinct<T: PartialEq>(
    values: &mut Vec<T>,
    value: T,
    budget: Option<&dyn CustomInvariantLeaseBudget>,
) -> Result<(), WorthQueryApplicationInvariantPreparationError> {
    for previous in values.iter() {
        preparation_checkpoint(budget, PLANAR_CANDIDATE_WORK)?;
        if previous == &value {
            return Ok(());
        }
    }
    if budget.is_some_and(|budget| !budget.claim_scratch(std::mem::size_of::<T>() as u64)) {
        return Err(WorthQueryApplicationInvariantPreparationError::new(
            "planar root storage refused",
        ));
    }
    values.reserve_exact(1);
    values.push(value);
    Ok(())
}

fn claim_triangle<Schema: TopologyModelBinding>(
    budget: Option<&dyn CustomInvariantLeaseBudget>,
) -> Result<(), WorthQueryApplicationInvariantPreparationError> {
    let bytes =
        std::mem::size_of::<[WorthQueryApplicationInvariantEntity<Schema, Body>; 3]>() as u64;
    if budget.is_some_and(|budget| !budget.claim_result(bytes)) {
        return Err(WorthQueryApplicationInvariantPreparationError::new(
            "planar triangle storage refused",
        ));
    }
    Ok(())
}

fn preparation_checkpoint(
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

fn exactly_one<Schema: TopologyModelBinding>(
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

fn preparation_error(
    error: WorthQueryInvariantAccessDenial,
) -> WorthQueryApplicationInvariantPreparationError {
    WorthQueryApplicationInvariantPreparationError::new(error.to_string())
}
