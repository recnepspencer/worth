//! Each recovered original is continued or released through World's recovery
//! port; an undecided write leaves the original to the next process.
use super::model::Original;
use super::*;

impl Model {
    /// Continue each effect-bearing original (unless `release_all`) and
    /// release the rest, then compare the live state with the model.
    pub fn decide_all(
        &mut self,
        fixture: &Fixture,
        request: &ExecutionRequest<'_, '_>,
        release_all: bool,
    ) {
        let page = fixture
            .world
            .inspection_port()
            .recovery_page(None, maximum())
            .unwrap();
        let recovery = fixture.world.recovery_port();
        for row in page.rows() {
            let effects = recovery.inspect_recovered_effects(row.handle()).unwrap();
            let original = self.take(identities(&effects));
            let continued = self.continued.contains(&original.identities);
            if release_all && continued {
                let outcome = release(&recovery, request, effects);
                assert!(
                    !matches!(outcome, WorldRecoveredReleaseOutcome::Released(_)),
                    "a durable Continued disposition never becomes a release"
                );
                drop(outcome);
                self.originals.push(original);
            } else if release_all || original.receipt.is_none() {
                let outcome = release(&recovery, request, effects);
                assert!(
                    matches!(outcome, WorldRecoveredReleaseOutcome::Released(_)),
                    "a recovered original releases: {outcome:?}"
                );
                drop(outcome);
                assert!(
                    recovery.inspect_recovered_effects(row.handle()).is_err(),
                    "a duplicate request cannot reclaim a released original"
                );
                self.stale |= original.receipt.is_some();
                self.released.push(original);
            } else {
                let outcome = advance(&recovery, request, effects);
                assert!(
                    matches!(outcome, WorldRecoveredContinuationOutcome::Published { .. }),
                    "a recovered Native effect continues once: {outcome:?}"
                );
                self.continued
                    .retain(|identity| *identity != original.identities);
                self.generation += 1;
            }
        }
        drop(page);
        self.compare(fixture, request);
    }

    /// Decide the first original while its decision write reports an
    /// indeterminate fate, then die. The model follows what actually landed.
    pub fn undecided(
        &mut self,
        fixture: &Fixture,
        request: &ExecutionRequest<'_, '_>,
        landed: bool,
    ) {
        let page = fixture
            .world
            .inspection_port()
            .recovery_page(None, maximum())
            .unwrap();
        let recovery = fixture.world.recovery_port();
        let effects = recovery
            .inspect_recovered_effects(page.rows()[0].handle())
            .unwrap();
        let identities = identities(&effects);
        let continued = effects.native_receipt().is_some();
        fixture.recorder.undecide_next(landed);
        if continued {
            let outcome = advance(&recovery, request, effects);
            assert!(
                matches!(outcome, WorldRecoveredContinuationOutcome::Indeterminate(_)),
                "an unknown occurrence fate stays undecided: {outcome:?}"
            );
        } else {
            let outcome = release(&recovery, request, effects);
            assert!(
                matches!(outcome, WorldRecoveredReleaseOutcome::Indeterminate(_)),
                "an unknown release fate is not proof of no effect: {outcome:?}"
            );
        }
        fixture.recorder.crash();
        drop(page);
        // A continuation's first durable write is its Continued disposition;
        // the occurrence follows only in a later continuation.
        if landed && continued {
            self.continued.push(identities);
        } else if landed {
            let original: Original = self.take(identities);
            self.released.push(original);
        }
    }
}

fn identities(effects: &WorldRecoveredUnpublishedEffects) -> [u64; 3] {
    [
        effects.attempt_identity().ordinal(),
        effects.reserved_commit_identity().ordinal(),
        effects.identity().ordinal(),
    ]
}

fn advance(
    recovery: &RuntimeWorldRecoveryPort,
    request: &ExecutionRequest<'_, '_>,
    effects: WorldRecoveredUnpublishedEffects,
) -> WorldRecoveredContinuationOutcome {
    let mut outcome = recovery.continue_recovered_effects(request, effects);
    for _ in 0..64 {
        let WorldRecoveredContinuationOutcome::Pending(effects) = outcome else {
            break;
        };
        outcome = recovery.continue_recovered_effects(request, effects);
    }
    outcome
}

fn release(
    recovery: &RuntimeWorldRecoveryPort,
    request: &ExecutionRequest<'_, '_>,
    effects: WorldRecoveredUnpublishedEffects,
) -> WorldRecoveredReleaseOutcome {
    let mut outcome = recovery.release_recovered_effects(request, effects);
    for _ in 0..64 {
        let WorldRecoveredReleaseOutcome::Pending(effects) = outcome else {
            break;
        };
        outcome = recovery.release_recovered_effects(request, effects);
    }
    outcome
}
