//! Forwards fact-free ability eligibility onto `worth-query-execution-query`.

use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_schema::fragment::RecordSelection;
use worth_query_declaration::facade::application_schema::{
    ApplicationBindingContext, ApplicationSchema,
};
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_execution_query::facade::domain_computation::authorization::eligibility::{
    WorthQueryAbilityEligibility, WorthQueryAbilityEligibilityDenial, WorthQueryAbilityKey,
    WorthQueryEligibilityBasis, WorthQueryEligibilityQuestion,
};

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// The asker's own verdict; see the execution-query owner.
    pub fn observe_asker_eligibility<Context, Ability, Field>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        basis: WorthQueryEligibilityBasis<'_>,
        asker: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        question: &WorthQueryEligibilityQuestion<Schema, Context, Ability, Field>,
    ) -> Result<WorthQueryAbilityEligibility, WorthQueryAbilityEligibilityDenial>
    where
        Context: ApplicationBindingContext<Schema>,
        Field: RecordSelection,
        Field::Value: Clone,
    {
        self.inner
            .observe_asker_eligibility(phase, basis, asker, request, question)
    }

    /// Per-candidate verdicts behind the asker's disclosure; see the
    /// execution-query owner.
    pub fn observe_roster_eligibility<Context, Ability, Field>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        basis: WorthQueryEligibilityBasis<'_>,
        asker: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        question: &WorthQueryEligibilityQuestion<Schema, Context, Ability, Field>,
        disclosure: WorthQueryAbilityKey,
        candidates: &[WorthQueryExternalPrincipalIdentity],
    ) -> Result<Vec<WorthQueryAbilityEligibility>, WorthQueryAbilityEligibilityDenial>
    where
        Context: ApplicationBindingContext<Schema>,
        Field: RecordSelection,
        Field::Value: Clone,
    {
        self.inner.observe_roster_eligibility(
            phase, basis, asker, request, question, disclosure, candidates,
        )
    }
}
