//! Moved whole from `domain_computation::primary_graph::vocabulary::resolution_denial`: the items below and every impl they own.

/// Why an authenticated principal could not be resolved to its principal
/// entity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPrincipalResolutionDenialKind {
    Stopped(
        Result<
            crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind,
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
    pub(in crate::domain_computation) fn new(
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
