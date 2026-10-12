use worth_query_installation::facade::WorthQueryPrincipalBindingInstallationDenialKind;
use worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind;

/// Why an authenticated principal could not be resolved to its principal
/// entity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPrincipalResolutionDenialKind {
    /// The supplied phase belongs to another installed runtime.
    ForeignPhase,
    ExecutionRequest(crate::domain_computation::WorthQueryProviderSessionDenialKind),
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
    /// Fresh principal resolution exceeded its carried Work allowance.
    ProjectionWorkBudgetExceeded,
    /// Fresh principal resolution could not reserve its preparation backing.
    ProjectionPreparationMemoryExhausted,
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
    ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: usize,
    },
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
    native_query_read_stop:
        Option<std::sync::Arc<worth_relational::facade::runtime::QueryReadExecutionStop>>,
}

impl WorthQueryPrincipalResolutionDenial {
    pub(super) fn new(
        kind: WorthQueryPrincipalResolutionDenialKind,
        binding: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            binding: binding.into(),
            native_query_read_stop: None,
        }
    }

    pub const fn kind(&self) -> WorthQueryPrincipalResolutionDenialKind {
        self.kind
    }

    pub fn binding(&self) -> &str {
        &self.binding
    }

    /// The original Relational read refusal, shared when this denial is cloned.
    pub fn native_query_read_stop(
        &self,
    ) -> Option<&worth_relational::facade::runtime::QueryReadExecutionStop> {
        self.native_query_read_stop.as_deref()
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

impl std::error::Error for WorthQueryPrincipalResolutionDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.native_query_read_stop()
            .map(|stop| stop as &(dyn std::error::Error + 'static))
    }
}

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
        WorthQueryPrincipalBindingInstallationDenialKind::BindingNotInstalled
        | WorthQueryPrincipalBindingInstallationDenialKind::BindingMeaningChanged
        | WorthQueryPrincipalBindingInstallationDenialKind::PackageIdentityChanged
        | WorthQueryPrincipalBindingInstallationDenialKind::SchemaMeaningChanged
        | WorthQueryPrincipalBindingInstallationDenialKind::AuthorityMismatch => {
            WorthQueryPrincipalResolutionDenialKind::BindingNotInstalled
        }
    };
    resolution_denial(resolution_kind, binding)
}

pub(super) fn entity_lookup_resolution_denial(
    kind: BoundedEntityFieldLookupDenialKind,
    binding: &str,
) -> WorthQueryPrincipalResolutionDenial {
    let resolution_kind = match kind {
        BoundedEntityFieldLookupDenialKind::Execution(cause) => {
            return relational_read_denial(cause, binding)
        }
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
        super::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Execution(stop) => {
            let kind = match stop {
                Ok(kind) => WorthQueryPrincipalResolutionDenialKind::ExecutionRequest(kind),
                Err(
                    crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled,
                ) => WorthQueryPrincipalResolutionDenialKind::Cancelled,
                Err(
                    crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut,
                ) => WorthQueryPrincipalResolutionDenialKind::DeadlineExceeded,
            };
            return resolution_denial(kind, binding);
        }

        super::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Basis(
            super::WorthQueryExactBasisSnapshotDenial::BranchMaterializationSuspended,
        ) => WorthQueryPrincipalResolutionDenialKind::BranchMaterializationSuspended,
        super::index_currency::WorthQueryPrimaryIndexCurrencyDenial::IndexUnavailable(_)
        | super::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Basis(
            super::WorthQueryExactBasisSnapshotDenial::BranchObservationUnavailable
            | super::WorthQueryExactBasisSnapshotDenial::RetentionCapacityExhausted
            | super::WorthQueryExactBasisSnapshotDenial::RetentionIdentityExhausted
            | super::WorthQueryExactBasisSnapshotDenial::ForeignRuntime { .. }
            | super::WorthQueryExactBasisSnapshotDenial::ActiveSnapshotCapacityExhausted { .. }
            | super::WorthQueryExactBasisSnapshotDenial::SnapshotIdentityExhausted,
        ) => WorthQueryPrincipalResolutionDenialKind::IdentityIndexUnavailable,
        #[cfg(test)]
        super::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Basis(
            super::WorthQueryExactBasisSnapshotDenial::BranchIdentityUnavailable,
        ) => WorthQueryPrincipalResolutionDenialKind::IdentityIndexUnavailable,
    };
    resolution_denial(kind, binding)
}

pub(super) fn resolution_denial(
    kind: WorthQueryPrincipalResolutionDenialKind,
    binding: impl Into<String>,
) -> WorthQueryPrincipalResolutionDenial {
    WorthQueryPrincipalResolutionDenial::new(kind, binding)
}

pub(super) fn query_read_resolution_denial(
    stop: worth_relational::facade::runtime::QueryReadExecutionStop,
    binding: &str,
) -> WorthQueryPrincipalResolutionDenial {
    use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
    use WorthQueryPrincipalResolutionDenialKind as Kind;
    let kind = match super::provider::relational_execution_denial::query_read_execution_kind(&stop)
    {
        Ok(Some(kind)) => Kind::ExecutionRequest(kind),
        Ok(None) => Kind::StalePrincipalProof,
        Err(Control::Cancelled) => Kind::Cancelled,
        Err(Control::TimedOut) => Kind::DeadlineExceeded,
    };
    WorthQueryPrincipalResolutionDenial {
        kind,
        binding: binding.to_owned(),
        native_query_read_stop: Some(std::sync::Arc::new(stop)),
    }
}

pub(super) fn relational_read_denial(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
    binding: &str,
) -> WorthQueryPrincipalResolutionDenial {
    use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
    let kind = match super::provider::relational_execution_denial::relational_execution_kind(
        cause, None,
    ) {
        Ok(kind) => WorthQueryPrincipalResolutionDenialKind::ExecutionRequest(kind),
        Err(Control::Cancelled) => WorthQueryPrincipalResolutionDenialKind::Cancelled,
        Err(Control::TimedOut) => WorthQueryPrincipalResolutionDenialKind::DeadlineExceeded,
    };
    resolution_denial(kind, binding)
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
