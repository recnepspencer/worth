//! Select affected Native islands, then use Native successor orbits for geometry.
use super::*;
use std::collections::BTreeSet;

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
        let mut windows = PreparedTriangles {
            touched: touched.iter().map(|entity| entity.entity_id()).collect(),
            ordered: BTreeSet::new(),
            triples: Vec::new(),
        };
        let mut covered = BTreeSet::new();
        for member in &touched {
            preparation_checkpoint(budget, PLANAR_CANDIDATE_WORK)?;
            if covered.contains(&member.entity_id()) {
                continue;
            }
            let before = planner.reads();
            let component = view
                .component_of(&self.islands, member)
                .map_err(preparation_error)?;
            self.observe(PlanarInvariantReadEvent::CompleteComponent {
                before,
                after: planner.reads(),
                visited: component.len(),
                identity: *component.identity().bytes(),
            });
            for root in component.members() {
                preparation_checkpoint(budget, PLANAR_CANDIDATE_WORK)?;
                covered.insert(root.entity_id());
                if windows.ordered.contains(&root.entity_id()) {
                    continue;
                }
                let before = planner.reads();
                let orbit = view
                    .orbit(&self.successor, &root)
                    .map_err(preparation_error)?;
                self.observe(PlanarInvariantReadEvent::CompleteOrbit {
                    before,
                    after: planner.reads(),
                    visited: orbit.len(),
                    physical_relations: orbit.relations().len(),
                    identity: *orbit.identity().bytes(),
                });
                windows.include(&orbit, budget)?;
            }
        }
        preparation_checkpoint(budget, 0)?;
        Ok(windows.triples)
    }
}

/// Transient deduplication and geometry windows; Native retains topology truth.
struct PreparedTriangles<S> {
    touched: BTreeSet<EntityId>,
    ordered: BTreeSet<EntityId>,
    triples: Vec<[WorthQueryApplicationInvariantEntity<S, Body>; 3]>,
}
impl<S> PreparedTriangles<S> {
    fn include(
        &mut self,
        orbit: &WorthQueryApplicationInvariantOrbit<S, PlanarSuccessor, Body>,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<(), WorthQueryApplicationInvariantPreparationError> {
        for (index, root) in orbit.members().enumerate() {
            preparation_checkpoint(budget, PLANAR_CANDIDATE_WORK)?;
            self.ordered.insert(root.entity_id());
            let middle = orbit
                .member((index + 1) % orbit.len())
                .expect("complete orbit");
            let next = orbit
                .member((index + 2) % orbit.len())
                .expect("complete orbit");
            // A geometric triple changes iff one of its vertices changed.
            if [&root, &middle, &next]
                .iter()
                .any(|entity| self.touched.contains(&entity.entity_id()))
            {
                self.triples.try_reserve(1).map_err(|_| {
                    WorthQueryApplicationInvariantPreparationError::new(
                        "planar triangle allocation refused",
                    )
                })?;
                self.triples.push([root, middle, next]);
            }
        }
        Ok(())
    }
}

fn triples_storage<Schema: TopologyModelBinding>(
    count: usize,
    budget: Option<&dyn CustomInvariantLeaseBudget>,
) -> Result<
    Vec<[WorthQueryApplicationInvariantEntity<Schema, Body>; 3]>,
    WorthQueryApplicationInvariantPreparationError,
> {
    preparation_checkpoint(budget, 0)?;
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
