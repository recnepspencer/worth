use super::*;

pub(super) fn unfunded_principal_admission_denial(
    stop: CompanionPreflightStop,
) -> WorthQueryPrincipalResolutionDenial {
    let kind = match stop {
        CompanionPreflightStop::PreparationMemoryExhausted { .. }
        | CompanionPreflightStop::PreparationMemoryCounterOverflow => {
            WorthQueryPrincipalResolutionDenialKind::ProjectionPreparationMemoryExhausted
        }
        _ => WorthQueryPrincipalResolutionDenialKind::ProjectionWorkBudgetExceeded,
    };
    WorthQueryPrincipalResolutionDenial::new(kind, String::new())
}
