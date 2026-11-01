//! Both images route only bounded incidence; no component or universe census.
use super::*;
impl<S: SceneFaceSchemaBinding> TriangleFaceRule<S> {
    pub(super) fn prepare(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, S>,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<Vec<FaceEntity<S>>, WorthQueryApplicationInvariantPreparationError> {
        checkpoint(budget)?;
        let mut affected = BTreeMap::new();
        for view in [planner.committed(), planner.proposed()] {
            for face in view.touched_entities_of(&self.faces).map_err(preparation)? {
                affected.insert(face.entity_id(), face);
            }
            for body in view
                .touched_entities_of(&self.bodies)
                .map_err(preparation)?
            {
                let mut layer = vec![body];
                // Every vertex in a triangle is at most two predecessors from
                // its anchor. Old edges matter when a rewire removes incidence.
                for depth in 0..3 {
                    let mut previous = Vec::new();
                    for body in layer {
                        checkpoint(budget)?;
                        for anchor in view
                            .relations_to(&self.anchor, &body)
                            .map_err(preparation)?
                        {
                            let face = anchor.from().clone();
                            affected.insert(face.entity_id(), face);
                        }
                        if depth < 2 {
                            for edge in view
                                .relations_to(&self.successor, &body)
                                .map_err(preparation)?
                            {
                                previous.try_reserve(1).map_err(preparation)?;
                                previous.push(edge.from().clone());
                            }
                        }
                    }
                    layer = previous;
                }
            }
        }
        let proposed = planner.proposed();
        let mut scope = Vec::new();
        for selector in affected.into_values() {
            checkpoint(budget)?;
            if let Some(face) = proposed
                .reobserve_entity(&self.faces, &selector)
                .map_err(preparation)?
            {
                // Admit precisely the relation/endpoint reads evaluation consumes.
                let _ = self
                    .contour_is_triangle(&proposed, &face)
                    .map_err(preparation)?;
                scope.try_reserve(1).map_err(preparation)?;
                scope.push(face);
            }
        }
        Ok(scope)
    }
}
fn checkpoint(
    budget: Option<&dyn CustomInvariantLeaseBudget>,
) -> Result<(), WorthQueryApplicationInvariantPreparationError> {
    if budget.is_some_and(|b| !b.checkpoint(1)) {
        Err(preparation("face preparation stopped"))
    } else {
        Ok(())
    }
}
fn preparation(cause: impl std::fmt::Display) -> WorthQueryApplicationInvariantPreparationError {
    WorthQueryApplicationInvariantPreparationError::new(cause.to_string())
}
