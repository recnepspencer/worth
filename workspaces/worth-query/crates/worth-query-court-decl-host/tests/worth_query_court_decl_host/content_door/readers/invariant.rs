//! A touched-record rule exercises proposed content independently of Source scope.
use super::super::*;
use std::sync::{atomic::Ordering, Arc};
use worth_query_host::facade::application_invariants::*;
pub struct CheckContent;
impl ApplicationInvariantMarkerIdentity<Schema> for CheckContent {
    const IDENTIFIER: &'static str = "ObserveReaderContent";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}
pub fn definition() -> ApplicationInvariantDefinition<Schema, CheckContent> {
    let closure = [ApplicationInvariantScopeTarget::Entity("Record".into())];
    ApplicationInvariantDefinition::new(
        CheckContent::reference(),
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
}
pub struct Rule {
    entity: WorthQueryApplicationInvariantEntityBinding<Schema, Record>,
    calls: Arc<source::Calls>,
}
pub fn resolve(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, Schema>,
    calls: Arc<source::Calls>,
) -> Result<Rule, String> {
    Ok(Rule {
        entity: resolver
            .typed_entity(Record::reference())
            .ok_or("missing content record")?,
        calls,
    })
}
impl WorthQueryApplicationInvariantRule<Schema> for Rule {
    type Scope = Vec<WorthQueryApplicationInvariantEntity<Schema, Record>>;
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
            .proposed()
            .touched_entities_of(&self.entity)
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
            let view = context.proposed();
            for entity in scope {
                if self.calls.reader_impostor.load(Ordering::SeqCst) != 0 {
                    assert!(matches!(
                        view.content(entity, content_field(Values::at("Values"), codec::Impostor)),
                        Err(ContentReadDenial::BindingContractMismatch)
                    ));
                    return Ok(WorthQueryApplicationInvariantVerdict::Pass);
                }
                if let Some(content) = view.content(entity, samples())? {
                    if !budget.claim_result(content.payload().len() as u64) {
                        return Err(WorthQueryApplicationInvariantError::Stopped);
                    }
                    self.calls
                        .reader_bytes
                        .lock()
                        .unwrap()
                        .push(content.payload().to_vec());
                }
            }
            Ok(WorthQueryApplicationInvariantVerdict::Pass)
        })())
    }
}
