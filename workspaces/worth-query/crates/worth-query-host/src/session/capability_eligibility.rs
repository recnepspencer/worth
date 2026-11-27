//! "Who may exercise this capability?" from the installed capability
//! contract itself: one source of truth for admission and eligibility.
//!
//! A verdict runs capability admission's own observation for the principal
//! on the question's request: grant selection, currentness, validity at one
//! trusted instant, composition rules and delegation lineage. A disabled
//! role assignment, an expired or revoked grant and an inactive extent refuse
//! eligibility exactly as they refuse admission. Asking retains no decision,
//! reserves no grant and commits nothing; acting still admits the actor.
//!
//! Head only, by contract: a verdict judges grant validity at the trusted
//! instant of the call, so a snapshot pinned at an older commit would judge
//! that commit's grants against today's clock. A
//! [`super::WorthQuerySessionSnapshot`] therefore offers ability eligibility
//! (relationship policy, no clock) but not capability eligibility.
//!
//! ```text
//! let review = WorthQueryCapabilityEligibilityQuestion::of_request(
//!     Desktop, ReviewCapability::reference(), ReviewOp::reference(), &request,
//! )?;
//! if session.eligible_for_capability(&review)?.is_eligible() { /* show it */ }
//! let queue = session.eligible_among_for_capability(&review, ViewQueue::reference(), &staff)?;
//! ```
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_declaration::facade::application_schema::{
    ApplicationAbilityRef, ApplicationBindingContext, ApplicationOperationMarkerIdentity,
    ApplicationSchema, ApplicationStructuredValueBinding,
};
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_execution::facade::eligibility::{
    WorthQueryAbilityKey, WorthQueryCapabilityEligibility, WorthQueryCapabilityEligibilityQuestion,
};

use super::eligibility::{WorthQueryEligibilityDenial, WorthQueryEligibilityRoster};
use super::WorthQueryHostSession;

type Question<Schema, Context, Capability, Operation, Input> =
    WorthQueryCapabilityEligibilityQuestion<Schema, Context, Capability, Operation, Input>;
type Scope<Schema, Capability, Input> =
    <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope;

impl<Schema: ApplicationSchema, Program> WorthQueryHostSession<'_, Schema, Program> {
    /// Whether this session's principal may exercise the question's
    /// capability now, at the branch head an action admitted now would read.
    pub fn eligible_for_capability<Context, Capability, Operation, Input>(
        &self,
        question: &Question<Schema, Context, Capability, Operation, Input>,
    ) -> Result<WorthQueryCapabilityEligibility, WorthQueryEligibilityDenial>
    where
        Context: ApplicationBindingContext<Schema>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability> + 'static,
    {
        observe_capability_asker(self, question)
    }

    /// Which `candidates` may exercise the question's capability now, behind
    /// the ability roster's disclosure gate: this session's principal must
    /// hold `disclosure` on the request's scope, or the call refuses
    /// `RosterNotDisclosed` before evaluating any candidate.
    pub fn eligible_among_for_capability<Context, Capability, Operation, Input, Disclosure>(
        &self,
        question: &Question<Schema, Context, Capability, Operation, Input>,
        disclosure: ApplicationAbilityRef<Schema, Disclosure, Scope<Schema, Capability, Input>>,
        candidates: &[WorthQueryExternalPrincipalIdentity],
    ) -> Result<
        WorthQueryEligibilityRoster<WorthQueryCapabilityEligibility>,
        WorthQueryEligibilityDenial,
    >
    where
        Context: ApplicationBindingContext<Schema>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability> + 'static,
    {
        let disclosure = WorthQueryAbilityKey::of(disclosure);
        observe_capability_roster(self, question, disclosure, candidates)
    }
}

/// The asker's own capability verdict at the head, under a fresh request
/// scope and the session's current proof.
fn observe_capability_asker<Schema, Program, Context, Capability, Operation, Input>(
    session: &WorthQueryHostSession<'_, Schema, Program>,
    question: &Question<Schema, Context, Capability, Operation, Input>,
) -> Result<WorthQueryCapabilityEligibility, WorthQueryEligibilityDenial>
where
    Schema: ApplicationSchema,
    Context: ApplicationBindingContext<Schema>,
    Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
    Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
    Input: ApplicationCapabilityRequest<Schema, Capability> + 'static,
{
    let scope = session.fresh_scope();
    let principal = session.call_principal();
    let request = session.request_as(&principal, &scope);
    session
        .with_funded_advancement(&scope, |phase| {
            request.observe_capability_eligibility(&phase, question)
        })
        .map_err(WorthQueryEligibilityDenial::ExecutionRequest)?
        .map_err(WorthQueryEligibilityDenial::Observation)
}

/// Candidate capability verdicts at the head.
fn observe_capability_roster<Schema, Program, Context, Capability, Operation, Input>(
    session: &WorthQueryHostSession<'_, Schema, Program>,
    question: &Question<Schema, Context, Capability, Operation, Input>,
    disclosure: WorthQueryAbilityKey,
    candidates: &[WorthQueryExternalPrincipalIdentity],
) -> Result<WorthQueryEligibilityRoster<WorthQueryCapabilityEligibility>, WorthQueryEligibilityDenial>
where
    Schema: ApplicationSchema,
    Context: ApplicationBindingContext<Schema>,
    Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
    Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
    Input: ApplicationCapabilityRequest<Schema, Capability> + 'static,
{
    let scope = session.fresh_scope();
    let principal = session.call_principal();
    let request = session.request_as(&principal, &scope);
    let verdicts = session
        .with_funded_advancement(&scope, |phase| {
            request.observe_roster_capability_eligibility(&phase, question, disclosure, candidates)
        })
        .map_err(WorthQueryEligibilityDenial::ExecutionRequest)?
        .map_err(WorthQueryEligibilityDenial::Observation)?;
    Ok(WorthQueryEligibilityRoster::of(candidates, verdicts))
}
