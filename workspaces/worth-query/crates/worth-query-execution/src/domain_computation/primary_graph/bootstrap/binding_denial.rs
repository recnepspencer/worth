use super::WorthQueryPrimaryGraphInstallationDenialKind;
use worth_query_installation::facade::WorthQueryPrincipalBindingInstallationDenialKind;

pub(super) fn map_binding_denial_kind(
    kind: WorthQueryPrincipalBindingInstallationDenialKind,
) -> WorthQueryPrimaryGraphInstallationDenialKind {
    match kind {
        WorthQueryPrincipalBindingInstallationDenialKind::ForeignRuntime => {
            WorthQueryPrimaryGraphInstallationDenialKind::ForeignRuntime
        }
        WorthQueryPrincipalBindingInstallationDenialKind::StaleGeneration => {
            WorthQueryPrimaryGraphInstallationDenialKind::StaleInstalledSchema
        }
        WorthQueryPrincipalBindingInstallationDenialKind::BindingMeaningChanged
        | WorthQueryPrincipalBindingInstallationDenialKind::SchemaMeaningChanged => {
            WorthQueryPrimaryGraphInstallationDenialKind::BindingSchemaMismatch
        }
        WorthQueryPrincipalBindingInstallationDenialKind::BindingNotInstalled
        | WorthQueryPrincipalBindingInstallationDenialKind::PackageIdentityChanged
        | WorthQueryPrincipalBindingInstallationDenialKind::AuthorityMismatch => {
            WorthQueryPrimaryGraphInstallationDenialKind::BindingNotInstalled
        }
    }
}
