use worth_query_installation::facade::WorthQueryPrincipalBindingInstallationDenialKind;
use worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind;

/// Why an authenticated principal could not be resolved to its principal
/// entity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WorthQueryPrincipalResolutionDenialKind {
    /// The primary graph is not installed.
    PrimaryGraphNotInstalled,
    /// The principal binding is not installed.
    BindingNotInstalled,
    /// The principal proof belongs to a different runtime.
    ForeignRuntime,
    /// The installed schema changed since the proof was issued.
    StaleInstalledSchema,
    /// The authentication has expired.
    ExpiredAuthentication,
    /// Resolution was cancelled.
    Cancelled,
    /// Resolution reached its deadline.
    DeadlineExceeded,
    /// The branch's materialization is suspended.
    BranchMaterializationSuspended,
    /// The principal identity index is unavailable.
    IdentityIndexUnavailable,
    /// The identity index disagrees with stored records.
    CorruptIdentityIndex,
    /// No principal has this identity.
    UnknownPrincipal,
    /// The principal is disabled.
    DisabledPrincipal,
    /// More than one principal has this identity.
    AmbiguousPrincipal,
    /// The principal's target entity is missing.
    MissingPrincipalTarget,
    /// The principal has more than one target entity.
    AmbiguousPrincipalTarget,
    /// The principal's target is not the expected entity kind.
    WrongPrincipalTargetKind,
    /// The principal proof no longer matches current state; resolve again.
    StalePrincipalProof,
    /// The installed limit on concurrently active snapshots was reached.
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    /// The runtime ran out of snapshot identities.
    SnapshotIdentityExhausted,
    /// No capacity remains to retain a basis.
    RetentionCapacityExhausted,
    /// The runtime ran out of basis-retention identities.
    RetentionIdentityExhausted,
}

/// Refusal to resolve an authenticated principal.
///
/// [`Self::kind`] says why and [`Self::binding`] names the principal binding.
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
