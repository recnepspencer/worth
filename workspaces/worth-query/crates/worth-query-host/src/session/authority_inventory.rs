//! "Explain authority": everything one principal may do on one scope, and
//! why, from the installed abilities and capabilities admission enforces.
//!
//! ```text
//! let question = WorthQueryAuthorityQuestion::new(
//!     Desktop, auditor_identity, SiteFacts::site_id(), "north".into(), ViewAuditors::reference(),
//! );
//! let inventory = session.explain_principal_authority(&question)?;
//! for entry in inventory.entries() { /* authority, Allowed { through } or Denied(cause) */ }
//! ```
//!
//! Revealing a principal's authority is disclosure: the session's principal
//! must hold the question's disclosure ability on the scope at the head.
//! Nothing is retained, reserved or committed, and an entry is not authority.
use worth_query_declaration::facade::application_schema::fragment::RecordSelection;
use worth_query_declaration::facade::application_schema::{
    ApplicationBindingContext, ApplicationSchema,
};
use worth_query_execution::facade::eligibility::{
    WorthQueryAuthorityInventory, WorthQueryAuthorityQuestion,
};
use worth_query_publication::facade::application_entry::WorthQueryApplicationReadObservation;

use super::eligibility::WorthQueryEligibilityDenial;
use super::WorthQueryHostSession;

impl<Schema: ApplicationSchema, Program> WorthQueryHostSession<'_, Schema, Program> {
    /// Every installed ability and capability the question's subject may
    /// exercise on its scope now, with the allowing paths or the policy
    /// cause of each refusal. Refuses `RosterNotDisclosed` before reading
    /// anything about the subject when this session's principal does not
    /// hold the disclosure ability on the scope.
    pub fn explain_principal_authority<Context, Field>(
        &self,
        question: &WorthQueryAuthorityQuestion<Schema, Context, Field>,
    ) -> Result<WorthQueryAuthorityInventory, WorthQueryEligibilityDenial>
    where
        Context: ApplicationBindingContext<Schema>,
        Field: RecordSelection,
        Field::Value: Clone,
    {
        observe_inventory(self, None, question)
    }
}

/// The inventory at the head or at a retained observation, under a fresh
/// request scope and the session's current proof.
pub(super) fn observe_inventory<Schema, Program, Context, Field>(
    session: &WorthQueryHostSession<'_, Schema, Program>,
    at: Option<&WorthQueryApplicationReadObservation>,
    question: &WorthQueryAuthorityQuestion<Schema, Context, Field>,
) -> Result<WorthQueryAuthorityInventory, WorthQueryEligibilityDenial>
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
            None => request.observe_authority_inventory(&phase, question),
            Some(observation) => request
                .at(observation)
                .observe_authority_inventory(&phase, question),
        })
        .map_err(WorthQueryEligibilityDenial::ExecutionRequest)?
        .map_err(WorthQueryEligibilityDenial::Observation)
}
