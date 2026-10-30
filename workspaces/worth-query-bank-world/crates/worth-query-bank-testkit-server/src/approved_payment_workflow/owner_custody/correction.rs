//! A real rail owner settles before an independently admitted journal compensation.
pub use super::super::{
    assertions::{key, require_completed},
    fixture,
    ready_payment::ReadyPaymentWorld,
};
pub use bank_domain::schema::PaymentStatus;
pub use bank_external_rail::test_control::FaultScript;
pub use std::{num::NonZeroUsize, time::Duration};
pub use worth_query_host::facade::{application_entry::*, primary_graph::*};

#[path = "correction/fixture.rs"]
pub mod support;

pub fn limits() -> WorthQueryCorrectionHistoryLimits {
    WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN)
}

pub fn require_pending(
    result: Result<
        bank_server::BankApprovedPaymentCorrectionPreparation<'_>,
        WorthQueryApplicationCorrectionDenial,
    >,
) {
    let Err(denial) = result else {
        panic!("authentic unsettled custody must refuse before an inverse candidate");
    };
    assert!(
        matches!(&denial,
        WorthQueryApplicationCorrectionDenial::Derivation(WorthQueryCorrectionDerivationDenial::Workflow(cause))
            if cause.kind() == WorthQueryApplicationAttemptDenialKind::WorkflowOperationInOwnerCustody),
        "authentic unsettled custody must return its owning refusal: {denial:?}"
    );
}
