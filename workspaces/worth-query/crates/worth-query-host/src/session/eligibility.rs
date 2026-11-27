//! "Who may act?" without acting: fact-free eligibility observation.
//!
//! A verdict is the installed policy decision admission would reach for that
//! principal, read from the same evaluator. Asking mints no decision fact,
//! reserves no grant, commits nothing and advances no sequence. It is not
//! authority either: acting still admits the actor freshly.
//!
//! ```text
//! let approve = WorthQueryEligibilityQuestion::new(
//!     Desktop, Approve::reference(), DocumentFacts::document_id(), doc_id,
//! );
//! if session.eligibility(&approve)?.is_eligible() { /* show the button */ }
//! let roster = session.eligible_among(&approve, ViewApprovers::reference(), &candidates)?;
//! for approver in roster.eligible() { /* list them */ }
//! ```
use worth_query_declaration::facade::application_schema::fragment::RecordSelection;
use worth_query_declaration::facade::application_schema::{
    ApplicationAbilityRef, ApplicationBindingContext, ApplicationSchema,
};
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial;
use worth_query_execution::facade::eligibility::{
    WorthQueryAbilityEligibility, WorthQueryAbilityEligibilityDenial, WorthQueryAbilityKey,
    WorthQueryEligibilityQuestion,
};
use worth_query_publication::facade::application_entry::WorthQueryApplicationReadObservation;

use super::WorthQueryHostSession;

/// Per-candidate verdicts, in the order the candidates were given.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryEligibilityRoster {
    members: Vec<(
        WorthQueryExternalPrincipalIdentity,
        WorthQueryAbilityEligibility,
    )>,
}

impl WorthQueryEligibilityRoster {
    /// Every candidate with its verdict.
    pub fn iter(
        &self,
    ) -> impl Iterator<
        Item = (
            &WorthQueryExternalPrincipalIdentity,
            WorthQueryAbilityEligibility,
        ),
    > {
        self.members
            .iter()
            .map(|(identity, verdict)| (identity, *verdict))
    }

    /// The candidates the installed policy allows, in candidate order.
    pub fn eligible(&self) -> impl Iterator<Item = &WorthQueryExternalPrincipalIdentity> {
        self.iter()
            .filter(|(_, verdict)| verdict.is_eligible())
            .map(|(identity, _)| identity)
    }

    /// The verdict for `identity`, when it was a candidate.
    pub fn verdict(
        &self,
        identity: &WorthQueryExternalPrincipalIdentity,
    ) -> Option<WorthQueryAbilityEligibility> {
        self.iter()
            .find(|(candidate, _)| *candidate == identity)
            .map(|(_, verdict)| verdict)
    }

    pub fn len(&self) -> usize {
        self.members.len()
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
}

/// Why an eligibility question was not answered. Nothing was minted or written.
#[derive(Debug)]
pub enum WorthQueryEligibilityDenial {
    /// The session's execution policy refused to fund the observation.
    ExecutionRequest(WorthQueryAdvancementDenial),
    /// The observation itself refused: see the typed cause.
    Observation(WorthQueryAbilityEligibilityDenial),
}

impl<Schema: ApplicationSchema, Program> WorthQueryHostSession<'_, Schema, Program> {
    /// Whether this session's principal may exercise the question's ability
    /// now, at the branch head an action admitted now would read.
    pub fn eligibility<Context, Ability, Field>(
        &self,
        question: &WorthQueryEligibilityQuestion<Schema, Context, Ability, Field>,
    ) -> Result<WorthQueryAbilityEligibility, WorthQueryEligibilityDenial>
    where
        Context: ApplicationBindingContext<Schema>,
        Field: RecordSelection,
        Field::Value: Clone,
    {
        observe_asker(self, None, question)
    }

    /// Which `candidates` may exercise the question's ability now. Revealing
    /// another principal's authority is disclosure: this session's principal
    /// must itself hold `disclosure` on the same scope, or the call refuses
    /// `RosterNotDisclosed` before evaluating any candidate.
    pub fn eligible_among<Context, Ability, Disclosure, Field>(
        &self,
        question: &WorthQueryEligibilityQuestion<Schema, Context, Ability, Field>,
        disclosure: ApplicationAbilityRef<Schema, Disclosure, Field::Entity>,
        candidates: &[WorthQueryExternalPrincipalIdentity],
    ) -> Result<WorthQueryEligibilityRoster, WorthQueryEligibilityDenial>
    where
        Context: ApplicationBindingContext<Schema>,
        Field: RecordSelection,
        Field::Value: Clone,
    {
        let disclosure = WorthQueryAbilityKey::of(disclosure);
        observe_roster(self, None, question, disclosure, candidates)
    }
}

/// The asker's own verdict, at the head or at a retained observation, under
/// a fresh request scope and the session's current proof.
pub(super) fn observe_asker<Schema, Program, Context, Ability, Field>(
    session: &WorthQueryHostSession<'_, Schema, Program>,
    at: Option<&WorthQueryApplicationReadObservation>,
    question: &WorthQueryEligibilityQuestion<Schema, Context, Ability, Field>,
) -> Result<WorthQueryAbilityEligibility, WorthQueryEligibilityDenial>
where
    Schema: ApplicationSchema,
    Context: ApplicationBindingContext<Schema>,
    Field: RecordSelection,
    Field::Value: Clone,
{
    let scope = session.fresh_scope();
    let principal = session.call_principal();
    let request = session.request_as(&principal, &scope);
    session
        .with_funded_advancement(&scope, |phase| match at {
            None => request.observe_eligibility(&phase, question),
            Some(observation) => request
                .at(observation)
                .observe_eligibility(&phase, question),
        })
        .map_err(WorthQueryEligibilityDenial::ExecutionRequest)?
        .map_err(WorthQueryEligibilityDenial::Observation)
}

/// Candidate verdicts, at the head or at a retained observation, under a
/// fresh request scope and the session's current proof.
pub(super) fn observe_roster<Schema, Program, Context, Ability, Field>(
    session: &WorthQueryHostSession<'_, Schema, Program>,
    at: Option<&WorthQueryApplicationReadObservation>,
    question: &WorthQueryEligibilityQuestion<Schema, Context, Ability, Field>,
    disclosure: WorthQueryAbilityKey,
    candidates: &[WorthQueryExternalPrincipalIdentity],
) -> Result<WorthQueryEligibilityRoster, WorthQueryEligibilityDenial>
where
    Schema: ApplicationSchema,
    Context: ApplicationBindingContext<Schema>,
    Field: RecordSelection,
    Field::Value: Clone,
{
    let scope = session.fresh_scope();
    let principal = session.call_principal();
    let request = session.request_as(&principal, &scope);
    let verdicts = session
        .with_funded_advancement(&scope, |phase| match at {
            None => request.observe_roster_eligibility(&phase, question, disclosure, candidates),
            Some(observation) => request
                .at(observation)
                .observe_roster_eligibility(&phase, question, disclosure, candidates),
        })
        .map_err(WorthQueryEligibilityDenial::ExecutionRequest)?
        .map_err(WorthQueryEligibilityDenial::Observation)?;
    Ok(WorthQueryEligibilityRoster {
        members: candidates.iter().cloned().zip(verdicts).collect(),
    })
}
