//! Correct the original closed movement through current Bank admission.
pub use super::{key, read_controls, request_scope, BankMutationControls};
pub use bank_domain::{
    model::{AccountId, BankPrincipalId, JournalEntryId, Money},
    schema::{ApplyOpeningFunding, MoneyMovementResult, SendMoney},
};
pub use bank_server::{mutations, queries};
pub use worth_query_host::facade::{
    application_entry::*,
    declaration::{
        application_operation::ApplicationMutationIdentityDenial,
        application_schema::ApplicationValueEncodeDenial,
    },
    primary_graph::{WorthQueryCorrectionSelectionDenial, WorthQueryRecordedCorrectionSource},
};

#[path = "recorded_correction/custody.rs"]
pub mod custody;
#[path = "recorded_correction/fixture.rs"]
pub mod fixture;
#[path = "recorded_correction/retained.rs"]
pub mod retained;

pub fn limits() -> WorthQueryCorrectionHistoryLimits {
    WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::new(1).unwrap())
}
