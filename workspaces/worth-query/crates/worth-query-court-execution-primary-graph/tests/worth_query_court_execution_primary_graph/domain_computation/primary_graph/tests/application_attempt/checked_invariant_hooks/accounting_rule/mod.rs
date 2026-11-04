//! An authored application rule's bounded work, exercised through native hooks.
use crate::domain_computation::primary_graph::tests::application_attempt::checked_invariant_hooks::*;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_declaration::facade::application_schema::*;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantContext;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantRule;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantScopePlanner;
use worth_query_execution::facade::application_invariants::WorthQueryApplicationInvariantVerdict;
use worth_relational::facade::runtime::CustomInvariantExecutionError as WorthQueryApplicationInvariantExecutionError;
use worth_relational::facade::runtime::CustomInvariantLeaseBudget;
use worth_relational::facade::runtime::CustomInvariantPreparationError as WorthQueryApplicationInvariantPreparationError;

pub const PREPARATION_UNITS: u64 = 3;
pub const EVALUATION_UNITS: u64 = 5;

pub struct HookAccounting;
impl ApplicationInvariantMarkerIdentity<HookSchema> for HookAccounting {
    const IDENTIFIER: &'static str = "HookAccounting";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

pub fn definition() -> ApplicationInvariantDefinition<HookSchema, HookAccounting> {
    ApplicationInvariantDefinition::new(
        HookAccounting::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        RelationalInvariantWorkBudget::new(Some(std::num::NonZeroU64::new(1024).unwrap())),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [ApplicationInvariantGroup::SchemaCompliance],
            [ApplicationInvariantScopeTarget::Entity(
                "Account".to_owned(),
            )],
            [ApplicationInvariantScopeTarget::Entity(
                "Account".to_owned(),
            )],
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
}

#[derive(Clone, Default)]
pub struct Contacts {
    pub armed: Arc<AtomicBool>,
    pub ordinary: Arc<AtomicUsize>,
    pub preparation: Arc<AtomicUsize>,
    pub evaluation: Arc<AtomicUsize>,
    pub visits: Arc<AtomicUsize>,
}
impl Contacts {
    pub fn snapshot(&self) -> [usize; 4] {
        [
            &self.ordinary,
            &self.preparation,
            &self.evaluation,
            &self.visits,
        ]
        .map(|counter| counter.load(Ordering::SeqCst))
    }
}

pub struct AccountingRule {
    pub checked: bool,
    pub work: bool,
    pub cancellation: Option<WorthQueryCancellationSource>,
    pub contacts: Contacts,
}
impl WorthQueryApplicationInvariantRule<HookSchema> for AccountingRule {
    type Refusal = ();
    type RefusalBinding =
        worth_query_execution::facade::application_invariants::WorthQueryEmptyInvariantRefusal;
    type Scope = ();
    fn prepare_scope(
        &self,
        _: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, HookSchema>,
    ) -> Result<(), WorthQueryApplicationInvariantPreparationError> {
        self.contacts.ordinary.fetch_add(1, Ordering::SeqCst);
        for _ in 0..if self.work { PREPARATION_UNITS } else { 0 } {
            self.contacts.visits.fetch_add(1, Ordering::SeqCst);
        }
        Ok(())
    }
    fn evaluate(
        &self,
        _: &WorthQueryApplicationInvariantContext<'_, '_, HookSchema>,
        _: &(),
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<()>,
        WorthQueryApplicationInvariantExecutionError,
    > {
        self.contacts.ordinary.fetch_add(1, Ordering::SeqCst);
        for _ in 0..if self.work { EVALUATION_UNITS } else { 0 } {
            self.contacts.visits.fetch_add(1, Ordering::SeqCst);
        }
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
    fn supports_checked_preparation(&self) -> bool {
        self.checked
    }
    fn prepare_scope_checked(
        &self,
        _: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, HookSchema>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<(), WorthQueryApplicationInvariantPreparationError>> {
        self.contacts.preparation.fetch_add(1, Ordering::SeqCst);
        if !budget.checkpoint(0) {
            return Some(Err(WorthQueryApplicationInvariantPreparationError::new(
                "caller stopped",
            )));
        }
        if self.contacts.armed.load(Ordering::SeqCst)
            && let Some(source) = &self.cancellation
        {
            source.cancel();
        }
        if !budget.checkpoint(0) {
            return Some(Err(WorthQueryApplicationInvariantPreparationError::new(
                "caller stopped",
            )));
        }
        for _ in 0..if self.work { PREPARATION_UNITS } else { 0 } {
            if !budget.checkpoint(1) {
                return Some(Err(WorthQueryApplicationInvariantPreparationError::new(
                    "caller stopped",
                )));
            }
            self.contacts.visits.fetch_add(1, Ordering::SeqCst);
        }
        Some(Ok(()))
    }
    fn evaluate_checked(
        &self,
        _: &WorthQueryApplicationInvariantContext<'_, '_, HookSchema>,
        _: &(),
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<
        Result<
            WorthQueryApplicationInvariantVerdict<()>,
            WorthQueryApplicationInvariantExecutionError,
        >,
    > {
        self.contacts.evaluation.fetch_add(1, Ordering::SeqCst);
        if !budget.checkpoint(0) {
            return Some(Err(WorthQueryApplicationInvariantExecutionError::new(
                "caller stopped",
            )));
        }
        for _ in 0..if self.work { EVALUATION_UNITS } else { 0 } {
            if !budget.checkpoint(1) {
                return Some(Err(WorthQueryApplicationInvariantExecutionError::new(
                    "caller stopped",
                )));
            }
            self.contacts.visits.fetch_add(1, Ordering::SeqCst);
        }
        Some(Ok(WorthQueryApplicationInvariantVerdict::Pass))
    }
}
