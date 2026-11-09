//! Judge freshly walked facts inside the actual frozen preparation authority.
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::rule::*;

impl Rule {
    pub fn evaluate_under(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, WalkSchema>,
        scope: &[Root],
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<Refusal>,
        WorthQueryApplicationInvariantError<Refusal>,
    > {
        let outcome = self.judge(context, scope, budget);
        self.observation.evaluated(context.reads());
        outcome
    }

    pub fn judge(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, WalkSchema>,
        scope: &[Root],
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<Refusal>,
        WorthQueryApplicationInvariantError<Refusal>,
    > {
        context
            .check_live()
            .map_err(WorthQueryApplicationInvariantError::from)?;
        let view = context.proposed();
        for root in scope {
            checkpoint(budget)?;
            if view.field(&self.active, &root.entity).map_err(access)? != Some(true) {
                continue;
            }
            // compiled-example: begin
            let reachable = view
                .walk(&self.next, RelationDirection::Forward, &root.entity)
                .map_err(access)?;
            for step in &reachable {
                checkpoint(budget)?;
                if step.distance() >= 3 {
                    self.observation.deep();
                }
                if view.field(&self.blocked, step.entity()).map_err(access)? == Some(true) {
                    let blocked = view
                        .field(&self.key, step.entity())
                        .map_err(access)?
                        .ok_or_else(|| {
                            WorthQueryApplicationInvariantError::MissingRequiredField {
                                field: "blocked key absent",
                            }
                        })?;
                    return Ok(WorthQueryApplicationInvariantVerdict::Violation(Refusal {
                        root: root.key,
                        blocked,
                        distance: step.distance() as u64,
                    }));
                }
            }
            // compiled-example: end
        }
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
}

pub fn checkpoint(
    budget: Option<&dyn CustomInvariantLeaseBudget>,
) -> Result<(), WorthQueryApplicationInvariantError<Refusal>> {
    if budget.is_some_and(|payer| !payer.checkpoint(1)) {
        Err(WorthQueryApplicationInvariantError::Stopped)
    } else {
        Ok(())
    }
}

pub fn access(
    error: impl Into<WorthQueryApplicationInvariantError<Refusal>>,
) -> WorthQueryApplicationInvariantError<Refusal> {
    error.into()
}
