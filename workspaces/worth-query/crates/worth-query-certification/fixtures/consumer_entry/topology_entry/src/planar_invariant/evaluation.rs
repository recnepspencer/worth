//! Evaluate prepared geometry without expanding its admitted read scope.
use super::*;

impl<Schema: TopologyModelBinding> PositiveTurnRule<Schema> {
    // compiled-example: begin
    pub(super) fn evaluate_with_budget(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Schema>,
        scope: &<Self as WorthQueryApplicationInvariantRule<Schema>>::Scope,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<()>,
        WorthQueryApplicationInvariantExecutionError,
    > {
        let outcome = self.evaluate_inner(context, scope, budget);
        self.observe(PlanarInvariantReadEvent::Evaluated {
            reads: context.reads(),
            returned_verdict: outcome.is_ok(),
        });
        outcome
    }
    // compiled-example: end

    fn evaluate_inner(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Schema>,
        scope: &<Self as WorthQueryApplicationInvariantRule<Schema>>::Scope,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<()>,
        WorthQueryApplicationInvariantExecutionError,
    > {
        evaluation_checkpoint(budget, 0)?;
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
                evaluation_checkpoint(budget, PLANAR_CANDIDATE_WORK)?;
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
                return Ok(WorthQueryApplicationInvariantVerdict::Violation(()));
            }
        }
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
}

fn evaluation_checkpoint(
    budget: Option<&dyn CustomInvariantLeaseBudget>,
    work: u64,
) -> Result<(), WorthQueryApplicationInvariantExecutionError> {
    if budget.is_some_and(|budget| !budget.checkpoint(work)) {
        return Err(WorthQueryApplicationInvariantExecutionError::new(
            "planar evaluation refused",
        ));
    }
    Ok(())
}

fn required_field<Schema: TopologyModelBinding>(
    view: &WorthQueryApplicationInvariantReadView<'_, Schema>,
    field: &WorthQueryApplicationInvariantFieldBinding<Schema, Body, PositiveLength>,
    entity: &WorthQueryApplicationInvariantEntity<Schema, Body>,
    name: &str,
) -> Result<PositiveLength, WorthQueryApplicationInvariantExecutionError> {
    view.field(field, entity)
        .map_err(|error| WorthQueryApplicationInvariantExecutionError::new(error.to_string()))?
        .ok_or_else(|| WorthQueryApplicationInvariantExecutionError::new(format!("{name} absent")))
}
