//! Evaluate prepared geometry without expanding its admitted read scope.
use super::*;

impl<Schema: TopologyModelBinding> PositiveTurnRule<Schema> {
    // compiled-example: begin
    pub(super) fn evaluate_with_budget(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Schema>,
        scope: &<Self as WorthQueryApplicationInvariantRule<Schema>>::Scope,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<WorthQueryApplicationInvariantVerdict<()>, WorthQueryApplicationInvariantError>
    {
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
    ) -> Result<WorthQueryApplicationInvariantVerdict<()>, WorthQueryApplicationInvariantError>
    {
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
                    Err(WorthQueryApplicationInvariantError::from(error))
                }
                Ok(_) => Ok(WorthQueryApplicationInvariantVerdict::Pass),
                Err(_) => Ok(WorthQueryApplicationInvariantVerdict::Pass),
            };
        }
        for triple in scope {
            let mut points = [[0u64; 2]; 3];
            for (index, entity) in triple.iter().enumerate() {
                evaluation_checkpoint(budget, PLANAR_CANDIDATE_WORK)?;
                points[index] = [
                    PositiveLength::get(&required_field(&proposed, &self.x, entity, "position x")?),
                    PositiveLength::get(&required_field(&proposed, &self.y, entity, "position y")?),
                ];
            }
            let [a, b, c] = points;
            if !worth_query_consumer_values::integer_orientation(a, b, c).is_gt() {
                return Ok(WorthQueryApplicationInvariantVerdict::Violation(()));
            }
        }
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
}

fn evaluation_checkpoint(
    budget: Option<&dyn CustomInvariantLeaseBudget>,
    work: u64,
) -> Result<(), WorthQueryApplicationInvariantError> {
    if budget.is_some_and(|budget| !budget.checkpoint(work)) {
        return Err(WorthQueryApplicationInvariantError::Stopped);
    }
    Ok(())
}

fn required_field<Schema: TopologyModelBinding>(
    view: &WorthQueryApplicationInvariantReadView<'_, Schema>,
    field: &WorthQueryApplicationInvariantFieldBinding<Schema, Body, PositiveLength>,
    entity: &WorthQueryApplicationInvariantEntity<Schema, Body>,
    name: &'static str,
) -> Result<PositiveLength, WorthQueryApplicationInvariantError> {
    view.field(field, entity)
        .map_err(WorthQueryApplicationInvariantError::from)?
        .ok_or(WorthQueryApplicationInvariantError::MissingRequiredField { field: name })
}
