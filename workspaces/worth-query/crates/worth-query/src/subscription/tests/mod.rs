use super::*;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

fn roomy_budget() -> QuerySubscriptionWorkBudget {
    QuerySubscriptionWorkBudget::scratch_buffer_only(8, 8, 8, 32, 1)
}

fn roomy_slice_budget() -> QuerySubscriptionSliceBudget {
    QuerySubscriptionSliceBudget::scratch_buffer_only(8, 8, 8, 8, 8, 8, 8, 8)
}

fn roomy_lowering_budget() -> QuerySubscriptionBridgeLoweringBudget {
    QuerySubscriptionBridgeLoweringBudget::admitted(1, 8, 8, 1, 1)
}

fn continuation_test_identity(label: &str) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "subscription_continuation_test_identity_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("label"), label)
        .seal()
        .expect("fixture digest must fit")
}

mod active;
mod active_delivery;
mod admission;
mod certification;
mod declaration;
mod diagnostic_bundles;
mod diagnostics;
mod runtime_certification;
mod runtime_certification_closure_support;
mod selection;
mod support;

pub(crate) use runtime_certification_closure_support::runtime_backed_subscription_certification_summary;
