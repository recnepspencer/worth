//! Exact typed refusal topology for shared access and remote transport.

use std::fmt;

/// Why remote authentication failed before touching Home data.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum AuthenticationRefusalReason {
    MalformedToken,
    UnknownKey,
    InvalidSignature,
    HomeIdentityMismatch,
    StaleIncarnation,
    IncarnationPinRequired,
    TokenExpired,
    AdapterDenied,
}

/// Why WORTH capability, principal, or scope governance refused access.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum AuthorizationRefusalReason {
    CapabilityDenied,
    PrincipalResolutionDenied,
    ScopeNotFound,
    WatchCapabilityRevoked,
    StalePrincipal,
    StaleScope,
}

/// Why binary wire framing failed closed before payload allocation or dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ProtocolRefusalReason {
    InvalidMagic,
    UnsupportedVersion,
    InvalidFlags,
    FrameTooLarge { requested: u32, maximum: u32 },
    MalformedPayload,
    TrailingBytes,
    UnknownMessageKind,
}

/// Why a bounded resource budget refused a remote request.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum BudgetRefusalReason {
    WatchCapacityExhausted { active: u32, maximum: u32 },
    ResponseTooLarge { bytes: u32, maximum: u32 },
    RequestLimitExceeded,
    ExecutionResourceDenied,
}

/// Why a remote watch lifecycle operation failed closed.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WatchRefusalReason {
    WatchNotFound,
    WatchClosed,
    WatchOverflow,
}

/// Complete typed refusal surface for shared scope access and remote transport.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SharedAccessDenial {
    AuthenticationRefused(AuthenticationRefusalReason),
    AuthorizationRefused(AuthorizationRefusalReason),
    ProtocolRefused(ProtocolRefusalReason),
    BudgetRefused(BudgetRefusalReason),
    WatchRefused(WatchRefusalReason),
    DirectoryHomeRequired,
    NonLoopbackAddressRefused,
    InvalidGrantIdentifier,
    InvalidAuthConfiguration,
    GrantMutationRefused(String),
    HostUnavailable,
    TransportIo(String),
}

impl SharedAccessDenial {
    /// Deterministic binary wire representation `(code, detail_a, detail_b)`.
    pub(super) const fn to_wire_triple(&self) -> (u16, u32, u32) {
        use AuthenticationRefusalReason as Auth;
        use AuthorizationRefusalReason as Authz;
        use BudgetRefusalReason as Budget;
        use ProtocolRefusalReason as Proto;
        use WatchRefusalReason as Watch;
        match self {
            Self::AuthenticationRefused(Auth::MalformedToken) => (101, 0, 0),
            Self::AuthenticationRefused(Auth::UnknownKey) => (102, 0, 0),
            Self::AuthenticationRefused(Auth::InvalidSignature) => (103, 0, 0),
            Self::AuthenticationRefused(Auth::HomeIdentityMismatch) => (104, 0, 0),
            Self::AuthenticationRefused(Auth::StaleIncarnation) => (105, 0, 0),
            Self::AuthenticationRefused(Auth::IncarnationPinRequired) => (106, 0, 0),
            Self::AuthenticationRefused(Auth::TokenExpired) => (107, 0, 0),
            Self::AuthenticationRefused(Auth::AdapterDenied) => (108, 0, 0),
            Self::AuthorizationRefused(Authz::CapabilityDenied) => (201, 0, 0),
            Self::AuthorizationRefused(Authz::PrincipalResolutionDenied) => (202, 0, 0),
            Self::AuthorizationRefused(Authz::ScopeNotFound) => (203, 0, 0),
            Self::AuthorizationRefused(Authz::WatchCapabilityRevoked) => (204, 0, 0),
            Self::AuthorizationRefused(Authz::StalePrincipal) => (205, 0, 0),
            Self::AuthorizationRefused(Authz::StaleScope) => (206, 0, 0),
            Self::ProtocolRefused(Proto::InvalidMagic) => (301, 0, 0),
            Self::ProtocolRefused(Proto::UnsupportedVersion) => (302, 0, 0),
            Self::ProtocolRefused(Proto::InvalidFlags) => (303, 0, 0),
            Self::ProtocolRefused(Proto::FrameTooLarge { requested, maximum }) => {
                (304, *requested, *maximum)
            }
            Self::ProtocolRefused(Proto::MalformedPayload) => (305, 0, 0),
            Self::ProtocolRefused(Proto::TrailingBytes) => (306, 0, 0),
            Self::ProtocolRefused(Proto::UnknownMessageKind) => (307, 0, 0),
            Self::BudgetRefused(Budget::WatchCapacityExhausted { active, maximum }) => {
                (401, *active, *maximum)
            }
            Self::BudgetRefused(Budget::ResponseTooLarge { bytes, maximum }) => {
                (402, *bytes, *maximum)
            }
            Self::BudgetRefused(Budget::RequestLimitExceeded) => (403, 0, 0),
            Self::BudgetRefused(Budget::ExecutionResourceDenied) => (404, 0, 0),
            Self::WatchRefused(Watch::WatchNotFound) => (501, 0, 0),
            Self::WatchRefused(Watch::WatchClosed) => (502, 0, 0),
            Self::WatchRefused(Watch::WatchOverflow) => (503, 0, 0),
            Self::DirectoryHomeRequired => (601, 0, 0),
            Self::InvalidGrantIdentifier => (602, 0, 0),
            Self::InvalidAuthConfiguration => (603, 0, 0),
            Self::GrantMutationRefused(_) => (604, 0, 0),
            Self::HostUnavailable => (605, 0, 0),
            Self::TransportIo(_) => (606, 0, 0),
            Self::NonLoopbackAddressRefused => (607, 0, 0),
        }
    }

    pub(super) fn from_wire_triple(code: u16, a: u32, b: u32) -> Self {
        use AuthenticationRefusalReason as Auth;
        use AuthorizationRefusalReason as Authz;
        use BudgetRefusalReason as Budget;
        use ProtocolRefusalReason as Proto;
        use WatchRefusalReason as Watch;
        match code {
            101 => Self::AuthenticationRefused(Auth::MalformedToken),
            102 => Self::AuthenticationRefused(Auth::UnknownKey),
            103 => Self::AuthenticationRefused(Auth::InvalidSignature),
            104 => Self::AuthenticationRefused(Auth::HomeIdentityMismatch),
            105 => Self::AuthenticationRefused(Auth::StaleIncarnation),
            106 => Self::AuthenticationRefused(Auth::IncarnationPinRequired),
            107 => Self::AuthenticationRefused(Auth::TokenExpired),
            108 => Self::AuthenticationRefused(Auth::AdapterDenied),
            201 => Self::AuthorizationRefused(Authz::CapabilityDenied),
            202 => Self::AuthorizationRefused(Authz::PrincipalResolutionDenied),
            203 => Self::AuthorizationRefused(Authz::ScopeNotFound),
            204 => Self::AuthorizationRefused(Authz::WatchCapabilityRevoked),
            205 => Self::AuthorizationRefused(Authz::StalePrincipal),
            206 => Self::AuthorizationRefused(Authz::StaleScope),
            301 => Self::ProtocolRefused(Proto::InvalidMagic),
            302 => Self::ProtocolRefused(Proto::UnsupportedVersion),
            303 => Self::ProtocolRefused(Proto::InvalidFlags),
            304 => Self::ProtocolRefused(Proto::FrameTooLarge {
                requested: a,
                maximum: b,
            }),
            305 => Self::ProtocolRefused(Proto::MalformedPayload),
            306 => Self::ProtocolRefused(Proto::TrailingBytes),
            307 => Self::ProtocolRefused(Proto::UnknownMessageKind),
            401 => Self::BudgetRefused(Budget::WatchCapacityExhausted {
                active: a,
                maximum: b,
            }),
            402 => Self::BudgetRefused(Budget::ResponseTooLarge {
                bytes: a,
                maximum: b,
            }),
            403 => Self::BudgetRefused(Budget::RequestLimitExceeded),
            404 => Self::BudgetRefused(Budget::ExecutionResourceDenied),
            501 => Self::WatchRefused(Watch::WatchNotFound),
            502 => Self::WatchRefused(Watch::WatchClosed),
            503 => Self::WatchRefused(Watch::WatchOverflow),
            601 => Self::DirectoryHomeRequired,
            602 => Self::InvalidGrantIdentifier,
            603 => Self::InvalidAuthConfiguration,
            604 => Self::GrantMutationRefused("remote grant mutation refused".to_owned()),
            605 => Self::HostUnavailable,
            606 => Self::TransportIo("remote transport error".to_owned()),
            607 => Self::NonLoopbackAddressRefused,
            _ => Self::ProtocolRefused(Proto::MalformedPayload),
        }
    }
}

impl fmt::Display for SharedAccessDenial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "shared access denied: {self:?}")
    }
}

impl std::error::Error for SharedAccessDenial {}
