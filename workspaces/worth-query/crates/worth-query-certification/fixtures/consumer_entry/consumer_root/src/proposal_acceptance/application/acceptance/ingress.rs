//! Fresh disclosure and native key resolution precede every protected slot or result lookup.
use super::*;
impl<'host> Model<'host> {
    pub fn accept(
        &mut self,
        storage: &Storage<'host>,
        caller: &Caller<'_>,
        loaded: &LoadedProposal,
        key: u64,
    ) -> Result<Event, Failure> {
        let rows = storage
            .admit_attempt(self, caller, loaded)
            .map_err(Failure::Admission)?;
        let resolution = storage
            .resolve_claim(caller, &loaded.input(key), key)
            .map_err(Failure::Resolution)?;
        if matches!(resolution, Resolution::IntentDrift) {
            return Err(Failure::IntentDrift);
        }
        if let Some(pending) = &self.pending {
            if pending.key != key {
                return Err(Failure::Busy);
            }
            // Before registration there is no native intent binding to call drift.
            if !pending.matches(loaded, key) {
                return Err(Failure::OriginalContinuationRetained);
            }
            let attempt = self.pending.take().expect("exclusive pending attempt");
            let (pending, event) = attempt.advance(self, storage, caller);
            self.pending = pending;
            return event;
        }
        if matches!(resolution, Resolution::AlreadyCommitted(_)) {
            let claim = storage
                .read_claim(caller, &loaded.input(key))
                .map_err(Failure::ClaimRead)?;
            return match storage::results::select_result(&rows, loaded, &claim) {
                Ok(result) => Ok(Event::Cached(result)),
                Err(storage::results::ResultReadFailure::Missing) => {
                    Err(Failure::ContinuationUnavailable)
                }
                Err(cause) => Err(Failure::ResultRead(cause)),
            };
        }
        candidate::preparation::eligible(self, caller, loaded.proposal())
            .map_err(Failure::Eligibility)?;
        let operation = storage
            .claim_operation(caller, &loaded.input(key), key)
            .map_err(Failure::Construction)?;
        self.pending = Some(Attempt {
            loaded: loaded.clone(),
            key,
            phase: Phase::Claim(operation),
        });
        Ok(Event::Progress)
    }
    pub fn inspect_attempt(
        &self,
        storage: &Storage<'host>,
        caller: &Caller<'_>,
        loaded: &LoadedProposal,
        key: u64,
    ) -> Result<Option<View<'_, 'host>>, Failure> {
        storage
            .admit_attempt(self, caller, loaded)
            .map_err(Failure::Admission)?;
        if matches!(
            storage
                .resolve_claim(caller, &loaded.input(key), key)
                .map_err(Failure::Resolution)?,
            Resolution::IntentDrift
        ) {
            return Err(Failure::IntentDrift);
        }
        match &self.pending {
            Some(attempt) if attempt.matches(loaded, key) => Ok(Some(attempt.view())),
            Some(_) => Err(Failure::Busy),
            None => Ok(None),
        }
    }
}
impl<'host> Attempt<'host> {
    fn advance(
        self,
        model: &mut Model<'host>,
        storage: &Storage<'host>,
        caller: &Caller<'_>,
    ) -> (Option<Self>, Result<Event, Failure>) {
        let Self { loaded, key, phase } = self;
        let (phase, event) = match phase {
            phase @ (Phase::Claim(_) | Phase::ClaimCompleted(_)) => {
                registration::advance(phase, storage, caller, &loaded, key)
            }
            phase @ (Phase::Registered(_) | Phase::Staging { .. } | Phase::Ready { .. }) => {
                staging::advance(phase, model, storage, caller, &loaded, key)
            }
            phase @ (Phase::Adopted { .. }
            | Phase::PreviousClose { .. }
            | Phase::RetiredPrevious { .. }) => {
                delivery::advance(phase, model, storage, caller, &loaded, key)
            }
            phase => (Some(phase), Ok(Event::Stopped)),
        };
        (phase.map(|phase| Self { loaded, key, phase }), event)
    }
}
