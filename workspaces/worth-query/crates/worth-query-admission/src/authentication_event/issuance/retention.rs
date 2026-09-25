use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_installation::facade::{WorthQueryNamedClock, WorthQueryNamedClockSource};

use super::{AuthenticationEventClock, WorthQueryAuthenticationEventDenial};

pub(super) struct RetainedAuthenticationEvent {
    pub(super) principal: WorthQueryExternalPrincipalIdentity,
    pub(super) expires_at_nanoseconds: u64,
    pub(super) remaining_uses: u32,
    pub(super) revoked: Arc<AtomicBool>,
}

pub(super) struct AuthenticationEventState<Clock, Source> {
    pub(super) clock: AuthenticationEventClock<Clock, Source>,
    next_identity: u64,
    pub(super) events: HashMap<u64, RetainedAuthenticationEvent>,
    pub(super) pending: HashMap<u64, WorthQueryExternalPrincipalIdentity>,
    pub(super) invalidated_pending: HashSet<u64>,
    pub(super) expiry_order: BTreeSet<(u64, u64)>,
}

impl<Clock, Source> AuthenticationEventState<Clock, Source>
where
    Clock: WorthQueryNamedClock,
    Source: WorthQueryNamedClockSource<Clock>,
{
    pub(super) fn new(clock: AuthenticationEventClock<Clock, Source>) -> Self {
        Self {
            clock,
            next_identity: 1,
            events: HashMap::new(),
            pending: HashMap::new(),
            invalidated_pending: HashSet::new(),
            expiry_order: BTreeSet::new(),
        }
    }

    pub(super) fn next_identity(&mut self) -> Result<u64, WorthQueryAuthenticationEventDenial> {
        let identity = self.next_identity;
        self.next_identity = identity
            .checked_add(1)
            .ok_or(WorthQueryAuthenticationEventDenial::IdentityExhausted)?;
        Ok(identity)
    }

    pub(super) fn remove_event(&mut self, identity: u64) -> Option<RetainedAuthenticationEvent> {
        let retained = self.events.remove(&identity)?;
        self.expiry_order
            .remove(&(retained.expires_at_nanoseconds, identity));
        Some(retained)
    }

    pub(super) fn discard_expired(&mut self, now: u64) {
        while let Some(&(expires_at, identity)) = self.expiry_order.first() {
            if expires_at > now {
                break;
            }
            self.remove_event(identity);
        }
    }
}
