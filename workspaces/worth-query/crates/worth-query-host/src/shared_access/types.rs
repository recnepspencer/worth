//! Core value types, bounds, and sealed grant receipts for shared scope access.

use std::time::Duration;

use worth_foundational::facade::{HomeIdentity, HomeIncarnation};
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;

use super::denial::SharedAccessDenial;

pub const MAX_GRANT_ID_BYTES: usize = 128;
pub const MAX_SCOPE_KEY_BYTES: usize = 256;
pub const MIN_MAX_FRAME_BYTES: u32 = 64;
pub const DEFAULT_MAX_FRAME_BYTES: u32 = 256 * 1024;
pub const HARD_MAX_FRAME_BYTES: u32 = 1024 * 1024;
pub const DEFAULT_MAX_ROWS_PER_RESPONSE: u32 = 1024;
pub const DEFAULT_MAX_ACTIVE_WATCHES: u32 = 64;
pub const DEFAULT_MAX_WATCHES_PER_PRINCIPAL: u32 = 16;
pub const DEFAULT_LIVE_BUFFER_CAPACITY: usize = 64;
pub const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 15;
pub const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 300;

/// Access level granted to a collaborator on a single application scope.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ScopeAccess {
    ReadOnly,
    ReadAndWatch,
}

impl ScopeAccess {
    pub const fn read_only() -> Self {
        Self::ReadOnly
    }

    pub const fn read_and_watch() -> Self {
        Self::ReadAndWatch
    }

    pub const fn allows_read(self) -> bool {
        match self {
            Self::ReadOnly | Self::ReadAndWatch => true,
        }
    }

    pub const fn allows_watch(self) -> bool {
        match self {
            Self::ReadOnly => false,
            Self::ReadAndWatch => true,
        }
    }

    pub const fn wire_code(self) -> u8 {
        match self {
            Self::ReadOnly => 1,
            Self::ReadAndWatch => 2,
        }
    }

    pub const fn from_wire_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::ReadOnly),
            2 => Some(Self::ReadAndWatch),
            _ => None,
        }
    }
}

/// Validated canonical identifier of one durable scope-access grant.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ScopeGrantId(String);

impl ScopeGrantId {
    pub fn parse(value: impl Into<String>) -> Result<Self, SharedAccessDenial> {
        let value = value.into();
        if value.is_empty()
            || value.len() > MAX_GRANT_ID_BYTES
            || value.trim() != value
            || value.chars().any(char::is_control)
        {
            return Err(SharedAccessDenial::InvalidGrantIdentifier);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ScopeGrantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Owner-issued receipt for a committed durable scope-access grant.
///
/// Only `WorthQueryApplicationHome::grant_scope_access` constructs this value,
/// strictly after WORTH's own mutation runtime returns a published commit receipt
/// with non-zero durable record changes. Holding it carries metadata for
/// revocation and grants no runtime authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScopeAccessGrant {
    id: ScopeGrantId,
    scope_key: String,
    principal: WorthQueryExternalPrincipalIdentity,
    access: ScopeAccess,
    home_identity: Option<HomeIdentity>,
    home_incarnation: Option<HomeIncarnation>,
    committed_record_changes: usize,
}

pub(super) struct CommittedGrantReceipt {
    pub(super) id: ScopeGrantId,
    pub(super) scope_key: String,
    pub(super) principal: WorthQueryExternalPrincipalIdentity,
    pub(super) access: ScopeAccess,
    pub(super) home_identity: Option<HomeIdentity>,
    pub(super) home_incarnation: Option<HomeIncarnation>,
    pub(super) committed_record_changes: usize,
}

impl ScopeAccessGrant {
    pub(super) fn from_committed(receipt: CommittedGrantReceipt) -> Self {
        Self {
            id: receipt.id,
            scope_key: receipt.scope_key,
            principal: receipt.principal,
            access: receipt.access,
            home_identity: receipt.home_identity,
            home_incarnation: receipt.home_incarnation,
            committed_record_changes: receipt.committed_record_changes,
        }
    }

    pub const fn id(&self) -> &ScopeGrantId {
        &self.id
    }

    pub fn scope_key(&self) -> &str {
        &self.scope_key
    }

    pub const fn principal(&self) -> &WorthQueryExternalPrincipalIdentity {
        &self.principal
    }

    pub const fn access(&self) -> ScopeAccess {
        self.access
    }

    pub const fn home_identity(&self) -> Option<HomeIdentity> {
        self.home_identity
    }

    pub const fn home_incarnation(&self) -> Option<HomeIncarnation> {
        self.home_incarnation
    }

    pub const fn committed_record_changes(&self) -> usize {
        self.committed_record_changes
    }
}

/// Server-assigned identifier for one live governed remote watch handle.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RemoteWatchId(u64);

impl RemoteWatchId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// Result of polling a live governed remote watch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RemoteWatchPollOutcome<Row> {
    Delivered(Vec<Row>),
    Pending,
}

/// Explicit bounded frame, memory, watch, and deadline budgets for remote hosting.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RemoteServerLimits {
    max_frame_bytes: u32,
    max_rows_per_response: u32,
    max_active_watches: u32,
    max_watches_per_principal: u32,
    live_buffer_capacity: usize,
    request_timeout: Duration,
    idle_timeout: Duration,
}

impl Default for RemoteServerLimits {
    fn default() -> Self {
        Self {
            max_frame_bytes: DEFAULT_MAX_FRAME_BYTES,
            max_rows_per_response: DEFAULT_MAX_ROWS_PER_RESPONSE,
            max_active_watches: DEFAULT_MAX_ACTIVE_WATCHES,
            max_watches_per_principal: DEFAULT_MAX_WATCHES_PER_PRINCIPAL,
            live_buffer_capacity: DEFAULT_LIVE_BUFFER_CAPACITY,
            request_timeout: Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS),
            idle_timeout: Duration::from_secs(DEFAULT_IDLE_TIMEOUT_SECS),
        }
    }
}

impl RemoteServerLimits {
    pub const fn with_max_frame_bytes(mut self, max_frame_bytes: u32) -> Option<Self> {
        if max_frame_bytes < MIN_MAX_FRAME_BYTES || max_frame_bytes > HARD_MAX_FRAME_BYTES {
            return None;
        }
        self.max_frame_bytes = max_frame_bytes;
        Some(self)
    }

    pub const fn with_max_rows_per_response(mut self, max_rows: u32) -> Option<Self> {
        if max_rows == 0 {
            return None;
        }
        self.max_rows_per_response = max_rows;
        Some(self)
    }

    pub const fn with_max_active_watches(
        mut self,
        max_active: u32,
        max_per_principal: u32,
    ) -> Option<Self> {
        if max_active == 0 || max_per_principal == 0 || max_per_principal > max_active {
            return None;
        }
        self.max_active_watches = max_active;
        self.max_watches_per_principal = max_per_principal;
        Some(self)
    }

    pub const fn with_live_buffer_capacity(mut self, capacity: usize) -> Option<Self> {
        if capacity == 0 || capacity > 1024 {
            return None;
        }
        self.live_buffer_capacity = capacity;
        Some(self)
    }

    pub const fn with_request_timeout(mut self, timeout: Duration) -> Option<Self> {
        if timeout.is_zero() {
            return None;
        }
        self.request_timeout = timeout;
        Some(self)
    }

    pub const fn with_idle_timeout(mut self, timeout: Duration) -> Option<Self> {
        if timeout.is_zero() {
            return None;
        }
        self.idle_timeout = timeout;
        Some(self)
    }

    pub const fn max_frame_bytes(self) -> u32 {
        self.max_frame_bytes
    }

    pub const fn max_rows_per_response(self) -> u32 {
        self.max_rows_per_response
    }

    pub const fn max_active_watches(self) -> u32 {
        self.max_active_watches
    }

    pub const fn max_watches_per_principal(self) -> u32 {
        self.max_watches_per_principal
    }

    pub const fn live_buffer_capacity(self) -> usize {
        self.live_buffer_capacity
    }

    pub const fn request_timeout(self) -> Duration {
        self.request_timeout
    }

    pub const fn idle_timeout(self) -> Duration {
        self.idle_timeout
    }
}

/// Client-side bounded framing and I/O deadline configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RemoteClientLimits {
    max_frame_bytes: u32,
    io_timeout: Duration,
}

impl Default for RemoteClientLimits {
    fn default() -> Self {
        Self {
            max_frame_bytes: DEFAULT_MAX_FRAME_BYTES,
            io_timeout: Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS),
        }
    }
}

impl RemoteClientLimits {
    pub const fn with_max_frame_bytes(mut self, max_frame_bytes: u32) -> Option<Self> {
        if max_frame_bytes < MIN_MAX_FRAME_BYTES || max_frame_bytes > HARD_MAX_FRAME_BYTES {
            return None;
        }
        self.max_frame_bytes = max_frame_bytes;
        Some(self)
    }

    pub const fn with_io_timeout(mut self, timeout: Duration) -> Option<Self> {
        if timeout.is_zero() {
            return None;
        }
        self.io_timeout = timeout;
        Some(self)
    }

    pub const fn max_frame_bytes(self) -> u32 {
        self.max_frame_bytes
    }

    pub const fn io_timeout(self) -> Duration {
        self.io_timeout
    }
}
