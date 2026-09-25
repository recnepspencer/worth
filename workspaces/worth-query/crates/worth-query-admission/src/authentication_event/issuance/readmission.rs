use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;

use super::*;

impl<Schema, Clock, Source, Verifier>
    WorthQueryInstalledAuthenticationEventOwner<Schema, Clock, Source, Verifier>
where
    Clock: WorthQueryNamedClock,
    Source: WorthQueryNamedClockSource<Clock>,
    Verifier: WorthQueryAuthenticationEventVerifier,
{
    /// Re-admits a consumed event at its exact installed owner.
    pub fn readmit_consumed_for_signing(
        &self,
        consumed: WorthQueryConsumedAuthenticationEvent<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        expected_intent: &WorthQueryAuthenticationEventIntent,
        scope: &WorthQueryRequestScope,
    ) -> Result<WorthQueryConsumedAuthenticationEvent<Schema>, WorthQueryAuthenticationEventDenial>
    {
        self.check_principal(principal)?;
        check_interruption(scope)?;
        if consumed.principal != *principal.identity() {
            return Err(WorthQueryAuthenticationEventDenial::PrincipalMismatch);
        }
        if consumed.intent != *expected_intent {
            return Err(WorthQueryAuthenticationEventDenial::IntentMismatch);
        }
        self.readmit_for_publication(consumed)
    }

    pub fn revoke(
        &self,
        event: &WorthQueryAuthenticationEvent<Schema>,
    ) -> Result<(), WorthQueryAuthenticationEventDenial> {
        self.check_event(event)?;
        event.revoked.store(true, Ordering::Release);
        let mut state = self.state();
        if let Some(retained) = state.remove_event(event.event_id) {
            retained.revoked.store(true, Ordering::Release);
        }
        Ok(())
    }

    pub(super) fn readmit_for_publication(
        &self,
        consumed: WorthQueryConsumedAuthenticationEvent<Schema>,
    ) -> Result<WorthQueryConsumedAuthenticationEvent<Schema>, WorthQueryAuthenticationEventDenial>
    {
        if !Arc::ptr_eq(&self.owner_identity, &consumed.owner_identity) {
            return Err(WorthQueryAuthenticationEventDenial::WrongOwner);
        }
        if self.binding_identity != consumed.binding_identity {
            return Err(WorthQueryAuthenticationEventDenial::WrongSchemaBinding);
        }
        if consumed.principal_valid_until <= Instant::now() {
            return Err(WorthQueryAuthenticationEventDenial::PrincipalExpired);
        }
        if consumed.revoked.load(Ordering::Acquire) {
            return Err(WorthQueryAuthenticationEventDenial::EventRevoked);
        }
        let mut state = self.state();
        if consumed.source_identity != *state.clock.source_identity()
            || consumed.timeline_identity != *state.clock.timeline_identity()
        {
            return Err(WorthQueryAuthenticationEventDenial::ClockIdentityChanged);
        }
        let now = state.clock.observe()?;
        if now >= consumed.expires_at_nanoseconds {
            return Err(WorthQueryAuthenticationEventDenial::EventExpired);
        }
        Ok(consumed)
    }

    /// Administrative revocation contacts only the finite active inventory;
    /// ordinary signing remains a direct event lookup.
    pub fn revoke_outstanding_for_principal(
        &self,
        principal: &WorthQueryExternalPrincipalIdentity,
    ) -> usize {
        let mut state = self.state();
        let matching: Vec<u64> = state
            .events
            .iter()
            .filter_map(|(identity, event)| (&event.principal == principal).then_some(*identity))
            .collect();
        for identity in &matching {
            if let Some(event) = state.remove_event(*identity) {
                event.revoked.store(true, Ordering::Release);
            }
        }
        let pending: Vec<u64> = state
            .pending
            .iter()
            .filter_map(|(identity, candidate)| (candidate == principal).then_some(*identity))
            .collect();
        state.invalidated_pending.extend(pending.iter().copied());
        matching.len() + pending.len()
    }
}
