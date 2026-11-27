//! Forwards fact-free capability eligibility onto `worth-query-execution-query`.

use crate::basis::WorthQueryProductBranch;
use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_declaration::facade::application_schema::{
    ApplicationBindingContext, ApplicationOperationMarkerIdentity, ApplicationSchema,
    ApplicationStructuredValueBinding,
};
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_execution_query::facade::domain_computation::authorization::eligibility::{
    WorthQueryAbilityEligibilityDenial, WorthQueryAbilityKey, WorthQueryCapabilityEligibility,
    WorthQueryCapabilityEligibilityQuestion,
};

type Question<Schema, Context, Capability, Operation, Input> =
    WorthQueryCapabilityEligibilityQuestion<Schema, Context, Capability, Operation, Input>;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// The asker's own capability verdict; see the execution-query owner.
    pub fn observe_asker_capability_eligibility<Context, Capability, Operation, Input>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        branch: WorthQueryProductBranch,
        asker: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        question: &Question<Schema, Context, Capability, Operation, Input>,
    ) -> Result<WorthQueryCapabilityEligibility, WorthQueryAbilityEligibilityDenial>
    where
        Context: ApplicationBindingContext<Schema>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability> + 'static,
    {
        self.inner
            .observe_asker_capability_eligibility(phase, branch, asker, request, question)
    }

    /// Per-candidate capability verdicts behind the asker's disclosure; see
    /// the execution-query owner.
    pub fn observe_roster_capability_eligibility<Context, Capability, Operation, Input>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        branch: WorthQueryProductBranch,
        asker: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        question: &Question<Schema, Context, Capability, Operation, Input>,
        disclosure: WorthQueryAbilityKey,
        candidates: &[WorthQueryExternalPrincipalIdentity],
    ) -> Result<Vec<WorthQueryCapabilityEligibility>, WorthQueryAbilityEligibilityDenial>
    where
        Context: ApplicationBindingContext<Schema>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability> + 'static,
    {
        self.inner.observe_roster_capability_eligibility(
            phase, branch, asker, request, question, disclosure, candidates,
        )
    }
}
