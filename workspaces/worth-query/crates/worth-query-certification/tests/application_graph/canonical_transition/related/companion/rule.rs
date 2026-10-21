//! Actual proposed companion edge judgment selected by the target program.
use super::{super::schema::CanonicalSchema, schema::*};
use worth_query_host::facade::{application_invariants::*, declaration::application_schema::*};
pub struct CompanionRule;
impl ApplicationInvariantMarkerIdentity<CanonicalSchema> for CompanionRule {
    const IDENTIFIER: &'static str = "canonical-companion-link";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}
pub fn definition() -> ApplicationInvariantDefinition<CanonicalSchema, CompanionRule> {
    ApplicationInvariantDefinition::new(
        CompanionRule::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        RelationalInvariantWorkBudget::new(Some(std::num::NonZeroU64::new(4096).unwrap())),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [ApplicationInvariantGroup::SchemaCompliance],
            [
                ApplicationInvariantScopeTarget::Entity("Companion".into()),
                ApplicationInvariantScopeTarget::Relation("CompanionLink".into()),
            ],
            [ApplicationInvariantScopeTarget::Relation(
                "CompanionLink".into(),
            )],
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
}
type Entity = WorthQueryApplicationInvariantEntity<CanonicalSchema, Companion>;
type Field = WorthQueryApplicationInvariantFieldBinding<CanonicalSchema, Companion, String>;
type Edge = WorthQueryApplicationInvariantRelationBinding<
    CanonicalSchema,
    CompanionLink,
    Companion,
    Companion,
>;
pub struct Rule {
    field: Field,
    edge: Edge,
}
pub fn resolve(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, CanonicalSchema>,
) -> Result<Rule, String> {
    Ok(Rule {
        field: resolver
            .typed_field(CompanionValueField::reference())
            .ok_or("companion value missing")?,
        edge: resolver
            .typed_relation(CompanionLink::reference())
            .ok_or("companion link missing")?,
    })
}
impl Rule {
    fn prepare(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, CanonicalSchema>,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<Vec<Entity>, WorthQueryApplicationInvariantPreparationError> {
        let view = planner.proposed();
        let entities = view
            .touched_entities(&self.field)
            .map_err(|e| WorthQueryApplicationInvariantPreparationError::new(e.to_string()))?;
        if budget.is_some_and(|b| {
            !b.claim_result((entities.capacity() * std::mem::size_of::<Entity>()) as u64)
        }) {
            return Err(WorthQueryApplicationInvariantPreparationError::new(
                "stopped",
            ));
        }
        for entity in &entities {
            if budget.is_some_and(|b| !b.checkpoint(1)) {
                return Err(WorthQueryApplicationInvariantPreparationError::new(
                    "stopped",
                ));
            }
            for edge in view
                .relations_from(&self.edge, entity)
                .map_err(|e| WorthQueryApplicationInvariantPreparationError::new(e.to_string()))?
            {
                view.field(&self.field, edge.from()).map_err(|e| {
                    WorthQueryApplicationInvariantPreparationError::new(e.to_string())
                })?;
                view.field(&self.field, edge.to()).map_err(|e| {
                    WorthQueryApplicationInvariantPreparationError::new(e.to_string())
                })?;
            }
        }
        Ok(entities)
    }
    fn judge(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, CanonicalSchema>,
        scope: &[Entity],
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<WorthQueryApplicationInvariantVerdict, WorthQueryApplicationInvariantExecutionError>
    {
        let view = context.proposed();
        for entity in scope {
            if budget.is_some_and(|b| !b.checkpoint(1)) {
                return Err(WorthQueryApplicationInvariantExecutionError::new("stopped"));
            }
            for edge in view
                .relations_from(&self.edge, entity)
                .map_err(|e| WorthQueryApplicationInvariantExecutionError::new(e.to_string()))?
            {
                let from = view.field(&self.field, edge.from()).map_err(|e| {
                    WorthQueryApplicationInvariantExecutionError::new(e.to_string())
                })?;
                let to = view.field(&self.field, edge.to()).map_err(|e| {
                    WorthQueryApplicationInvariantExecutionError::new(e.to_string())
                })?;
                if from.as_deref() != Some("companion-left")
                    || to.as_deref() != Some("companion-right")
                {
                    return Ok(WorthQueryApplicationInvariantVerdict::Violation);
                }
            }
        }
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
}
impl WorthQueryApplicationInvariantRule<CanonicalSchema> for Rule {
    type Scope = Vec<Entity>;
    fn supports_checked_preparation(&self) -> bool {
        true
    }
    fn prepare_scope_checked(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, CanonicalSchema>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, WorthQueryApplicationInvariantPreparationError>> {
        Some(self.prepare(planner, Some(budget)))
    }
    fn evaluate_checked(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, CanonicalSchema>,
        scope: &Self::Scope,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<
        Result<WorthQueryApplicationInvariantVerdict, WorthQueryApplicationInvariantExecutionError>,
    > {
        Some(self.judge(context, scope, Some(budget)))
    }
    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, CanonicalSchema>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError> {
        self.prepare(planner, None)
    }
    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, CanonicalSchema>,
        scope: &Self::Scope,
    ) -> Result<WorthQueryApplicationInvariantVerdict, WorthQueryApplicationInvariantExecutionError>
    {
        self.judge(context, scope, None)
    }
}
