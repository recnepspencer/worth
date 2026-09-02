use super::super::state::UiObservationOrderPosture;
use super::super::{UiObservationFamily, UiObservationProfile};
use super::{
    UiAdmittedObservation, UiAdmittedObservationSet, UiObservationAdmissionDenial,
    UiObservationAdmissionReceipt, UiObservationTurnDenial, UiObservationTurnIdentity,
};

mod close;

pub struct UiObservationTurn<'state> {
    pub(in crate::runtime::observation) runtime: &'state mut crate::runtime::WorthUiRuntime,
    identity: UiObservationTurnIdentity,
    pub(in crate::runtime::observation) session:
        crate::facade::WorthUiActiveApplicationSessionIdentity,
    pub(in crate::runtime::observation) source_basis: u64,
    profile: UiObservationProfile,
    observations: Vec<UiAdmittedObservation>,
    retained_bytes: usize,
    poisoned: bool,
    appearance_close: Option<super::UiAppearanceObservationCloseInput<'state>>,
}

pub(crate) struct UiObservationTurnCloseAuthority {
    _private: (),
}

#[cfg(test)]
impl UiObservationTurnCloseAuthority {
    pub(crate) const fn for_test() -> Self {
        Self { _private: () }
    }
}

pub(crate) struct UiPreparedObservationProgressCommit {
    progress: Box<[super::super::progress::UiObservationProgress]>,
}

impl<'state> UiObservationTurn<'state> {
    pub(super) fn new(
        runtime: &'state mut crate::runtime::WorthUiRuntime,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        source_basis: u64,
        identity: u64,
        profile: UiObservationProfile,
        appearance_close: Option<super::UiAppearanceObservationCloseInput<'state>>,
    ) -> Self {
        Self {
            runtime,
            identity: UiObservationTurnIdentity::issued_by_runtime(identity),
            session,
            source_basis,
            profile,
            observations: Vec::new(),
            retained_bytes: 0,
            poisoned: false,
            appearance_close,
        }
    }

    pub const fn identity(&self) -> UiObservationTurnIdentity {
        self.identity
    }

    pub fn resource_snapshot(&self) -> super::super::UiObservationResourceSnapshot {
        self.runtime.observation.resource_snapshot()
    }

    pub(in crate::runtime::observation) fn admit(
        &mut self,
        observation: UiAdmittedObservation,
    ) -> Result<UiObservationAdmissionReceipt, UiObservationAdmissionDenial> {
        if self.poisoned {
            return Err(UiObservationAdmissionDenial::PoisonedTurn);
        }
        let result = self.admit_unpoisoned(observation);
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }

    pub(in crate::runtime::observation) fn admit_batch(
        &mut self,
        observations: Vec<UiAdmittedObservation>,
    ) -> Result<Box<[UiObservationAdmissionReceipt]>, UiObservationAdmissionDenial> {
        self.admit_batch_recoverable(observations)
            .map_err(UiObservationBatchAdmissionStop::into_denial)
    }

    pub(in crate::runtime::observation) fn admit_batch_recoverable(
        &mut self,
        observations: Vec<UiAdmittedObservation>,
    ) -> Result<Box<[UiObservationAdmissionReceipt]>, UiObservationBatchAdmissionStop> {
        if self.poisoned {
            return Err(UiObservationBatchAdmissionStop::new(
                UiObservationAdmissionDenial::PoisonedTurn,
                observations,
            ));
        }
        if let Err(denial) = self.validate_batch(&observations) {
            self.poisoned = true;
            return Err(UiObservationBatchAdmissionStop::new(denial, observations));
        }
        Ok(self.commit_batch(observations))
    }

    pub(in crate::runtime::observation) fn reject(
        &mut self,
        denial: UiObservationAdmissionDenial,
    ) -> UiObservationAdmissionDenial {
        self.poisoned = true;
        denial
    }

    pub(in crate::runtime::observation) fn poison(&mut self) {
        self.poisoned = true;
    }

    fn admit_unpoisoned(
        &mut self,
        observation: UiAdmittedObservation,
    ) -> Result<UiObservationAdmissionReceipt, UiObservationAdmissionDenial> {
        if observation.session() != self.session {
            return Err(UiObservationAdmissionDenial::ForeignSession);
        }
        if observation.source_basis() != self.source_basis {
            return Err(UiObservationAdmissionDenial::ForeignSourceBasis);
        }
        if let Some(progress) = observation.progress() {
            match self.runtime.observation.order_posture(progress) {
                UiObservationOrderPosture::Fresh => {}
                UiObservationOrderPosture::Duplicate => {
                    return Err(UiObservationAdmissionDenial::DuplicateOwnerOrder)
                }
                UiObservationOrderPosture::Historical => {
                    return Err(UiObservationAdmissionDenial::HistoricalOwnerOrder)
                }
            }
        }
        self.can_admit(observation.family(), observation.retained_bytes())?;
        let receipt = UiObservationAdmissionReceipt::new(
            observation.family(),
            observation.owner_order(),
            observation.retained_bytes(),
        );
        self.push_admitted(observation);
        Ok(receipt)
    }

    fn validate_batch(
        &self,
        observations: &[UiAdmittedObservation],
    ) -> Result<(), UiObservationAdmissionDenial> {
        let mut families = std::collections::BTreeSet::new();
        let mut retained_bytes = 0usize;
        for observation in observations {
            if observation.session() != self.session {
                return Err(UiObservationAdmissionDenial::ForeignSession);
            }
            if observation.source_basis() != self.source_basis {
                return Err(UiObservationAdmissionDenial::ForeignSourceBasis);
            }
            if let Some(progress) = observation.progress() {
                match self.runtime.observation.order_posture(progress) {
                    UiObservationOrderPosture::Fresh => {}
                    UiObservationOrderPosture::Duplicate => {
                        return Err(UiObservationAdmissionDenial::DuplicateOwnerOrder)
                    }
                    UiObservationOrderPosture::Historical => {
                        return Err(UiObservationAdmissionDenial::HistoricalOwnerOrder)
                    }
                }
            }
            if !families.insert(observation.family())
                || self
                    .observations
                    .iter()
                    .any(|item| item.family() == observation.family())
            {
                return Err(UiObservationAdmissionDenial::DuplicateFamily);
            }
            retained_bytes = retained_bytes
                .checked_add(observation.retained_bytes())
                .ok_or(UiObservationAdmissionDenial::ByteCapacityExceeded)?;
        }
        if self
            .observations
            .len()
            .checked_add(observations.len())
            .is_none_or(|count| count > self.profile.admitted_per_turn())
        {
            return Err(UiObservationAdmissionDenial::TurnCapacityExceeded);
        }
        if self
            .retained_bytes
            .checked_add(retained_bytes)
            .is_none_or(|bytes| bytes > self.profile.retained_bytes_per_turn())
        {
            return Err(UiObservationAdmissionDenial::ByteCapacityExceeded);
        }
        Ok(())
    }

    fn commit_batch(
        &mut self,
        observations: Vec<UiAdmittedObservation>,
    ) -> Box<[UiObservationAdmissionReceipt]> {
        let receipts = observations
            .iter()
            .map(|observation| {
                UiObservationAdmissionReceipt::new(
                    observation.family(),
                    observation.owner_order(),
                    observation.retained_bytes(),
                )
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        for observation in observations {
            self.push_admitted(observation);
        }
        receipts
    }

    pub(in crate::runtime::observation) fn can_admit(
        &self,
        family: UiObservationFamily,
        retained_bytes: usize,
    ) -> Result<(), UiObservationAdmissionDenial> {
        if self.poisoned {
            return Err(UiObservationAdmissionDenial::PoisonedTurn);
        }
        if self.observations.iter().any(|item| item.family() == family) {
            return Err(UiObservationAdmissionDenial::DuplicateFamily);
        }
        if self.observations.len() == self.profile.admitted_per_turn() {
            return Err(UiObservationAdmissionDenial::TurnCapacityExceeded);
        }
        let total = self
            .retained_bytes
            .checked_add(retained_bytes)
            .ok_or(UiObservationAdmissionDenial::ByteCapacityExceeded)?;
        if total > self.profile.retained_bytes_per_turn() {
            return Err(UiObservationAdmissionDenial::ByteCapacityExceeded);
        }
        Ok(())
    }

    pub(in crate::runtime::observation) fn push_admitted(
        &mut self,
        observation: UiAdmittedObservation,
    ) {
        self.retained_bytes += observation.retained_bytes();
        self.observations.push(observation);
        self.runtime
            .observation
            .update_active_resources(self.observations.len(), self.retained_bytes);
    }
}

pub(in crate::runtime::observation) struct UiObservationBatchAdmissionStop {
    denial: UiObservationAdmissionDenial,
    observations: Vec<UiAdmittedObservation>,
}

impl UiObservationBatchAdmissionStop {
    fn new(denial: UiObservationAdmissionDenial, observations: Vec<UiAdmittedObservation>) -> Self {
        Self {
            denial,
            observations,
        }
    }

    fn into_denial(self) -> UiObservationAdmissionDenial {
        self.denial
    }

    pub(in crate::runtime::observation) fn into_parts(
        self,
    ) -> (UiObservationAdmissionDenial, Vec<UiAdmittedObservation>) {
        (self.denial, self.observations)
    }
}

impl Drop for UiObservationTurn<'_> {
    fn drop(&mut self) {
        self.runtime.observation.abandon();
    }
}

impl crate::runtime::WorthUiRuntime {
    #[allow(
        dead_code,
        reason = "runtime-only tests may begin a turn without facade owner capture"
    )]
    pub(crate) fn begin_observation_turn<'state>(
        &'state mut self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        source_basis: u64,
    ) -> Result<UiObservationTurn<'state>, UiObservationTurnDenial> {
        self.begin_observation_turn_with_appearance_close(session, source_basis, None)
    }

    pub(crate) fn begin_observation_turn_with_appearance_close<'state>(
        &'state mut self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        source_basis: u64,
        appearance_close: Option<super::UiAppearanceObservationCloseInput<'state>>,
    ) -> Result<UiObservationTurn<'state>, UiObservationTurnDenial> {
        let profile = self.change_profile.observation();
        let (identity, profile) = self.observation.begin(profile)?;
        Ok(UiObservationTurn::new(
            self,
            session,
            source_basis,
            identity,
            profile,
            appearance_close,
        ))
    }

    pub(in crate::runtime) fn commit_prepared_observation_progress(
        &mut self,
        commit: UiPreparedObservationProgressCommit,
    ) {
        debug_assert!(commit.progress.iter().all(|progress| matches!(
            self.observation.order_posture(progress),
            UiObservationOrderPosture::Fresh
        )));
        self.observation.finish_committed(commit.progress.iter());
    }
}
