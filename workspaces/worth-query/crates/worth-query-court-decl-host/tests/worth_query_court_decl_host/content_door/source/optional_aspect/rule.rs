//! The real installed Source audience requires the proposed index during commit.
use super::*;
pub struct CheckOptionalContent;
impl ApplicationInvariantMarkerIdentity<Schema> for CheckOptionalContent {
    const IDENTIFIER: &'static str = "ObserveOptionalContentSource";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}
pub fn definition() -> ApplicationInvariantDefinition<Schema, CheckOptionalContent> {
    let closure = [
        ApplicationInvariantScopeTarget::Entity("Record".into()),
        ApplicationInvariantScopeTarget::Relation("ContainsOptionalSamples".into()),
    ];
    ApplicationInvariantDefinition::new(
        CheckOptionalContent::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        RelationalInvariantWorkBudget::new(None),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [ApplicationInvariantGroup::SchemaCompliance],
            closure.clone(),
            closure,
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
    .managed_index_source(Index::reference())
    .managed_index_cold_sources(ApplicationManagedIndexColdSource::AdmitRebuild)
}
pub struct Rule {
    index: WorthQueryApplicationInvariantManagedIndexBinding<Schema, Index>,
}
pub fn resolve(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, Schema>,
) -> Result<Rule, String> {
    Ok(Rule {
        index: resolver
            .typed_managed_index(Index::reference())
            .ok_or("missing optional content Source")?,
    })
}
impl WorthQueryApplicationInvariantRule<Schema> for Rule {
    type Scope = WorthQueryPreparedInvariantManagedIndex<Schema, Index>;
    type Refusal = ();
    type RefusalBinding = WorthQueryEmptyInvariantRefusal;
    fn supports_checked_preparation(&self) -> bool {
        true
    }
    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Schema>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantError> {
        planner
            .retain_managed_index(&self.index)
            .map_err(Into::into)
    }
    fn prepare_scope_checked(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Schema>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, WorthQueryApplicationInvariantError>> {
        Some(if budget.checkpoint(1) {
            self.prepare_scope(planner)
        } else {
            Err(WorthQueryApplicationInvariantError::Stopped)
        })
    }
    fn evaluate(
        &self,
        _: &WorthQueryApplicationInvariantContext<'_, '_, Schema>,
        _: &Self::Scope,
    ) -> Result<WorthQueryApplicationInvariantVerdict<()>, WorthQueryApplicationInvariantError>
    {
        Err(WorthQueryApplicationInvariantError::CheckedHookRequired)
    }
    fn evaluate_checked(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Schema>,
        scope: &Self::Scope,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<
        Result<WorthQueryApplicationInvariantVerdict<()>, WorthQueryApplicationInvariantError>,
    > {
        Some((|| {
            if !budget.checkpoint(1) {
                return Err(WorthQueryApplicationInvariantError::Stopped);
            }
            context.managed_index(scope)?.proposed().query(&0u64)?;
            Ok(WorthQueryApplicationInvariantVerdict::Pass)
        })())
    }
}
