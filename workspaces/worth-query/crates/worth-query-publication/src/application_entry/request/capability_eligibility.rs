//! Fact-free "who may exercise this capability?" on an ordinary or retained
//! request. Verdicts come from admission's own capability observation; no
//! decision is retained, no grant reserved and nothing committed.

use super::{WorthQueryApplicationRequest, WorthQueryApplicationRetainedRequest};
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_declaration::facade::application_schema::{
    ApplicationBindingContext, ApplicationOperationMarkerIdentity,
    ApplicationStructuredValueBinding,
};
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase;
use worth_query_execution::facade::eligibility::{
    WorthQueryAbilityEligibilityDenial, WorthQueryAbilityKey, WorthQueryCapabilityEligibility,
    WorthQueryCapabilityEligibilityQuestion, WorthQueryEligibilityBasis,
};
use worth_query_installation::facade::ApplicationSchema;

type Question<Schema, Context, Capability, Operation, Input> =
    WorthQueryCapabilityEligibilityQuestion<Schema, Context, Capability, Operation, Input>;
type Verdict = Result<WorthQueryCapabilityEligibility, WorthQueryAbilityEligibilityDenial>;
type Verdicts = Result<Vec<WorthQueryCapabilityEligibility>, WorthQueryAbilityEligibilityDenial>;

impl<Schema: ApplicationSchema> WorthQueryApplicationRequest<'_, '_, '_, Schema> {
    /// The request principal's own capability verdict at the branch head.
    pub fn observe_capability_eligibility<Context, Capability, Operation, Input>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        question: &Question<Schema, Context, Capability, Operation, Input>,
    ) -> Verdict
    where
        Context: ApplicationBindingContext<Schema>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability> + 'static,
    {
        let basis = WorthQueryEligibilityBasis::Current(self.branch);
        self.application.observe_asker_capability_eligibility(
            phase,
            basis,
            self.principal,
            self.scope,
            question,
        )
    }

    /// Per-candidate capability verdicts at the branch head, disclosed only
    /// when the request principal holds `disclosure` on the request's scope.
    pub fn observe_roster_capability_eligibility<Context, Capability, Operation, Input>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        question: &Question<Schema, Context, Capability, Operation, Input>,
        disclosure: WorthQueryAbilityKey,
        candidates: &[WorthQueryExternalPrincipalIdentity],
    ) -> Verdicts
    where
        Context: ApplicationBindingContext<Schema>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability> + 'static,
    {
        let basis = WorthQueryEligibilityBasis::Current(self.branch);
        self.application.observe_roster_capability_eligibility(
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
    /// The request principal's own capability verdict as of the pinned
    /// commit; the principal is still admitted at the branch head.
    pub fn observe_capability_eligibility<Context, Capability, Operation, Input>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        question: &Question<Schema, Context, Capability, Operation, Input>,
    ) -> Verdict
    where
        Context: ApplicationBindingContext<Schema>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability> + 'static,
    {
        let basis = WorthQueryEligibilityBasis::Retained(&self.observation);
        self.application.observe_asker_capability_eligibility(
            phase,
            basis,
            self.principal,
            self.scope,
            question,
        )
    }

    /// Per-candidate capability verdicts as of the pinned commit, disclosed
    /// only when the request principal holds `disclosure` at the branch head.
    pub fn observe_roster_capability_eligibility<Context, Capability, Operation, Input>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        question: &Question<Schema, Context, Capability, Operation, Input>,
        disclosure: WorthQueryAbilityKey,
        candidates: &[WorthQueryExternalPrincipalIdentity],
    ) -> Verdicts
    where
        Context: ApplicationBindingContext<Schema>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability> + 'static,
    {
        let basis = WorthQueryEligibilityBasis::Retained(&self.observation);
        self.application.observe_roster_capability_eligibility(
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
