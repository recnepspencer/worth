use super::{
    UiAdmittedObservation, UiAdmittedObservationSet, UiObservationAdmissionDenial,
    UiObservationTurn, UiObservationTurnCloseAuthority, UiPreparedObservationProgressCommit,
};

impl UiObservationTurn<'_> {
    pub fn seal(self) -> Result<UiAdmittedObservationSet, UiObservationAdmissionDenial> {
        self.seal_with_time(None)
    }

    /// Close an as-of host-time observation, including a stationary pointer turn.
    /// Native sessions sample their installed clock instead of this caller reading.
    pub fn seal_at_host_time(
        self,
        time: worth_ui_host_contract::UiHostObservationTimeBasis,
    ) -> Result<UiAdmittedObservationSet, UiObservationAdmissionDenial> {
        self.seal_with_time(Some(time))
    }

    fn seal_with_time(
        mut self,
        time: Option<worth_ui_host_contract::UiHostObservationTimeBasis>,
    ) -> Result<UiAdmittedObservationSet, UiObservationAdmissionDenial> {
        self.validate_close(time.is_some())?;
        self.order_observations();
        self.runtime.observation.finish_committed(
            self.observations
                .iter()
                .filter_map(UiAdmittedObservation::progress),
        );
        Ok(self.seal_set(time))
    }

    pub(crate) fn prepare_seal(
        mut self,
    ) -> Result<
        (
            UiAdmittedObservationSet,
            UiPreparedObservationProgressCommit,
        ),
        UiObservationAdmissionDenial,
    > {
        self.validate_close(false)?;
        self.order_observations();
        let progress = self
            .observations
            .iter()
            .filter_map(UiAdmittedObservation::progress)
            .cloned()
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Ok((
            self.seal_set(None),
            UiPreparedObservationProgressCommit { progress },
        ))
    }

    fn validate_close(&self, supplied_time: bool) -> Result<(), UiObservationAdmissionDenial> {
        if self.poisoned {
            return Err(UiObservationAdmissionDenial::PoisonedTurn);
        }
        if self.observations.is_empty()
            && !(self
                .pointer_close
                .as_ref()
                .is_some_and(|input| supplied_time || input.has_clock()))
        {
            return Err(UiObservationAdmissionDenial::EmptyTurn);
        }
        Ok(())
    }

    fn order_observations(&mut self) {
        self.observations.sort_by_key(|observation| {
            (
                observation.family().definition().framework_rank(),
                observation.owner_order(),
            )
        });
    }

    fn seal_set(
        &mut self,
        time: Option<worth_ui_host_contract::UiHostObservationTimeBasis>,
    ) -> UiAdmittedObservationSet {
        let lease = self
            .runtime
            .observation
            .retain_set(self.observations.len(), self.retained_bytes);
        let observations = std::mem::take(&mut self.observations).into_boxed_slice();
        let appearance_owner_snapshot = self.seal_appearance_owner_snapshot();
        let authority = UiObservationTurnCloseAuthority { _private: () };
        let pointer_snapshot = self
            .pointer_close
            .take()
            .map(|input| input.seal(&authority, self.identity, self.source_basis, time));
        UiAdmittedObservationSet::seal(
            self.identity,
            self.session,
            self.source_basis,
            observations,
            self.retained_bytes,
            appearance_owner_snapshot,
            pointer_snapshot,
            lease,
        )
    }

    fn seal_appearance_owner_snapshot(
        &mut self,
    ) -> Option<crate::runtime::appearance::UiAppearanceOwnerSnapshot> {
        let authority = UiObservationTurnCloseAuthority { _private: () };
        self.appearance_close
            .take()
            .map(|input| input.seal(&authority, self.identity, self.session, self.source_basis))
    }
}
