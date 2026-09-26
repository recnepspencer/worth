use std::marker::PhantomData;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Instant;

use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_installation::facade::{
    WorthQueryClockSourceIdentity, WorthQueryClockTimelineIdentity,
};

use crate::authentication_event::intent::{
    WorthQueryAuthenticationEventIntent, WorthQueryAuthenticationEventPolicy,
};

/// Sealed, move-only authentication evidence; the retained owner record, not
/// these descriptive fields, decides whether it may still be consumed.
///
/// Caller-authored data cannot construct or copy the proof:
///
/// ```compile_fail
/// use worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEvent;
/// let _: WorthQueryAuthenticationEvent<()> = serde_json::from_str("{}").unwrap();
/// ```
///
/// ```compile_fail
/// use worth_query_admission::facade::authentication_event::WorthQueryAuthenticationEvent;
/// fn duplicate(event: &WorthQueryAuthenticationEvent<()>) -> WorthQueryAuthenticationEvent<()> {
///     event.clone()
/// }
/// ```
pub struct WorthQueryAuthenticationEvent<Schema> {
    pub(super) owner_identity: Arc<()>,
    pub(super) event_id: u64,
    pub(super) binding_identity: ApplicationSchemaBindingIdentity,
    pub(super) principal: WorthQueryExternalPrincipalIdentity,
    pub(super) intent: WorthQueryAuthenticationEventIntent,
    pub(super) policy: WorthQueryAuthenticationEventPolicy,
    pub(super) source_identity: WorthQueryClockSourceIdentity,
    pub(super) timeline_identity: WorthQueryClockTimelineIdentity,
    pub(super) issued_at_nanoseconds: u64,
    pub(super) expires_at_nanoseconds: u64,
    pub(super) revoked: Arc<AtomicBool>,
    pub(super) marker: PhantomData<fn() -> Schema>,
}

impl<Schema> WorthQueryAuthenticationEvent<Schema> {
    pub const fn principal(&self) -> &WorthQueryExternalPrincipalIdentity {
        &self.principal
    }

    pub const fn intent(&self) -> &WorthQueryAuthenticationEventIntent {
        &self.intent
    }

    pub const fn policy(&self) -> WorthQueryAuthenticationEventPolicy {
        self.policy
    }

    pub const fn source_identity(&self) -> &WorthQueryClockSourceIdentity {
        &self.source_identity
    }

    pub const fn timeline_identity(&self) -> &WorthQueryClockTimelineIdentity {
        &self.timeline_identity
    }

    pub const fn issued_at_nanoseconds(&self) -> u64 {
        self.issued_at_nanoseconds
    }

    pub const fn expires_at_nanoseconds(&self) -> u64 {
        self.expires_at_nanoseconds
    }
}

/// One admitted consumption for a signing operation. It is not approval or
/// operation authority and must be moved into the approval owner's handoff.
#[must_use]
pub struct WorthQueryConsumedAuthenticationEvent<Schema> {
    pub(super) owner_identity: Arc<()>,
    pub(super) binding_identity: ApplicationSchemaBindingIdentity,
    pub(super) principal: WorthQueryExternalPrincipalIdentity,
    pub(super) intent: WorthQueryAuthenticationEventIntent,
    pub(super) source_identity: WorthQueryClockSourceIdentity,
    pub(super) timeline_identity: WorthQueryClockTimelineIdentity,
    pub(super) policy: WorthQueryAuthenticationEventPolicy,
    pub(super) issued_at_nanoseconds: u64,
    pub(super) expires_at_nanoseconds: u64,
    pub(super) observed_at_nanoseconds: u64,
    pub(super) principal_valid_until: Instant,
    pub(super) revoked: Arc<AtomicBool>,
    pub(super) marker: PhantomData<fn() -> Schema>,
}

impl<Schema> WorthQueryConsumedAuthenticationEvent<Schema> {
    pub const fn principal(&self) -> &WorthQueryExternalPrincipalIdentity {
        &self.principal
    }

    pub const fn intent(&self) -> &WorthQueryAuthenticationEventIntent {
        &self.intent
    }

    pub const fn policy(&self) -> WorthQueryAuthenticationEventPolicy {
        self.policy
    }

    pub const fn issued_at_nanoseconds(&self) -> u64 {
        self.issued_at_nanoseconds
    }

    pub const fn observed_at_nanoseconds(&self) -> u64 {
        self.observed_at_nanoseconds
    }

    pub const fn source_identity(&self) -> &WorthQueryClockSourceIdentity {
        &self.source_identity
    }

    pub const fn timeline_identity(&self) -> &WorthQueryClockTimelineIdentity {
        &self.timeline_identity
    }
}
