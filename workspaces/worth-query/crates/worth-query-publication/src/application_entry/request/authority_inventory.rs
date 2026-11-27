//! The fact-free authority inventory on an ordinary request: everything a
//! subject may do on a scope, disclosed only to a request principal holding
//! the disclosure ability there. Head only: capability entries judge grant
//! validity at the one trusted instant of the call, so no retained request
//! offers an inventory over a pinned commit.

use super::WorthQueryApplicationRequest;
use worth_query_declaration::facade::application_schema::fragment::RecordSelection;
use worth_query_declaration::facade::application_schema::ApplicationBindingContext;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase;
use worth_query_execution::facade::eligibility::{
    WorthQueryAbilityEligibilityDenial, WorthQueryAuthorityInventory, WorthQueryAuthorityQuestion,
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
        self.application.observe_authority_inventory(
            phase,
            self.branch,
            self.principal,
            self.scope,
            question,
        )
    }
}
