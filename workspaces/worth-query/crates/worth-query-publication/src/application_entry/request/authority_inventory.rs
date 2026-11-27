//! The fact-free authority inventory on an ordinary or retained request:
//! everything a subject may do on a scope, disclosed only to a request
//! principal holding the disclosure ability there at the branch head.

use super::{WorthQueryApplicationRequest, WorthQueryApplicationRetainedRequest};
use worth_query_declaration::facade::application_schema::fragment::RecordSelection;
use worth_query_declaration::facade::application_schema::ApplicationBindingContext;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase;
use worth_query_execution::facade::eligibility::{
    WorthQueryAbilityEligibilityDenial, WorthQueryAuthorityInventory, WorthQueryAuthorityQuestion,
    WorthQueryEligibilityBasis,
};
use worth_query_installation::facade::ApplicationSchema;

type Inventory = Result<WorthQueryAuthorityInventory, WorthQueryAbilityEligibilityDenial>;

impl<Schema: ApplicationSchema> WorthQueryApplicationRequest<'_, '_, '_, Schema> {
    /// The subject's inventory at this request's branch head.
    pub fn observe_authority_inventory<Context, Field>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        question: &WorthQueryAuthorityQuestion<Schema, Context, Field>,
    ) -> Inventory
    where
        Context: ApplicationBindingContext<Schema>,
        Field: RecordSelection,
        Field::Value: Clone,
    {
        let basis = WorthQueryEligibilityBasis::Current(self.branch);
        self.application.observe_authority_inventory(
            phase,
            basis,
            self.principal,
            self.scope,
            question,
        )
    }
}

impl<Schema: ApplicationSchema> WorthQueryApplicationRetainedRequest<'_, '_, '_, Schema> {
    /// The subject's inventory as of the pinned commit; disclosure is still
    /// decided at the branch head.
    pub fn observe_authority_inventory<Context, Field>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        question: &WorthQueryAuthorityQuestion<Schema, Context, Field>,
    ) -> Inventory
    where
        Context: ApplicationBindingContext<Schema>,
        Field: RecordSelection,
        Field::Value: Clone,
    {
        let basis = WorthQueryEligibilityBasis::Retained(&self.observation);
        self.application.observe_authority_inventory(
            phase,
            basis,
            self.principal,
            self.scope,
            question,
        )
    }
}
