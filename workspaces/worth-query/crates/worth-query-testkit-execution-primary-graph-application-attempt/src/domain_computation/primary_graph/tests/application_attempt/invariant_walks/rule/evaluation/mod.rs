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
        WorthQueryApplicationInvariantExecutionError,
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
        WorthQueryApplicationInvariantExecutionError,
    > {
        context
            .check_live()
            .map_err(|stop| failure(format!("{stop:?}")))?;
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
                        .ok_or_else(|| failure("blocked key absent"))?;
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
) -> Result<(), WorthQueryApplicationInvariantExecutionError> {
    if budget.is_some_and(|payer| !payer.checkpoint(1)) {
        Err(failure("walk evaluation stopped"))
    } else {
        Ok(())
    }
}

pub fn access(
    error: WorthQueryInvariantAccessDenial,
) -> WorthQueryApplicationInvariantExecutionError {
    failure(error.to_string())
}

pub fn failure(message: impl Into<String>) -> WorthQueryApplicationInvariantExecutionError {
    WorthQueryApplicationInvariantExecutionError::new(message.into())
}
