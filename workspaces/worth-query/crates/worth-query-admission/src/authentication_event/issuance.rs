use std::marker::PhantomData;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationSchemaBindingIdentity,
};
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;
use worth_query_installation::facade::{WorthQueryNamedClock, WorthQueryNamedClockSource};

use crate::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestInterruption, WorthQueryRequestScope,
};

use super::clock::AuthenticationEventClock;
use super::intent::{
    WorthQueryAuthenticationEventChallenge, WorthQueryAuthenticationEventIntent,
    WorthQueryAuthenticationEventPolicy,
};

mod contract;
mod proof;
mod readmission;
mod retention;
pub use contract::{
    WorthQueryAuthenticationEventDenial, WorthQueryAuthenticationEventFuture,
    WorthQueryAuthenticationEventVerifier, WorthQueryAuthenticationEventVerifierFailure,
};
pub use proof::{WorthQueryAuthenticationEvent, WorthQueryConsumedAuthenticationEvent};
use retention::{AuthenticationEventState, RetainedAuthenticationEvent};

/// One installation-bound owner for challenge issuance, named-clock age,
/// revocation, finite retention and atomic reuse accounting.
pub struct WorthQueryInstalledAuthenticationEventOwner<Schema, Clock, Source, Verifier> {
    verifier: Verifier,
    binding_identity: ApplicationSchemaBindingIdentity,
    owner_identity: Arc<()>,
    policy: WorthQueryAuthenticationEventPolicy,
    maximum_events: NonZeroUsize,
    state: Mutex<AuthenticationEventState<Clock, Source>>,
    marker: PhantomData<fn() -> Schema>,
}

/// A sealed signing capability retained by the installed workflow runtime.
/// Callers cannot substitute a different verifier when submitting an approval.
pub struct WorthQueryAuthenticationEventSigningOwner<Schema> {
    binding_identity: ApplicationSchemaBindingIdentity,
    consume: Arc<SigningConsumption<Schema>>,
    readmit: Arc<SigningReadmission<Schema>>,
}

type SigningConsumption<Schema> = dyn Fn(
        &WorthQueryAuthenticationEvent<Schema>,
        &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        &WorthQueryAuthenticationEventIntent,
        &WorthQueryRequestScope,
    )
        -> Result<WorthQueryConsumedAuthenticationEvent<Schema>, WorthQueryAuthenticationEventDenial>
    + Send
    + Sync;

type SigningReadmission<Schema> = dyn Fn(
        WorthQueryConsumedAuthenticationEvent<Schema>,
    )
        -> Result<WorthQueryConsumedAuthenticationEvent<Schema>, WorthQueryAuthenticationEventDenial>
    + Send
    + Sync;

impl<Schema> WorthQueryAuthenticationEventSigningOwner<Schema> {
    pub fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding_identity
    }

    pub fn consume_for_signing(
        &self,
        event: &WorthQueryAuthenticationEvent<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        intent: &WorthQueryAuthenticationEventIntent,
        scope: &WorthQueryRequestScope,
    ) -> Result<WorthQueryConsumedAuthenticationEvent<Schema>, WorthQueryAuthenticationEventDenial>
    {
        (self.consume)(event, principal, intent, scope)
    }

    pub fn readmit_for_publication(
        &self,
        consumed: WorthQueryConsumedAuthenticationEvent<Schema>,
    ) -> Result<WorthQueryConsumedAuthenticationEvent<Schema>, WorthQueryAuthenticationEventDenial>
    {
        (self.readmit)(consumed)
    }
}

impl<Schema> Clone for WorthQueryAuthenticationEventSigningOwner<Schema> {
    fn clone(&self) -> Self {
        Self {
            binding_identity: self.binding_identity.clone(),
            consume: Arc::clone(&self.consume),
            readmit: Arc::clone(&self.readmit),
        }
    }
}

/// Admission is reserved before external verification. Dropping a cancelled
/// authentication future releases exactly its unfinished reservation.
struct AuthenticationEventReservation<'a, Clock, Source> {
    state: &'a Mutex<AuthenticationEventState<Clock, Source>>,
    identity: u64,
    active: bool,
}

impl<Clock, Source> Drop for AuthenticationEventReservation<'_, Clock, Source> {
    fn drop(&mut self) {
        if self.active {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state.pending.remove(&self.identity);
            state.invalidated_pending.remove(&self.identity);
        }
    }
}

pub fn install_authentication_event_owner<Schema, Clock, Source, Verifier>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    source: Source,
    verifier: Verifier,
    policy: WorthQueryAuthenticationEventPolicy,
    maximum_events: NonZeroUsize,
) -> Result<
    WorthQueryInstalledAuthenticationEventOwner<Schema, Clock, Source, Verifier>,
    WorthQueryAuthenticationEventDenial,
>
where
    Schema: ApplicationSchema,
    Clock: WorthQueryNamedClock,
    Source: WorthQueryNamedClockSource<Clock>,
    Verifier: WorthQueryAuthenticationEventVerifier,
{
    let identity = verifier.configuration_identity();
    if identity.is_empty()
        || identity.trim() != identity
        || identity.len() > 512
        || identity.chars().any(char::is_control)
    {
        return Err(WorthQueryAuthenticationEventDenial::InvalidInstallation);
    }
    let clock = AuthenticationEventClock::install(source)?;
    Ok(WorthQueryInstalledAuthenticationEventOwner {
        verifier,
        binding_identity: schema.binding_identity(),
        owner_identity: Arc::new(()),
        policy,
        maximum_events,
        state: Mutex::new(AuthenticationEventState::new(clock)),
        marker: PhantomData,
    })
}

impl<Schema, Clock, Source, Verifier>
    WorthQueryInstalledAuthenticationEventOwner<Schema, Clock, Source, Verifier>
where
    Clock: WorthQueryNamedClock,
    Source: WorthQueryNamedClockSource<Clock>,
    Verifier: WorthQueryAuthenticationEventVerifier,
{
    pub fn signing_owner(self: &Arc<Self>) -> WorthQueryAuthenticationEventSigningOwner<Schema>
    where
        Schema: 'static,
        Clock: Send + Sync + 'static,
        Source: Send + Sync + 'static,
        Verifier: Send + Sync + 'static,
    {
        let owner = Arc::clone(self);
        let readmission_owner = Arc::clone(self);
        WorthQueryAuthenticationEventSigningOwner {
            binding_identity: self.binding_identity.clone(),
            consume: Arc::new(move |event, principal, intent, scope| {
                owner.consume_for_signing(event, principal, intent, scope)
            }),
            readmit: Arc::new(move |consumed| readmission_owner.readmit_for_publication(consumed)),
        }
    }

    pub async fn authenticate(
        &self,
        credential: Verifier::Credential,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        intent: WorthQueryAuthenticationEventIntent,
        scope: &WorthQueryRequestScope,
    ) -> Result<WorthQueryAuthenticationEvent<Schema>, WorthQueryAuthenticationEventDenial> {
        self.check_principal(principal)?;
        check_interruption(scope)?;
        let mut reservation = self.reserve_challenge(principal.identity())?;
        let event_id = reservation.identity;
        let mut nonce = [0_u8; 32];
        getrandom::fill(&mut nonce)
            .map_err(|_| WorthQueryAuthenticationEventDenial::EntropyUnavailable)?;
        let challenge = WorthQueryAuthenticationEventChallenge::issued(
            nonce,
            principal.identity().clone(),
            intent.clone(),
        );
        self.verifier
            .verify(credential, &challenge, scope)
            .await
            .map_err(WorthQueryAuthenticationEventDenial::VerifierFailed)?;
        check_interruption(scope)?;
        self.check_principal(principal)?;
        let mut state = self.state();
        if state.invalidated_pending.contains(&event_id) {
            return Err(WorthQueryAuthenticationEventDenial::EventRevoked);
        }
        let now = state.clock.observe()?;
        let expires_at = now
            .checked_add(self.policy.maximum_age_nanoseconds())
            .ok_or(WorthQueryAuthenticationEventDenial::ClockRangeExceeded)?;
        let revoked = Arc::new(AtomicBool::new(false));
        state.events.insert(
            event_id,
            RetainedAuthenticationEvent {
                principal: principal.identity().clone(),
                expires_at_nanoseconds: expires_at,
                remaining_uses: self.policy.reuse().maximum_uses(),
                revoked: Arc::clone(&revoked),
            },
        );
        state.expiry_order.insert((expires_at, event_id));
        state.pending.remove(&event_id);
        state.invalidated_pending.remove(&event_id);
        reservation.active = false;
        Ok(WorthQueryAuthenticationEvent {
            owner_identity: Arc::clone(&self.owner_identity),
            event_id,
            binding_identity: self.binding_identity.clone(),
            principal: principal.identity().clone(),
            intent,
            policy: self.policy,
            source_identity: state.clock.source_identity().clone(),
            timeline_identity: state.clock.timeline_identity().clone(),
            issued_at_nanoseconds: now,
            expires_at_nanoseconds: expires_at,
            revoked,
            marker: PhantomData,
        })
    }

    pub fn consume_for_signing(
        &self,
        event: &WorthQueryAuthenticationEvent<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        expected_intent: &WorthQueryAuthenticationEventIntent,
        scope: &WorthQueryRequestScope,
    ) -> Result<WorthQueryConsumedAuthenticationEvent<Schema>, WorthQueryAuthenticationEventDenial>
    {
        self.check_event(event)?;
        self.check_principal(principal)?;
        check_interruption(scope)?;
        if event.principal != *principal.identity() {
            return Err(WorthQueryAuthenticationEventDenial::PrincipalMismatch);
        }
        if event.intent != *expected_intent {
            return Err(WorthQueryAuthenticationEventDenial::IntentMismatch);
        }
        if event.revoked.load(Ordering::Acquire) {
            return Err(WorthQueryAuthenticationEventDenial::EventRevoked);
        }
        let mut state = self.state();
        let now = state.clock.observe()?;
        if now >= event.expires_at_nanoseconds {
            state.remove_event(event.event_id);
            return Err(WorthQueryAuthenticationEventDenial::EventExpired);
        }
        state.discard_expired(now);
        let retained = state
            .events
            .get_mut(&event.event_id)
            .ok_or(WorthQueryAuthenticationEventDenial::EventUnavailable)?;
        if retained.revoked.load(Ordering::Acquire) {
            return Err(WorthQueryAuthenticationEventDenial::EventRevoked);
        }
        retained.remaining_uses -= 1;
        if retained.remaining_uses == 0 {
            state.remove_event(event.event_id);
        }
        Ok(WorthQueryConsumedAuthenticationEvent {
            owner_identity: Arc::clone(&self.owner_identity),
            binding_identity: self.binding_identity.clone(),
            principal: event.principal.clone(),
            intent: event.intent.clone(),
            source_identity: event.source_identity.clone(),
            timeline_identity: event.timeline_identity.clone(),
            policy: event.policy,
            issued_at_nanoseconds: event.issued_at_nanoseconds,
            expires_at_nanoseconds: event.expires_at_nanoseconds,
            observed_at_nanoseconds: now,
            principal_valid_until: principal.valid_until(),
            revoked: Arc::clone(&event.revoked),
            marker: PhantomData,
        })
    }

    pub const fn policy(&self) -> WorthQueryAuthenticationEventPolicy {
        self.policy
    }

    fn check_event(
        &self,
        event: &WorthQueryAuthenticationEvent<Schema>,
    ) -> Result<(), WorthQueryAuthenticationEventDenial> {
        if !Arc::ptr_eq(&self.owner_identity, &event.owner_identity) {
            return Err(WorthQueryAuthenticationEventDenial::WrongOwner);
        }
        if self.binding_identity != event.binding_identity {
            return Err(WorthQueryAuthenticationEventDenial::WrongSchemaBinding);
        }
        Ok(())
    }

    fn check_principal(
        &self,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    ) -> Result<(), WorthQueryAuthenticationEventDenial> {
        if self.binding_identity != *principal.binding_identity() {
            return Err(WorthQueryAuthenticationEventDenial::WrongSchemaBinding);
        }
        if principal.valid_until() <= Instant::now() {
            return Err(WorthQueryAuthenticationEventDenial::PrincipalExpired);
        }
        Ok(())
    }

    fn state(&self) -> std::sync::MutexGuard<'_, AuthenticationEventState<Clock, Source>> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn reserve_challenge(
        &self,
        principal: &WorthQueryExternalPrincipalIdentity,
    ) -> Result<
        AuthenticationEventReservation<'_, Clock, Source>,
        WorthQueryAuthenticationEventDenial,
    > {
        let mut state = self.state();
        let now = state.clock.observe()?;
        state.discard_expired(now);
        if state.events.len() + state.pending.len() >= self.maximum_events.get() {
            return Err(WorthQueryAuthenticationEventDenial::CapacityExceeded);
        }
        let identity = state.next_identity()?;
        state.pending.insert(identity, principal.clone());
        Ok(AuthenticationEventReservation {
            state: &self.state,
            identity,
            active: true,
        })
    }
}

fn check_interruption(
    scope: &WorthQueryRequestScope,
) -> Result<(), WorthQueryAuthenticationEventDenial> {
    match scope.interruption() {
        None => Ok(()),
        Some(WorthQueryRequestInterruption::Cancelled) => {
            Err(WorthQueryAuthenticationEventDenial::Cancelled)
        }
        Some(WorthQueryRequestInterruption::DeadlineExceeded) => {
            Err(WorthQueryAuthenticationEventDenial::DeadlineExceeded)
        }
    }
}
