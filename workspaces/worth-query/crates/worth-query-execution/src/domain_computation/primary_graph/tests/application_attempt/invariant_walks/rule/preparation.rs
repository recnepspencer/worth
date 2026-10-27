//! Group membership selects roots without enumerating the tested path.
use super::*;
use std::collections::BTreeMap;

impl Rule {
    pub(super) fn prepare(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, WalkSchema>,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<Vec<Root>, WorthQueryApplicationInvariantPreparationError> {
        planner
            .check_live()
            .map_err(|stop| failure(format!("{stop:?}")))?;
        let view = planner.proposed();
        let roots = self.discover_roots(&view, budget)?;
        let mut prepared = Vec::new();
        for entity in roots.into_values() {
            checkpoint(budget)?;
            let key = view
                .field(&self.key, &entity)
                .map_err(access)?
                .ok_or_else(|| failure("root key absent"))?;
            if view.field(&self.active, &entity).map_err(access)? == Some(true) {
                view.walk(&self.next, RelationDirection::Forward, &entity)
                    .map_err(access)?;
            }
            if budget
                .is_some_and(|payer| !payer.claim_result((4 * std::mem::size_of::<Root>()) as u64))
            {
                return Err(failure("root result retention refused"));
            }
            prepared.push(Root { entity, key });
        }
        self.observation
            .prepared(prepared.iter().map(|root| root.entity.clone()));
        Ok(prepared)
    }

    fn discover_roots(
        &self,
        view: &WorthQueryApplicationInvariantReadView<'_, WalkSchema>,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<BTreeMap<EntityId, Entity>, WorthQueryApplicationInvariantPreparationError> {
        let mut roots = BTreeMap::new();
        for node in view.touched_entities(&self.key).map_err(access)? {
            checkpoint(budget)?;
            let membership = view
                .relations_from(&self.membership, &node)
                .map_err(access)?;
            let [membership] = membership.as_slice() else {
                return Err(failure("each node must have one authored root membership"));
            };
            let root = membership.to();
            if !roots.contains_key(&root.entity_id()) {
                let bytes = 11 * std::mem::size_of::<(EntityId, Entity)>()
                    + 14 * std::mem::size_of::<usize>();
                if budget.is_some_and(|payer| !payer.claim_scratch(bytes as u64)) {
                    return Err(failure("root discovery retention refused"));
                }
                roots.insert(root.entity_id(), root.clone());
            }
        }
        Ok(roots)
    }
}

fn checkpoint(
    budget: Option<&dyn CustomInvariantLeaseBudget>,
) -> Result<(), WorthQueryApplicationInvariantPreparationError> {
    if budget.is_some_and(|payer| !payer.checkpoint(1)) {
        Err(failure("root preparation stopped"))
    } else {
        Ok(())
    }
}

fn access(
    error: WorthQueryInvariantAccessDenial,
) -> WorthQueryApplicationInvariantPreparationError {
    failure(error.to_string())
}

fn failure(message: impl Into<String>) -> WorthQueryApplicationInvariantPreparationError {
    WorthQueryApplicationInvariantPreparationError::new(message.into())
}
