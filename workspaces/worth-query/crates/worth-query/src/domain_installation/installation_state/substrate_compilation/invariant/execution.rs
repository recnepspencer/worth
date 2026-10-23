//! Checked execution of the installed outgoing-relation invariant.
use super::*;
use worth_relational::facade::runtime::CustomInvariantLeaseBudget;

impl CustomInvariantRule for InstalledRequiresOutgoingRelationsRule {
    type Scope = RequiresOutgoingRelationsScope;
    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new(self.rule_id.clone()),
                semantic_version: self.semantic_version,
            },
            display_name: self.display_name.clone(),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: Some(self.maximum_work_units),
                access: worth_relational::facade::runtime::CustomInvariantAccessContract {
                    read_entity_kinds: self.relevant_entity_kinds.clone(),
                    read_relation_kinds: self.required_relation_kinds.clone(),
                    affected_entity_kinds: self.relevant_entity_kinds.clone(),
                    affected_relation_kinds: self.required_relation_kinds.clone(),
                    include_relation_endpoint_entity_touches: true,
                }
                .canonicalize(),
                execution_point: InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
            },
        }
    }

    fn supports_checked_preparation(&self) -> bool {
        true
    }
    fn prepare_scope(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        self.prepare_under(planner, None)
    }
    fn prepare_scope_checked(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, CustomInvariantPreparationError>> {
        Some(self.prepare_under(planner, Some(budget)))
    }
    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        self.evaluate_under(context, scope, None)
    }
    fn evaluate_checked(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<CustomInvariantVerdict, CustomInvariantExecutionError>> {
        Some(self.evaluate_under(context, scope, Some(budget)))
    }
}

impl InstalledRequiresOutgoingRelationsRule {
    fn prepare_under(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<RequiresOutgoingRelationsScope, CustomInvariantPreparationError> {
        preparation_checkpoint(budget, 0)?;
        let touched = planner.touched();
        let mut visible_relevant_entities = Vec::new();
        for entity in touched.visible_entity_ids() {
            preparation_checkpoint(budget, 1)?;
            let observed = planner.relations().entity_kind(*entity);
            preparation_checkpoint(budget, 0)?;
            let Ok(kind) = observed else {
                continue;
            };
            if kind_in_preparation(kind, &self.relevant_entity_kinds, budget)? {
                if budget.is_some_and(|budget| {
                    !budget.claim_scratch(std::mem::size_of::<EntityId>() as u64)
                }) {
                    return Err(CustomInvariantPreparationError::new("caller stopped"));
                }
                visible_relevant_entities.push(*entity);
            }
        }
        let traversal = planner
            .traversal()
            .walk_outgoing_from(&visible_relevant_entities, self.traversal_depth as usize)?;
        let mut planned_relevant_entity_count = 0;
        for create in touched.planned_entity_creates() {
            preparation_checkpoint(budget, 1)?;
            if kind_in_preparation(create.kind_id(), &self.relevant_entity_kinds, budget)? {
                planned_relevant_entity_count += 1;
            }
        }
        let mut planned_relevant_relation_count = 0;
        for create in touched.planned_relation_creates() {
            preparation_checkpoint(budget, 1)?;
            if kind_in_preparation(create.kind_id(), &self.required_relation_kinds, budget)? {
                planned_relevant_relation_count += 1;
            }
        }
        Ok(RequiresOutgoingRelationsScope {
            visible_relevant_entities,
            planned_relevant_entity_count,
            planned_relevant_relation_count,
            traversal_exhausted: traversal.frontier_exhausted(),
        })
    }

    fn evaluate_under(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &RequiresOutgoingRelationsScope,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        execution_checkpoint(budget, 0)?;
        if !scope.traversal_exhausted {
            return Ok(CustomInvariantVerdict::Violation(
                worth_relational::facade::runtime::CustomInvariantRefusal::Empty,
            ));
        }
        if scope.visible_relevant_entities.is_empty()
            && scope.planned_relevant_entity_count == 0
            && scope.planned_relevant_relation_count == 0
        {
            return Ok(CustomInvariantVerdict::Pass);
        }
        for entity in &scope.visible_relevant_entities {
            execution_checkpoint(budget, 1)?;
            if !self.visible_entity_satisfies(context, *entity, budget)? {
                return Ok(CustomInvariantVerdict::Violation(
                    worth_relational::facade::runtime::CustomInvariantRefusal::Empty,
                ));
            }
        }
        Ok(CustomInvariantVerdict::Pass)
    }

    fn visible_entity_satisfies(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        entity: EntityId,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<bool, CustomInvariantExecutionError> {
        let mut actual = Vec::new();
        for relation_id in context.relations().outgoing_relations_for_entity(entity)? {
            execution_checkpoint(budget, 1)?;
            let relation = context.relations().relation(relation_id).map_err(|error| {
                CustomInvariantExecutionError::new(format!(
                    "required relation read failed: {error:?}"
                ))
            })?;
            if budget
                .is_some_and(|budget| !budget.claim_scratch(std::mem::size_of::<KindId>() as u64))
            {
                return Err(CustomInvariantExecutionError::new("caller stopped"));
            }
            actual.push(relation.kind_id);
        }
        for required in &self.required_relation_kinds {
            execution_checkpoint(budget, 1)?;
            let mut found = false;
            for kind in &actual {
                execution_checkpoint(budget, 1)?;
                if kind == required {
                    found = true;
                    break;
                }
            }
            if !found {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

fn kind_in_preparation(
    kind: KindId,
    declared: &[KindId],
    budget: Option<&dyn CustomInvariantLeaseBudget>,
) -> Result<bool, CustomInvariantPreparationError> {
    for candidate in declared {
        preparation_checkpoint(budget, 1)?;
        if *candidate == kind {
            return Ok(true);
        }
    }
    Ok(false)
}
fn preparation_checkpoint(
    budget: Option<&dyn CustomInvariantLeaseBudget>,
    units: u64,
) -> Result<(), CustomInvariantPreparationError> {
    if budget.is_some_and(|budget| !budget.checkpoint(units)) {
        return Err(CustomInvariantPreparationError::new("caller stopped"));
    }
    Ok(())
}
fn execution_checkpoint(
    budget: Option<&dyn CustomInvariantLeaseBudget>,
    units: u64,
) -> Result<(), CustomInvariantExecutionError> {
    if budget.is_some_and(|budget| !budget.checkpoint(units)) {
        return Err(CustomInvariantExecutionError::new("caller stopped"));
    }
    Ok(())
}
