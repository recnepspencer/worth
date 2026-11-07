use std::num::NonZeroU64;

use worth_query_consumer_values::PositiveCount;
use worth_query_decl::facade::application_schema::{
    ApplicationInvariantCostPosture, ApplicationInvariantDefinition,
    ApplicationInvariantEnforcement, ApplicationInvariantExecutionPoint, ApplicationInvariantGroup,
    ApplicationInvariantMarkerIdentity, ApplicationInvariantOperationalContract,
    ApplicationInvariantScopeTarget, RelationalInvariantWorkBudget,
};
use worth_query_host::facade::application_invariants::{
    CustomInvariantLeaseBudget, WorthQueryApplicationInvariantContext,
    WorthQueryApplicationInvariantEntity, WorthQueryApplicationInvariantError,
    WorthQueryApplicationInvariantFieldBinding, WorthQueryApplicationInvariantRule,
    WorthQueryApplicationInvariantSchemaResolver, WorthQueryApplicationInvariantScopePlanner,
    WorthQueryApplicationInvariantVerdict,
};

use super::{Count, Parameter, ParameterSchemaBinding};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PositiveParameterCount;

impl<Schema: ParameterSchemaBinding> ApplicationInvariantMarkerIdentity<Schema>
    for PositiveParameterCount
{
    const IDENTIFIER: &'static str = "PositiveParameterCount";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

pub fn parameter_count_invariant<Schema>(
) -> ApplicationInvariantDefinition<Schema, PositiveParameterCount>
where
    Schema: ParameterSchemaBinding,
{
    ApplicationInvariantDefinition::new(
        PositiveParameterCount::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        RelationalInvariantWorkBudget::new(Some(NonZeroU64::new(1_024).unwrap())),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [ApplicationInvariantGroup::SchemaCompliance],
            [ApplicationInvariantScopeTarget::Entity(
                "Parameter".to_owned(),
            )],
            [ApplicationInvariantScopeTarget::Entity(
                "Parameter".to_owned(),
            )],
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
}

pub(crate) fn resolve_rule<Schema: ParameterSchemaBinding>(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, Schema>,
) -> Result<PositiveParameterCountRule<Schema>, String> {
    Ok(PositiveParameterCountRule {
        count: resolver
            .typed_field(Count::at("value"))
            .ok_or("missing parameter count")?,
    })
}

pub struct PositiveParameterCountRule<Schema: ParameterSchemaBinding> {
    count: WorthQueryApplicationInvariantFieldBinding<Schema, Parameter, PositiveCount>,
}

impl<Schema: ParameterSchemaBinding> WorthQueryApplicationInvariantRule<Schema>
    for PositiveParameterCountRule<Schema>
{
    type Refusal = ();
    type RefusalBinding =
        worth_query_host::facade::application_invariants::WorthQueryEmptyInvariantRefusal;
    type Scope = Vec<WorthQueryApplicationInvariantEntity<Schema, Parameter>>;

    fn supports_checked_preparation(&self) -> bool {
        true
    }
    fn prepare_scope_checked(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Schema>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, WorthQueryApplicationInvariantError>> {
        Some(self.prepare_under(planner, Some(budget)))
    }
    fn evaluate_checked(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Schema>,
        scope: &Self::Scope,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<
        Result<WorthQueryApplicationInvariantVerdict<()>, WorthQueryApplicationInvariantError>,
    > {
        Some(self.evaluate_under(context, scope, Some(budget)))
    }

    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Schema>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantError> {
        self.prepare_under(planner, None)
    }

    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Schema>,
        scope: &Self::Scope,
    ) -> Result<WorthQueryApplicationInvariantVerdict<()>, WorthQueryApplicationInvariantError>
    {
        self.evaluate_under(context, scope, None)
    }
}

impl<Schema: ParameterSchemaBinding> PositiveParameterCountRule<Schema> {
    fn prepare_under(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Schema>,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<
        <Self as WorthQueryApplicationInvariantRule<Schema>>::Scope,
        WorthQueryApplicationInvariantError,
    > {
        if budget.is_some_and(|budget| !budget.checkpoint(0)) {
            return Err(WorthQueryApplicationInvariantError::Stopped);
        }
        planner
            .proposed()
            .touched_entities(&self.count)
            .map_err(WorthQueryApplicationInvariantError::from)
    }
    fn evaluate_under(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Schema>,
        scope: &<Self as WorthQueryApplicationInvariantRule<Schema>>::Scope,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<WorthQueryApplicationInvariantVerdict<()>, WorthQueryApplicationInvariantError>
    {
        if budget.is_some_and(|budget| !budget.checkpoint(0)) {
            return Err(WorthQueryApplicationInvariantError::Stopped);
        }
        for entity in scope {
            if budget.is_some_and(|budget| !budget.checkpoint(1)) {
                return Err(WorthQueryApplicationInvariantError::Stopped);
            }
            let count = context
                .proposed()
                .field(&self.count, entity)
                .map_err(WorthQueryApplicationInvariantError::from)?
                .ok_or_else(
                    || WorthQueryApplicationInvariantError::MissingRequiredField {
                        field: "parameter count absent",
                    },
                )?;
            if PositiveCount::get(&count) == 0 {
                return Ok(WorthQueryApplicationInvariantVerdict::Violation(()));
            }
        }
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
}

#[cfg(test)]
mod lease_tests;
