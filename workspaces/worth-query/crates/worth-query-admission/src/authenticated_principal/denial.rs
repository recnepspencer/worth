use super::WorthQueryRequestInterruption;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryAuthenticationAdapterFailureKind {
    CredentialRejected,
    CredentialExpired,
    CredentialRevoked,
    BindingMismatch,
    DependencyUnavailable,
    Cancelled,
    DeadlineExceeded,
    ProtocolViolation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryAuthenticationAdapterFailure {
    kind: WorthQueryAuthenticationAdapterFailureKind,
}

impl WorthQueryAuthenticationAdapterFailure {
    pub const fn new(kind: WorthQueryAuthenticationAdapterFailureKind) -> Self {
        Self { kind }
    }

    pub const fn kind(self) -> WorthQueryAuthenticationAdapterFailureKind {
        self.kind
    }
}

impl std::fmt::Display for WorthQueryAuthenticationAdapterFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "authentication adapter failed: {:?}", self.kind)
    }
}

impl std::error::Error for WorthQueryAuthenticationAdapterFailure {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryAuthenticationDenialKind {
    ForeignClock,
    AdapterFailed(WorthQueryAuthenticationAdapterFailureKind),
    AudienceMismatch,
    MethodMismatch,
    ValidationTimeInFuture,
    Expired,
    Cancelled,
    DeadlineExceeded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryAuthenticationDenial {
    kind: WorthQueryAuthenticationDenialKind,
}

impl WorthQueryAuthenticationDenial {
    pub(super) const fn new(kind: WorthQueryAuthenticationDenialKind) -> Self {
        Self { kind }
    }

    pub const fn kind(self) -> WorthQueryAuthenticationDenialKind {
        self.kind
    }

    pub(super) const fn interrupted(interruption: WorthQueryRequestInterruption) -> Self {
        let kind = match interruption {
            WorthQueryRequestInterruption::Cancelled => {
                WorthQueryAuthenticationDenialKind::Cancelled
            }
            WorthQueryRequestInterruption::DeadlineExceeded => {
                WorthQueryAuthenticationDenialKind::DeadlineExceeded
            }
        };
        Self { kind }
    }
}

impl std::fmt::Display for WorthQueryAuthenticationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "authentication denied: {:?}", self.kind)
    }
}

impl std::error::Error for WorthQueryAuthenticationDenial {}
