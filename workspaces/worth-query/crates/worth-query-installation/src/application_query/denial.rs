#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationQueryInstallationDenialKind {
    BindingIdentityCollision,
    BindingMeaningChanged,
    BindingPrincipalMeaningChanged,
    BindingQueryMeaningChanged,
    BindingQueryNotInstalled,
    BindingResultLimitIsZero,
    BindingScopeMeaningChanged,
    BindingWorkLimitIsZero,
    InvalidBindingIdentity,
    QueryNotInstalled,
    QueryMeaningChanged,
    AuthorizationNotInstalled,
    LiveEffectNotInstalled,
    LiveScopeIdentityNotInstalled,
    LiveTargetIdentityNotInstalled,
    RootNotInstalled,
    ProjectionNotInstalled,
    RelationNotInstalled,
    PredicateNotInstalled,
    OrderingNotInstalled,
    ResultShapeDisconnected,
    DependencyCeilingExceeded,
    ForeignRuntime,
    StaleGeneration,
    SchemaMeaningChanged,
    PackageIdentityChanged,
    AuthorityMismatch,
    CanonicalEntryBudgetExceeded,
    CanonicalEncodedByteBudgetExceeded,
    CanonicalDigestSlotRejected,
    InvalidGraphObligationContract,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationQueryInstallationDenial {
    kind: WorthQueryApplicationQueryInstallationDenialKind,
    subject: String,
}

impl WorthQueryApplicationQueryInstallationDenial {
    pub(crate) fn new(
        kind: WorthQueryApplicationQueryInstallationDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }

    pub const fn kind(&self) -> WorthQueryApplicationQueryInstallationDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl std::fmt::Display for WorthQueryApplicationQueryInstallationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application query installation denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryApplicationQueryInstallationDenial {}
