//! Vacant-entry reservation and atomic completion for application attempts.

use super::super::WorthQueryPrimaryGraphProvider;
use super::{
    WorthQueryApplicationAttemptEntry, WorthQueryApplicationAttemptLookupKey,
    WorthQueryApplicationAttemptReservation, WorthQueryApplicationAttemptState,
    WorthQueryPrimaryGraphApplicationAttemptStore,
};
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptAffinity, WorthQueryPrimaryGraphApplicationAttempt,
};

pub(in crate::domain_computation::primary_graph) struct WorthQueryApplicationAttemptReservationOwner<
    'provider,
> {
    provider: &'provider WorthQueryPrimaryGraphProvider,
    reservation: Option<WorthQueryApplicationAttemptReservation>,
}

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn reserve_application_attempt(
        &self,
        affinity: &WorthQueryApplicationAttemptAffinity,
    ) -> Result<WorthQueryApplicationAttemptReservationOwner<'_>, &'static str> {
        self.application_attempt_work.observe_attempt_registration();
        let reservation = self
            .attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .reserve(affinity)?;
        Ok(WorthQueryApplicationAttemptReservationOwner {
            provider: self,
            reservation: Some(reservation),
        })
    }
}

impl WorthQueryApplicationAttemptReservationOwner<'_> {
    pub(in crate::domain_computation::primary_graph) fn complete(
        mut self,
        attempt: WorthQueryPrimaryGraphApplicationAttempt,
    ) -> Result<(), &'static str> {
        let reservation = self
            .reservation
            .as_ref()
            .ok_or("application attempt registration reservation was already consumed")?;
        {
            self.provider
                .attempts
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .complete_registration(reservation, attempt)?;
        }
        self.reservation.take();
        Ok(())
    }
}

impl Drop for WorthQueryApplicationAttemptReservationOwner<'_> {
    fn drop(&mut self) {
        let Some(reservation) = self.reservation.take() else {
            return;
        };
        let _ = self
            .provider
            .attempts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .cancel_registration(reservation);
    }
}

impl WorthQueryPrimaryGraphApplicationAttemptStore {
    pub(in crate::domain_computation::primary_graph::provider) fn reserve(
        &mut self,
        affinity: &WorthQueryApplicationAttemptAffinity,
    ) -> Result<WorthQueryApplicationAttemptReservation, &'static str> {
        let key = WorthQueryApplicationAttemptLookupKey::from_affinity(affinity);
        let identity = self
            .next_registration
            .checked_add(1)
            .ok_or("application attempt reservation identity space is exhausted")?;
        match self.attempts.entry(key) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(WorthQueryApplicationAttemptEntry::Reserved {
                    identity,
                    terminal: affinity.provider_session().clone(),
                });
                self.next_registration = identity;
                Ok(WorthQueryApplicationAttemptReservation { key, identity })
            }
            std::collections::btree_map::Entry::Occupied(_) => {
                Err("provider session already owns an application attempt")
            }
        }
    }

    pub(in crate::domain_computation::primary_graph::provider) fn complete_registration(
        &mut self,
        reservation: &WorthQueryApplicationAttemptReservation,
        attempt: WorthQueryPrimaryGraphApplicationAttempt,
    ) -> Result<(), &'static str> {
        let key = WorthQueryApplicationAttemptLookupKey::from_affinity(attempt.affinity());
        if key != reservation.key {
            return Err("application attempt does not match its registration reservation");
        }
        let Some(entry) = self.attempts.get_mut(&key) else {
            return Err("application attempt registration reservation was lost");
        };
        if !matches!(
            entry,
            WorthQueryApplicationAttemptEntry::Reserved { identity, .. }
                if *identity == reservation.identity
        ) {
            return Err("application attempt registration reservation was replaced");
        }
        *entry = WorthQueryApplicationAttemptEntry::Registered {
            identity: reservation.identity,
            state: WorthQueryApplicationAttemptState::registered(attempt),
        };
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph::provider) fn cancel_registration(
        &mut self,
        reservation: WorthQueryApplicationAttemptReservation,
    ) -> bool {
        let matches = self.attempts.get(&reservation.key).is_some_and(|entry| {
            matches!(
                entry,
                WorthQueryApplicationAttemptEntry::Reserved { identity, .. }
                    if *identity == reservation.identity
            )
        });
        if matches {
            self.attempts.remove(&reservation.key);
        }
        matches
    }
}
