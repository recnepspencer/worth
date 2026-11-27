//! Fact-free "who may act?" on an ordinary or retained request.
//!
//! The request's principal asks; the verdicts come from the installed policy
//! decision admission uses. No decision fact, grant or commit results.

use super::{WorthQueryApplicationRequest, WorthQueryApplicationRetainedRequest};
use worth_query_declaration::facade::application_schema::fragment::RecordSelection;
use worth_query_declaration::facade::application_schema::ApplicationBindingContext;
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase;
use worth_query_execution::facade::eligibility::{
    WorthQueryAbilityEligibility, WorthQueryAbilityEligibilityDenial, WorthQueryAbilityKey,
    WorthQueryEligibilityBasis, WorthQueryEligibilityQuestion,
};
use worth_query_installation::facade::ApplicationSchema;

type Verdicts = Result<Vec<WorthQueryAbilityEligibility>, WorthQueryAbilityEligibilityDenial>;

impl<Schema: ApplicationSchema> WorthQueryApplicationRequest<'_, '_, '_, Schema> {
    /// The request principal's own verdict at this request's branch head.
    pub fn observe_eligibility<Context, Ability, Field>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        question: &WorthQueryEligibilityQuestion<Schema, Context, Ability, Field>,
    ) -> Result<WorthQueryAbilityEligibility, WorthQueryAbilityEligibilityDenial>
    where
        Context: ApplicationBindingContext<Schema>,
        Field: RecordSelection,
        Field::Value: Clone,
    {
        let basis = WorthQueryEligibilityBasis::Current(self.branch);
        self.application.observe_asker_eligibility(
            phase,
            basis,
            self.principal,
            self.scope,
            question,
        )
    }

    /// Per-candidate verdicts at this request's branch head, disclosed only
    /// when the request principal holds `disclosure` on the same scope.
    pub fn observe_roster_eligibility<Context, Ability, Field>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        question: &WorthQueryEligibilityQuestion<Schema, Context, Ability, Field>,
        disclosure: WorthQueryAbilityKey,
        candidates: &[WorthQueryExternalPrincipalIdentity],
    ) -> Verdicts
    where
        Context: ApplicationBindingContext<Schema>,
        Field: RecordSelection,
        Field::Value: Clone,
    {
        let basis = WorthQueryEligibilityBasis::Current(self.branch);
        self.application.observe_roster_eligibility(
            phase,
            basis,
            self.principal,
            self.scope,
            question,
            disclosure,
            candidates,
        )
    }
}

impl<Schema: ApplicationSchema> WorthQueryApplicationRetainedRequest<'_, '_, '_, Schema> {
    /// The request principal's own verdict as of the pinned commit.
    pub fn observe_eligibility<Context, Ability, Field>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        question: &WorthQueryEligibilityQuestion<Schema, Context, Ability, Field>,
    ) -> Result<WorthQueryAbilityEligibility, WorthQueryAbilityEligibilityDenial>
    where
        Context: ApplicationBindingContext<Schema>,
        Field: RecordSelection,
        Field::Value: Clone,
    {
        let basis = WorthQueryEligibilityBasis::Retained(&self.observation);
        self.application.observe_asker_eligibility(
            phase,
            basis,
            self.principal,
            self.scope,
            question,
        )
    }

    /// Per-candidate verdicts as of the pinned commit, disclosed only when the
    /// request principal holds `disclosure` on the same scope there.
    pub fn observe_roster_eligibility<Context, Ability, Field>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        question: &WorthQueryEligibilityQuestion<Schema, Context, Ability, Field>,
        disclosure: WorthQueryAbilityKey,
        candidates: &[WorthQueryExternalPrincipalIdentity],
    ) -> Verdicts
    where
        Context: ApplicationBindingContext<Schema>,
        Field: RecordSelection,
        Field::Value: Clone,
    {
        let basis = WorthQueryEligibilityBasis::Retained(&self.observation);
        self.application.observe_roster_eligibility(
            phase,
            basis,
            self.principal,
            self.scope,
            question,
            disclosure,
            candidates,
        )
    }
}
