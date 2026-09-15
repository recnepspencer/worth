#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WorthQueryPrincipalResolutionDenialKind {
    PrimaryGraphNotInstalled,
    BindingNotInstalled,
    ForeignRuntime,
    StaleInstalledSchema,
    ExpiredAuthentication,
    Cancelled,
    DeadlineExceeded,
    BranchMaterializationSuspended,
    IdentityIndexUnavailable,
    CorruptIdentityIndex,
    UnknownPrincipal,
    DisabledPrincipal,
    AmbiguousPrincipal,
    MissingPrincipalTarget,
    AmbiguousPrincipalTarget,
    WrongPrincipalTargetKind,
    StalePrincipalProof,
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    SnapshotIdentityExhausted,
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPrincipalResolutionDenial {
    kind: WorthQueryPrincipalResolutionDenialKind,
    binding: String,
}

impl WorthQueryPrincipalResolutionDenial {
    pub(super) fn new(
        kind: WorthQueryPrincipalResolutionDenialKind,
        binding: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            binding: binding.into(),
        }
    }

    pub const fn kind(&self) -> WorthQueryPrincipalResolutionDenialKind {
        self.kind
    }

    pub fn binding(&self) -> &str {
        &self.binding
    }
}

impl std::fmt::Display for WorthQueryPrincipalResolutionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application principal resolution denied: {:?} ({})",
            self.kind, self.binding
        )
    }
}

impl std::error::Error for WorthQueryPrincipalResolutionDenial {}

pub(super) fn principal_binding_resolution_denial(
    kind: WorthQueryPrincipalBindingInstallationDenialKind,
    binding: &str,
) -> WorthQueryPrincipalResolutionDenial {
    let resolution_kind = match kind {
        WorthQueryPrincipalBindingInstallationDenialKind::ForeignRuntime => {
            WorthQueryPrincipalResolutionDenialKind::ForeignRuntime
        }
        WorthQueryPrincipalBindingInstallationDenialKind::StaleGeneration => {
            WorthQueryPrincipalResolutionDenialKind::StaleInstalledSchema
        }
        _ => WorthQueryPrincipalResolutionDenialKind::BindingNotInstalled,
    };
    resolution_denial(resolution_kind, binding)
}

pub(super) fn entity_lookup_resolution_denial(
    kind: BoundedEntityFieldLookupDenialKind,
    binding: &str,
) -> WorthQueryPrincipalResolutionDenial {
    let resolution_kind = match kind {
        BoundedEntityFieldLookupDenialKind::SnapshotUnavailable => {
            WorthQueryPrincipalResolutionDenialKind::StalePrincipalProof
        }
        BoundedEntityFieldLookupDenialKind::IndexNotInstalled
        | BoundedEntityFieldLookupDenialKind::WrongIndexKind
        | BoundedEntityFieldLookupDenialKind::ExactGenerationUnavailable
        | BoundedEntityFieldLookupDenialKind::InvalidCandidateLimit => {
            WorthQueryPrincipalResolutionDenialKind::IdentityIndexUnavailable
        }
        BoundedEntityFieldLookupDenialKind::CorruptIndexEntries
        | BoundedEntityFieldLookupDenialKind::StorageParityMismatch => {
            WorthQueryPrincipalResolutionDenialKind::CorruptIdentityIndex
        }
    };
    resolution_denial(resolution_kind, binding)
}

pub(super) fn principal_index_currency_denial(
    denial: super::index_currency::WorthQueryPrimaryIndexCurrencyDenial,
    binding: &str,
) -> WorthQueryPrincipalResolutionDenial {
    let kind = match denial {
        super::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Basis(
            super::WorthQueryExactBasisSnapshotDenial::BranchMaterializationSuspended,
        ) => WorthQueryPrincipalResolutionDenialKind::BranchMaterializationSuspended,
        _ => WorthQueryPrincipalResolutionDenialKind::IdentityIndexUnavailable,
    };
    resolution_denial(kind, binding)
}

pub(super) fn resolution_denial(
    kind: WorthQueryPrincipalResolutionDenialKind,
    binding: impl Into<String>,
) -> WorthQueryPrincipalResolutionDenial {
    WorthQueryPrincipalResolutionDenial::new(kind, binding)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suspended_materialization_is_not_reported_as_an_index_outage() {
        let denial = principal_index_currency_denial(
            super::super::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Basis(
                super::super::WorthQueryExactBasisSnapshotDenial::BranchMaterializationSuspended,
            ),
            "principal",
        );

        assert_eq!(
            denial.kind(),
            WorthQueryPrincipalResolutionDenialKind::BranchMaterializationSuspended
        );
    }
}
use worth_query_installation::facade::WorthQueryPrincipalBindingInstallationDenialKind;
use worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind;
