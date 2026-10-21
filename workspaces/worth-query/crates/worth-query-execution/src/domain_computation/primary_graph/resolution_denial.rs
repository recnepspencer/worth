use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionRefusalKind as PrincipalRefusal;
use worth_query_installation::facade::WorthQueryPrincipalBindingInstallationDenialKind;
use worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind;

/// Why an authenticated principal could not be resolved to its principal
/// entity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPrincipalResolutionDenialKind {
    Stopped(
        Result<
            crate::domain_computation::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::WorthQueryProviderSessionControlStopKind,
        >,
    ),
    Refused(WorthQueryPrincipalResolutionRefusalKind),
}

/// An ordinary principal-resolution refusal from its installed binding and exact truth.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPrincipalResolutionRefusalKind {
    /// The supplied phase belongs to another installed runtime.
    ForeignPhase,
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
            WorthQueryPrincipalResolutionDenialKind::Refused(PrincipalRefusal::ForeignRuntime)
        }
        WorthQueryPrincipalBindingInstallationDenialKind::StaleGeneration => {
            WorthQueryPrincipalResolutionDenialKind::Refused(PrincipalRefusal::StaleInstalledSchema)
        }
        WorthQueryPrincipalBindingInstallationDenialKind::BindingNotInstalled
        | WorthQueryPrincipalBindingInstallationDenialKind::BindingMeaningChanged
        | WorthQueryPrincipalBindingInstallationDenialKind::PackageIdentityChanged
        | WorthQueryPrincipalBindingInstallationDenialKind::SchemaMeaningChanged
        | WorthQueryPrincipalBindingInstallationDenialKind::AuthorityMismatch => {
            WorthQueryPrincipalResolutionDenialKind::Refused(PrincipalRefusal::BindingNotInstalled)
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
            WorthQueryPrincipalResolutionDenialKind::Refused(PrincipalRefusal::StalePrincipalProof)
        }
        BoundedEntityFieldLookupDenialKind::IndexNotInstalled
        | BoundedEntityFieldLookupDenialKind::WrongIndexKind
        | BoundedEntityFieldLookupDenialKind::ExactGenerationUnavailable
        | BoundedEntityFieldLookupDenialKind::InvalidCandidateLimit => {
            WorthQueryPrincipalResolutionDenialKind::Refused(
                PrincipalRefusal::IdentityIndexUnavailable,
            )
        }
        BoundedEntityFieldLookupDenialKind::CorruptIndexEntries
        | BoundedEntityFieldLookupDenialKind::StorageParityMismatch => {
            WorthQueryPrincipalResolutionDenialKind::Refused(PrincipalRefusal::CorruptIdentityIndex)
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
            let stop = crate::facade::primary_graph::relational_execution_kind(stop, None);
            let kind = match stop {
                Ok(kind) => WorthQueryPrincipalResolutionDenialKind::Stopped(Ok(kind)),
                Err(
                    crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled,
                ) => WorthQueryPrincipalResolutionDenialKind::Stopped(Err(
                    crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled,
                )),
                Err(
                    crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut,
                ) => WorthQueryPrincipalResolutionDenialKind::Stopped(Err(
                    crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut,
                )),
            };
            return resolution_denial(kind, binding);
        }

        super::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Basis(
            super::WorthQueryExactBasisSnapshotDenial::BranchMaterializationSuspended,
        ) => WorthQueryPrincipalResolutionDenialKind::Refused(
            PrincipalRefusal::BranchMaterializationSuspended,
        ),
        super::index_currency::WorthQueryPrimaryIndexCurrencyDenial::IndexUnavailable(_)
        | super::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Basis(
            super::WorthQueryExactBasisSnapshotDenial::BranchObservationUnavailable
            | super::WorthQueryExactBasisSnapshotDenial::RetentionCapacityExhausted
            | super::WorthQueryExactBasisSnapshotDenial::RetentionIdentityExhausted
            | super::WorthQueryExactBasisSnapshotDenial::ForeignRuntime { .. }
            | super::WorthQueryExactBasisSnapshotDenial::ActiveSnapshotCapacityExhausted { .. }
            | super::WorthQueryExactBasisSnapshotDenial::SnapshotIdentityExhausted,
        ) => WorthQueryPrincipalResolutionDenialKind::Refused(
            PrincipalRefusal::IdentityIndexUnavailable,
        ),
        #[cfg(test)]
        super::index_currency::WorthQueryPrimaryIndexCurrencyDenial::Basis(
            super::WorthQueryExactBasisSnapshotDenial::BranchIdentityUnavailable,
        ) => WorthQueryPrincipalResolutionDenialKind::Refused(
            PrincipalRefusal::IdentityIndexUnavailable,
        ),
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
        Ok(Some(kind)) => Kind::Stopped(Ok(kind)),
        Ok(None) => Kind::Refused(PrincipalRefusal::StalePrincipalProof),
        Err(Control::Cancelled) => Kind::Stopped(Err(
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled,
        )),
        Err(Control::TimedOut) => Kind::Stopped(Err(
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut,
        )),
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
        Ok(kind) => WorthQueryPrincipalResolutionDenialKind::Stopped(Ok(kind)),
        Err(Control::Cancelled) => WorthQueryPrincipalResolutionDenialKind::Stopped(Err(
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled,
        )),
        Err(Control::TimedOut) => WorthQueryPrincipalResolutionDenialKind::Stopped(Err(
            crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut,
        )),
    };
    resolution_denial(kind, binding)
}

impl worth_execution::ChargedBytes for WorthQueryPrincipalResolutionDenial {
    fn additional_charged_bytes(&self) -> u64 {
        // The shared native read-stop has no resident transport contract here.
        // Refuse retained error storage rather than underquote its native backing.
        if self.native_query_read_stop.is_some() {
            u64::MAX
        } else {
            self.binding.capacity() as u64
        }
    }
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
            WorthQueryPrincipalResolutionDenialKind::Refused(
                PrincipalRefusal::BranchMaterializationSuspended
            )
        );
    }
}
