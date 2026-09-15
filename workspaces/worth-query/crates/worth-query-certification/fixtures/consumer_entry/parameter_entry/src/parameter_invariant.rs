use std::num::NonZeroU64;

use worth_query_consumer_values::PositiveCount;
use worth_query_decl::facade::application_schema::{
    ApplicationInvariantCostPosture, ApplicationInvariantDefinition,
    ApplicationInvariantEnforcement, ApplicationInvariantExecutionPoint, ApplicationInvariantGroup,
    ApplicationInvariantMarkerIdentity, ApplicationInvariantOperationalContract,
    ApplicationInvariantScopeTarget, RelationalInvariantWorkBudget,
};
use worth_query_host::facade::application_invariants::{
    WorthQueryApplicationInvariantContext, WorthQueryApplicationInvariantEntity,
    WorthQueryApplicationInvariantExecutionError, WorthQueryApplicationInvariantFieldBinding,
    WorthQueryApplicationInvariantPreparationError, WorthQueryApplicationInvariantRule,
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
        RelationalInvariantWorkBudget::new(NonZeroU64::new(1_024).unwrap()),
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
            .typed_field(Count::reference::<Schema>())
            .ok_or("missing parameter count")?,
    })
}

pub struct PositiveParameterCountRule<Schema: ParameterSchemaBinding> {
    count: WorthQueryApplicationInvariantFieldBinding<Schema, Parameter, PositiveCount>,
}

impl<Schema: ParameterSchemaBinding> WorthQueryApplicationInvariantRule<Schema>
    for PositiveParameterCountRule<Schema>
{
    type Scope = Vec<WorthQueryApplicationInvariantEntity<Schema, Parameter>>;

    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Schema>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError> {
        planner
            .proposed()
            .touched_entities(&self.count)
            .map_err(|error| WorthQueryApplicationInvariantPreparationError::new(error.to_string()))
    }

    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Schema>,
        scope: &Self::Scope,
    ) -> Result<WorthQueryApplicationInvariantVerdict, WorthQueryApplicationInvariantExecutionError>
    {
        for entity in scope {
            let count = context
                .proposed()
                .field(&self.count, entity)
                .map_err(|error| {
                    WorthQueryApplicationInvariantExecutionError::new(error.to_string())
                })?
                .ok_or_else(|| {
                    WorthQueryApplicationInvariantExecutionError::new("parameter count absent")
                })?;
            if PositiveCount::get(&count) == 0 {
                return Ok(WorthQueryApplicationInvariantVerdict::Violation);
            }
        }
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
}
