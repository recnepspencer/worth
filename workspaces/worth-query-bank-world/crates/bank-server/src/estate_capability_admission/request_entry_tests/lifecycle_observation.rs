//! Recorded meaning agrees with the actual selected head of its owning fixture.
use crate::{estate_capability_admission::fixture::CapabilityFixture, BankAuthenticatedPrincipal};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    primary_graph::WorthQueryRecoveredApplicationCommitReceipt,
};
pub(super) fn assert_selected_original(
    fixture: &CapabilityFixture,
    principal: &BankAuthenticatedPrincipal,
    scope: &WorthQueryRequestScope,
    receipt: &WorthQueryRecoveredApplicationCommitReceipt,
) {
    assert_eq!(
        receipt
            .new_product_head()
            .memory()
            .unwrap()
            .selected_commit(),
        fixture.lifecycle_head(principal, scope)
    );
}
