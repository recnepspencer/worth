//! A rule governing only the ledger, which no account publication touches.
use super::rule::{Refusal, RefusalBinding};
use super::schema::RefusalSchema;
use crate::facade::application_invariants::*;
use worth_query_declaration::facade::application_schema::*;

pub(super) struct LedgerOnly;
impl ApplicationInvariantMarkerIdentity<RefusalSchema> for LedgerOnly {
    const IDENTIFIER: &'static str = "LedgerOnly";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

pub(super) fn definition() -> ApplicationInvariantDefinition<RefusalSchema, LedgerOnly> {
    ApplicationInvariantDefinition::new(
        LedgerOnly::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        RelationalInvariantWorkBudget::new(Some(std::num::NonZeroU64::new(1024).unwrap())),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [ApplicationInvariantGroup::SchemaCompliance],
            [ApplicationInvariantScopeTarget::Entity("Ledger".into())],
            [ApplicationInvariantScopeTarget::Entity("Ledger".into())],
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
}

/// Holds whenever it runs; the court is about whether it is asked to.
pub(super) struct LedgerRule;
impl WorthQueryApplicationInvariantRule<RefusalSchema> for LedgerRule {
    type Scope = ();
    type Refusal = Refusal;
    type RefusalBinding = RefusalBinding;
    fn supports_checked_preparation(&self) -> bool {
        true
    }
    fn prepare_scope_checked(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, RefusalSchema>,
        _: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<(), WorthQueryApplicationInvariantPreparationError>> {
        Some(self.prepare_scope(planner))
    }
    fn prepare_scope(
        &self,
        _: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, RefusalSchema>,
    ) -> Result<(), WorthQueryApplicationInvariantPreparationError> {
        Ok(())
    }
    fn evaluate(
        &self,
        _: &WorthQueryApplicationInvariantContext<'_, '_, RefusalSchema>,
        _: &(),
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<Refusal>,
        WorthQueryApplicationInvariantExecutionError,
    > {
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
}
