//! Forwards the fact-free authority inventory onto `worth-query-execution-query`.

use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_schema::fragment::RecordSelection;
use worth_query_declaration::facade::application_schema::{
    ApplicationBindingContext, ApplicationSchema,
};
use worth_query_execution_query::facade::domain_computation::authorization::eligibility::{
    WorthQueryAbilityEligibilityDenial, WorthQueryAuthorityInventory, WorthQueryAuthorityQuestion,
    WorthQueryEligibilityBasis,
};

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// The subject's authority inventory behind the asker's disclosure; see
    /// the execution-query owner.
    pub fn observe_authority_inventory<Context, Field>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        basis: WorthQueryEligibilityBasis<'_>,
        asker: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        question: &WorthQueryAuthorityQuestion<Schema, Context, Field>,
    ) -> Result<WorthQueryAuthorityInventory, WorthQueryAbilityEligibilityDenial>
    where
        Context: ApplicationBindingContext<Schema>,
        Field: RecordSelection,
        Field::Value: Clone,
    {
        self.inner
            .observe_authority_inventory(phase, basis, asker, request, question)
    }
}
