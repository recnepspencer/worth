//! Prepare diagnostics independently of the safety rule's admission.
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::probe::*;
impl Rule {
    pub fn prepare(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, WalkSchema>,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<Scope, WorthQueryApplicationInvariantError> {
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
    pub fn nodes(
        &self,
        view: &WorthQueryApplicationInvariantReadView<'_, WalkSchema>,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<Vec<(u64, Entity)>, WorthQueryApplicationInvariantError> {
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
            return Err(WorthQueryApplicationInvariantError::Stopped);
        }
        let mut nodes = Vec::new();
        for membership in memberships {
            if budget.is_some_and(|b| !b.checkpoint(1)) {
                return Err(WorthQueryApplicationInvariantError::Stopped);
            }
            let entity = membership.from().clone();
            let key = view
                .field(&self.key, &entity)
                .map_err(access)?
                .ok_or_else(
                    || WorthQueryApplicationInvariantError::MissingRequiredField {
                        field: "node key absent",
                    },
                )?;
            nodes.push((key, entity));
        }
        if budget
            .is_some_and(|b| !b.claim_scratch((nodes.len() * std::mem::size_of::<Entity>()) as u64))
        {
            return Err(WorthQueryApplicationInvariantError::Stopped);
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
pub fn access(
    e: impl Into<WorthQueryApplicationInvariantError>,
) -> WorthQueryApplicationInvariantError {
    e.into()
}
