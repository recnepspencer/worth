//! Prepare diagnostics independently of the safety rule's admission.
use super::*;
impl Rule {
    pub(super) fn prepare(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, WalkSchema>,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<Scope, WorthQueryApplicationInvariantPreparationError> {
        if !self.probe.enabled {
            return Ok(Scope {
                proposed: Vec::new(),
                committed: Vec::new(),
            });
        }
        Ok(Scope {
            proposed: self.nodes(&planner.proposed(), budget)?,
            committed: self.nodes(&planner.committed(), budget)?,
        })
    }
    fn nodes(
        &self,
        view: &WorthQueryApplicationInvariantReadView<'_, WalkSchema>,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<Vec<(u64, Entity)>, WorthQueryApplicationInvariantPreparationError> {
        let mut root = None;
        for entity in view.touched_entities(&self.key).map_err(access)? {
            if view.field(&self.key, &entity).map_err(access)? == Some(0) {
                root = Some(entity);
                break;
            }
        }
        let Some(root) = root else {
            return Ok(Vec::new());
        };
        let memberships = view.relations_to(&self.membership, &root).map_err(access)?;
        if budget.is_some_and(|b| {
            !b.claim_result((4 * memberships.len() * std::mem::size_of::<(u64, Entity)>()) as u64)
        }) {
            return Err(WorthQueryApplicationInvariantPreparationError::new(
                "probe scope retention refused",
            ));
        }
        let mut nodes = Vec::new();
        for membership in memberships {
            if budget.is_some_and(|b| !b.checkpoint(1)) {
                return Err(WorthQueryApplicationInvariantPreparationError::new(
                    "probe preparation stopped",
                ));
            }
            let entity = membership.from().clone();
            let key = view
                .field(&self.key, &entity)
                .map_err(access)?
                .ok_or_else(|| {
                    WorthQueryApplicationInvariantPreparationError::new("node key absent")
                })?;
            nodes.push((key, entity));
        }
        if budget
            .is_some_and(|b| !b.claim_scratch((nodes.len() * std::mem::size_of::<Entity>()) as u64))
        {
            return Err(WorthQueryApplicationInvariantPreparationError::new(
                "probe root retention refused",
            ));
        }
        let roots = nodes
            .iter()
            .map(|(_, node)| node.clone())
            .collect::<Vec<_>>();
        view.closure(&self.next, RelationDirection::Forward, &roots)
            .map_err(access)?;
        Ok(nodes)
    }
}
fn access(e: WorthQueryInvariantAccessDenial) -> WorthQueryApplicationInvariantPreparationError {
    WorthQueryApplicationInvariantPreparationError::new(e.to_string())
}
