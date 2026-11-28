#[cfg(test)]
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use super::super::super::component_obligation::RetentionControlSurface;
#[cfg(test)]
use super::super::super::component_obligation::{
    ComponentBasisReleaseOutcome, ComponentBasisReleaseReceipt, RetentionReleaseDenial,
    RetentionReleaseFailure,
};
use super::super::super::obligation_transfer::RetentionTransferDenial;
use super::super::super::unique_component_pin::ComponentBasisPinClaim;
use super::super::super::ComponentClaimKind;
#[cfg(test)]
use super::PinEntry;
use super::{FlightCompletion, PinFlight, RuntimeWorldRetentionOwner};

impl<D, I, T> RetentionControlSurface for RuntimeWorldRetentionOwner<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn try_retire_shared_pair(
        &self,
        dependency: ComponentClaimKind,
        relational: &ComponentBasisPinClaim,
        signal: &ComponentBasisPinClaim,
    ) -> Result<(), ()> {
        let mut state = self.state.try_lock().map_err(|_| ())?;
        if relational.key == signal.key {
            return Err(());
        }
        for claim in [relational, signal] {
            let entry = state.entries.get(&claim.key).ok_or(())?;
            if claim.owner != state.owner_identity
                || claim.dependency != dependency
                || entry.lease_identity != claim.lease_identity
                || entry.counts.get(claim.dependency) == 0
                || entry.counts.total() <= 1
            {
                return Err(());
            }
        }
        for claim in [relational, signal] {
            state
                .entries
                .get_mut(&claim.key)
                .expect("validated pair")
                .counts
                .decrement(claim.dependency)
                .expect("validated dependency");
        }
        state.active_obligations -= 2;
        #[cfg(test)]
        {
            state.costs.dependency_releases = state.costs.dependency_releases.saturating_add(2);
        }
        Ok(())
    }

    fn try_take_history_pair_for_close(
        &self,
        relational: &ComponentBasisPinClaim,
        signal: &ComponentBasisPinClaim,
    ) -> Result<crate::retention::component_obligation::RetiringHistoryComponents, ()> {
        self.try_take_checkpoint_pair(relational, signal)
    }

    fn fork_history_pair(
        &self,
        relational: &ComponentBasisPinClaim,
        signal: &ComponentBasisPinClaim,
    ) -> Result<(ComponentBasisPinClaim, ComponentBasisPinClaim), RetentionTransferDenial> {
        self.fork_history_claims(relational, signal)
    }

    #[cfg(test)]
    fn transfer_claim(
        &self,
        mut claim: ComponentBasisPinClaim,
        target: ComponentClaimKind,
    ) -> Result<ComponentBasisPinClaim, (ComponentBasisPinClaim, RetentionTransferDenial)> {
        let expected = self.lock().owner_identity;
        if claim.owner != expected {
            return Err((claim, RetentionTransferDenial::ForeignOwner));
        }
        if claim.dependency == target {
            return Ok(claim);
        }
        let mut state = self.lock();
        let Some(entry) = state.entries.get_mut(&claim.key) else {
            return Err((claim, RetentionTransferDenial::UnknownPin));
        };
        if entry.lease_identity != claim.lease_identity || entry.counts.get(claim.dependency) == 0 {
            return Err((claim, RetentionTransferDenial::UnknownPin));
        }
        if entry.counts.get(target) == usize::MAX {
            return Err((claim, RetentionTransferDenial::DependencyCountExhausted));
        }
        entry
            .counts
            .decrement(claim.dependency)
            .expect("validated source count");
        entry
            .counts
            .increment(target)
            .expect("validated target count");
        claim.dependency = target;
        Ok(claim)
    }

    fn transfer_pair(
        &self,
        relational: &mut ComponentBasisPinClaim,
        signal: &mut ComponentBasisPinClaim,
        target: ComponentClaimKind,
    ) -> Result<(), RetentionTransferDenial> {
        if relational.key == signal.key {
            return Err(RetentionTransferDenial::BasisMismatch);
        }
        let mut state = self.lock();
        if relational.owner != state.owner_identity || signal.owner != state.owner_identity {
            return Err(RetentionTransferDenial::ForeignOwner);
        }
        let relational_valid = state.entries.get(&relational.key).is_some_and(|entry| {
            entry.lease_identity == relational.lease_identity
                && entry.counts.get(relational.dependency) > 0
        });
        let signal_valid = state.entries.get(&signal.key).is_some_and(|entry| {
            entry.lease_identity == signal.lease_identity && entry.counts.get(signal.dependency) > 0
        });
        if !relational_valid || !signal_valid {
            return Err(RetentionTransferDenial::UnknownPin);
        }
        let relational_target_full = relational.dependency != target
            && state
                .entries
                .get(&relational.key)
                .is_some_and(|entry| entry.counts.get(target) == usize::MAX);
        let signal_target_full = signal.dependency != target
            && state
                .entries
                .get(&signal.key)
                .is_some_and(|entry| entry.counts.get(target) == usize::MAX);
        if relational_target_full || signal_target_full {
            return Err(RetentionTransferDenial::DependencyCountExhausted);
        }
        for claim in [&relational, &signal] {
            if claim.dependency == target {
                continue;
            }
            let entry = state
                .entries
                .get_mut(&claim.key)
                .expect("validated pair entry");
            entry
                .counts
                .decrement(claim.dependency)
                .expect("validated source count");
            entry
                .counts
                .increment(target)
                .expect("validated target count");
        }
        relational.dependency = target;
        signal.dependency = target;
        Ok(())
    }

    #[cfg(test)]
    fn release_claim(
        &self,
        claim: ComponentBasisPinClaim,
    ) -> Result<ComponentBasisReleaseReceipt, RetentionReleaseFailure> {
        let key = claim.key.clone();
        let mut state = self.lock();
        let expected = state.owner_identity;
        if claim.owner != expected {
            let actual = claim.owner;
            return Err(RetentionReleaseFailure {
                claim,
                denial: RetentionReleaseDenial::ForeignOwner { expected, actual },
            });
        }
        let Some(entry) = state.entries.get(&key) else {
            return Err(RetentionReleaseFailure {
                claim,
                denial: RetentionReleaseDenial::UnknownPin,
            });
        };
        if entry.lease_identity != claim.lease_identity || entry.counts.get(claim.dependency) == 0 {
            return Err(RetentionReleaseFailure {
                claim,
                denial: RetentionReleaseDenial::UnknownPin,
            });
        }
        if entry.counts.total() > 1 {
            let entry = state.entries.get_mut(&key).expect("validated entry");
            entry
                .counts
                .decrement(claim.dependency)
                .expect("validated source count");
            state.active_obligations -= 1;
            #[cfg(test)]
            {
                state.costs.dependency_releases = state.costs.dependency_releases.saturating_add(1);
            }
            return Ok(ComponentBasisReleaseReceipt::owner_issued(
                key,
                claim.lease_identity,
                ComponentBasisReleaseOutcome::SharedOwnerLease,
            ));
        }
        let PinEntry {
            owner_lease: lease,
            counts,
            lease_identity,
        } = state.entries.remove(&key).expect("validated final entry");
        let flight = Arc::new(PinFlight::new(true));
        state.flights.insert(key.clone(), Arc::clone(&flight));
        #[cfg(test)]
        {
            state.costs.owner_release_contacts =
                state.costs.owner_release_contacts.saturating_add(1);
        }
        drop(state);

        let result = catch_unwind(AssertUnwindSafe(|| self.release_component(lease)));
        let mut state = self.lock();
        state.flights.remove(&key);
        match result {
            Ok(Ok(())) => {
                state.active_obligations -= 1;
                #[cfg(test)]
                {
                    state.costs.dependency_releases =
                        state.costs.dependency_releases.saturating_add(1);
                }
                flight.finish(FlightCompletion::Released);
                Ok(ComponentBasisReleaseReceipt::owner_issued(
                    key,
                    claim.lease_identity,
                    ComponentBasisReleaseOutcome::OwnerReleased,
                ))
            }
            Ok(Err(failure)) => {
                state.entries.insert(
                    key,
                    PinEntry {
                        owner_lease: failure.lease,
                        counts,
                        lease_identity,
                    },
                );
                flight.finish(FlightCompletion::Released);
                Err(RetentionReleaseFailure {
                    claim,
                    denial: failure.reason,
                })
            }
            Err(_) => {
                state.active_obligations -= 1;
                #[cfg(test)]
                {
                    state.costs.dependency_releases =
                        state.costs.dependency_releases.saturating_add(1);
                }
                flight.finish(FlightCompletion::Released);
                Ok(ComponentBasisReleaseReceipt::owner_issued(
                    key,
                    claim.lease_identity,
                    ComponentBasisReleaseOutcome::OwnerOperationPanicked,
                ))
            }
        }
    }

    fn abandon_claim(&self, claim: ComponentBasisPinClaim) {
        let key = claim.key.clone();
        let (lease, flight) = {
            let mut state = self.lock();
            if claim.owner != state.owner_identity {
                return;
            }
            let Some(entry) = state.entries.get(&key) else {
                return;
            };
            if entry.lease_identity != claim.lease_identity
                || entry.counts.get(claim.dependency) == 0
            {
                return;
            }
            if entry.counts.total() > 1 {
                let entry = state.entries.get_mut(&key).expect("validated entry");
                entry
                    .counts
                    .decrement(claim.dependency)
                    .expect("validated source count");
                state.active_obligations -= 1;
                #[cfg(test)]
                {
                    state.costs.dependency_releases =
                        state.costs.dependency_releases.saturating_add(1);
                }
                return;
            }
            let lease = state
                .entries
                .remove(&key)
                .expect("validated final entry")
                .owner_lease;
            let flight = Arc::new(PinFlight::new(true));
            state.flights.insert(key.clone(), Arc::clone(&flight));
            state.active_obligations -= 1;
            #[cfg(test)]
            {
                state.costs.dependency_releases = state.costs.dependency_releases.saturating_add(1);
            }
            #[cfg(test)]
            {
                state.costs.owner_drop_releases = state.costs.owner_drop_releases.saturating_add(1);
            }
            (lease, flight)
        };
        drop(lease);
        let mut state = self.lock();
        state.flights.remove(&key);
        flight.finish(FlightCompletion::Released);
    }
}
