//! "Who may act?" observations: ability and capability eligibility, rosters
//! and authority inventories. A refused question minted and wrote nothing.
use worth_query_execution::facade::{
    eligibility::WorthQueryAbilityEligibilityDenial,
    primary_graph::WorthQueryPrincipalResolutionDenialKind as Principal,
};

use crate::session::WorthQueryEligibilityDenial;

use super::request::{BUSY, INTERNAL};
use super::stops::{authorization, principal_resolution, stopped};
use super::summary::{
    WorthQueryRefusalCategory as Category, WorthQueryRefusalSummary as Summary,
    WorthQueryUserFacingRefusal,
};

const ROSTER_WITHHELD: Summary = Summary::with_message(
    Category::PermissionDenied,
    "You don't have permission to see who can do this.",
);

impl WorthQueryUserFacingRefusal for WorthQueryEligibilityDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::ExecutionRequest(cause) => cause.user_summary(),
            Self::Observation(cause) => cause.user_summary(),
        }
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryAbilityEligibilityDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::RosterNotDisclosed { .. } => ROSTER_WITHHELD,
            Self::Asker(denial) => principal_resolution(denial.kind()),
            // Not the candidate's own mapping (that is a verdict): a stop
            // resolving it, never the asker's missing permission.
            Self::Candidate(denial) => match denial.kind() {
                Principal::Stopped(stop) => stopped(&stop),
                Principal::Refused(_) => INTERNAL,
            },
            Self::Authorization(denial) | Self::CapabilityRequest(denial) => {
                authorization(&denial.kind())
            }
            Self::Scope(cause) => cause.user_summary(),
            Self::Basis(_) => BUSY,
            // The question names an authority this program does not install,
            // or one a fact-free question cannot carry: a caller defect.
            Self::PrincipalBinding(_)
            | Self::AbilityNotInstalled { .. }
            | Self::PrincipalEntityMismatch { .. }
            | Self::CapabilityNotInstalled { .. }
            | Self::CapabilityRequiresElevation { .. } => INTERNAL,
        }
    }
}

#[cfg(test)]
#[path = "eligibility_tests.rs"]
mod tests;
