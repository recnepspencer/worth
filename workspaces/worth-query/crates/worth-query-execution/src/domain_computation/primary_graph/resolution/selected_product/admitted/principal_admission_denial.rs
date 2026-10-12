use super::*;

pub(super) fn unfunded_principal_admission_denial(
    stop: CompanionPreflightStop,
) -> WorthQueryPrincipalResolutionDenial {
    super::super::super::super::resolution::principal_admission_denial(stop, "")
}
