//! Eligibility, roster and authority-inventory refusals: a withheld roster
//! is a permission refusal, an uninstalled or elevation-only capability a
//! caller defect, and an interrupted question retryable.
use worth_query_execution::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    eligibility::WorthQueryAbilityEligibilityDenial as Observation,
};

use crate::session::WorthQueryEligibilityDenial;

use super::super::operator_safety::assert_renders;
use super::super::summary::{WorthQueryRefusalCategory as Category, WorthQueryUserFacingRefusal};

#[test]
fn eligibility_refusals_name_disclosure_defects_and_interruptions_apart() {
    for (cause, category) in [
        (
            Observation::RosterNotDisclosed {
                disclosure: "ViewAuditors",
            },
            Category::PermissionDenied,
        ),
        (
            Observation::CapabilityNotInstalled {
                capability: "AuditCapability",
            },
            Category::Internal,
        ),
        (
            Observation::CapabilityRequiresElevation {
                capability: "AuditCapability",
            },
            Category::Internal,
        ),
        (
            Observation::PrincipalEntityMismatch {
                ability: "ViewAuditors",
            },
            Category::Internal,
        ),
    ] {
        assert_renders(&WorthQueryEligibilityDenial::Observation(cause), category);
    }
    let interrupted =
        WorthQueryEligibilityDenial::ExecutionRequest(WorthQueryAdvancementDenial::Interrupted(
            WorthQueryManagedComputationInterruption::Cancelled,
        ));
    assert_renders(&interrupted, Category::Interrupted);
    assert!(interrupted.retryable());
}
