use super::schema::{Account, AccountOwner, AccountStatus, Principal, RefusalSchema};
use crate::facade::application_invariants::*;
use std::sync::{Arc, Mutex};
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_declaration::facade::application_schema::*;

#[derive(Clone, Debug, Eq, PartialEq, worth_foundational::facade::WorthValue)]
pub(super) struct Refusal {
    pub(super) rule: u64,
    pub(super) observed: String,
}
worth_query_declaration::worth_query_value_binding!(pub(super) RefusalBinding for Refusal {
    identity: "tests.account-refusal.v1", scalar: String,
    canonical: ValueDecodeBounds::new(128, 128, 4, 4),
});
worth_query_declaration::worth_query_value_binding!(pub(super) OtherBinding for Refusal {
    identity: "tests.other-refusal.v1", scalar: String,
    canonical: ValueDecodeBounds::new(128, 128, 4, 4),
});

pub(super) struct First;
pub(super) struct Second;
impl ApplicationInvariantMarkerIdentity<RefusalSchema> for First {
    const IDENTIFIER: &'static str = "ZFirstInstalled";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}
impl ApplicationInvariantMarkerIdentity<RefusalSchema> for Second {
    const IDENTIFIER: &'static str = "ASecondInstalled";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}
pub(super) fn definition<I: ApplicationInvariantMarkerIdentity<RefusalSchema>>(
) -> ApplicationInvariantDefinition<RefusalSchema, I> {
    ApplicationInvariantDefinition::new(
        I::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        RelationalInvariantWorkBudget::new(Some(std::num::NonZeroU64::new(1024).unwrap())),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [if I::IDENTIFIER == First::IDENTIFIER {
                ApplicationInvariantGroup::SchemaCompliance
            } else {
                ApplicationInvariantGroup::IdentityCoherence
            }],
            [
                ApplicationInvariantScopeTarget::Entity("Account".into()),
                ApplicationInvariantScopeTarget::Entity("Principal".into()),
                ApplicationInvariantScopeTarget::Relation("AccountOwner".into()),
            ],
            [
                ApplicationInvariantScopeTarget::Entity("Account".into()),
                ApplicationInvariantScopeTarget::Entity("Principal".into()),
                ApplicationInvariantScopeTarget::Relation("AccountOwner".into()),
            ],
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
}

pub(super) struct Rule {
    pub(super) ordinal: u64,
    /// Cancels the request on finding a violation, instead of refusing.
    pub(super) cancellation: Option<WorthQueryCancellationSource>,
    pub(super) visits: Arc<Mutex<Vec<u64>>>,
    pub(super) reads: Arc<Mutex<Vec<ReadSample>>>,
}
#[derive(Clone, Copy, Debug)]
pub(super) enum ReadStage {
    Prepared,
    EvaluationStart,
    FirstView,
    SecondView,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct ReadSample {
    pub(super) stage: ReadStage,
    pub(super) reads: WorthQueryInvariantProjectionWork,
}

pub(super) struct InstalledRule {
    rule: Rule,
    status: WorthQueryApplicationInvariantFieldBinding<RefusalSchema, Account, String>,
    owner: WorthQueryApplicationInvariantRelationBinding<
        RefusalSchema,
        AccountOwner,
        Principal,
        Account,
    >,
}
impl Rule {
    pub(super) fn resolve(
        self,
        resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, RefusalSchema>,
    ) -> Result<InstalledRule, String> {
        Ok(InstalledRule {
            rule: self,
            owner: resolver
                .typed_relation(AccountOwner::reference())
                .ok_or("owner missing")?,
            status: resolver
                .typed_field(AccountStatus::reference())
                .ok_or("status missing")?,
        })
    }
}
impl WorthQueryApplicationInvariantRule<RefusalSchema> for InstalledRule {
    type Scope = Vec<WorthQueryApplicationInvariantEntity<RefusalSchema, Account>>;
    type Refusal = Refusal;
    type RefusalBinding = RefusalBinding;
    fn supports_checked_preparation(&self) -> bool {
        true
    }
    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, RefusalSchema>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError> {
        let proposed = planner.proposed();
        let touched = proposed
            .touched_entities(&self.status)
            .map_err(|e| WorthQueryApplicationInvariantPreparationError::new(e.to_string()))?;
        for entity in &touched {
            proposed
                .field(&self.status, entity)
                .map_err(|e| WorthQueryApplicationInvariantPreparationError::new(e.to_string()))?;
        }
        for entity in &touched {
            proposed
                .relations_to(&self.owner, entity)
                .map_err(|e| WorthQueryApplicationInvariantPreparationError::new(e.to_string()))?;
        }
        self.record_reads(ReadStage::Prepared, planner.reads());
        Ok(touched)
    }
    fn prepare_scope_checked(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, RefusalSchema>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, WorthQueryApplicationInvariantPreparationError>> {
        if !budget.checkpoint(1) {
            return Some(Err(WorthQueryApplicationInvariantPreparationError::new(
                "stopped",
            )));
        }
        Some(self.prepare_scope(planner))
    }
    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, RefusalSchema>,
        scope: &Self::Scope,
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<Refusal>,
        WorthQueryApplicationInvariantExecutionError,
    > {
        self.evaluate_under(context, scope, None)
    }
    fn evaluate_checked(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, RefusalSchema>,
        scope: &Self::Scope,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<
        Result<
            WorthQueryApplicationInvariantVerdict<Refusal>,
            WorthQueryApplicationInvariantExecutionError,
        >,
    > {
        Some(self.evaluate_under(context, scope, Some(budget)))
    }
}
impl InstalledRule {
    fn evaluate_under(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, RefusalSchema>,
        scope: &<Self as WorthQueryApplicationInvariantRule<RefusalSchema>>::Scope,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<
        WorthQueryApplicationInvariantVerdict<Refusal>,
        WorthQueryApplicationInvariantExecutionError,
    > {
        self.record_reads(ReadStage::EvaluationStart, context.reads());
        for entity in scope {
            if budget.is_some_and(|b| !b.checkpoint(1)) {
                return Err(WorthQueryApplicationInvariantExecutionError::new("stopped"));
            }
            self.read_status(context, entity)?;
            self.record_reads(ReadStage::FirstView, context.reads());
            let status = self.read_status(context, entity)?;
            self.record_reads(ReadStage::SecondView, context.reads());
            if status != "open" {
                self.rule.visits.lock().unwrap().push(self.rule.ordinal);
                if let Some(source) = &self.rule.cancellation {
                    source.cancel();
                    return Err(WorthQueryApplicationInvariantExecutionError::new(
                        "evaluation cancelled",
                    ));
                }
                return Ok(WorthQueryApplicationInvariantVerdict::Violation(Refusal {
                    rule: self.rule.ordinal,
                    observed: status,
                }));
            }
        }
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
}

impl InstalledRule {
    fn read_status(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, RefusalSchema>,
        entity: &WorthQueryApplicationInvariantEntity<RefusalSchema, Account>,
    ) -> Result<String, WorthQueryApplicationInvariantExecutionError> {
        let proposed = context.proposed();
        proposed
            .relations_to(&self.owner, entity)
            .map_err(|e| WorthQueryApplicationInvariantExecutionError::new(e.to_string()))?;
        proposed
            .field(&self.status, entity)
            .map_err(|e| WorthQueryApplicationInvariantExecutionError::new(e.to_string()))?
            .ok_or_else(|| WorthQueryApplicationInvariantExecutionError::new("status absent"))
    }

    fn record_reads(&self, stage: ReadStage, reads: WorthQueryInvariantProjectionWork) {
        self.rule
            .reads
            .lock()
            .unwrap()
            .push(ReadSample { stage, reads });
    }
}
