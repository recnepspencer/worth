pub(super) struct WorthUiPreparedIntentConsequenceObservation {
    pub(super) set: crate::runtime::observation::UiAdmittedObservationSet,
    pub(super) progress: crate::runtime::observation::UiPreparedObservationProgressCommit,
    pub(super) posture: Option<crate::mounting::UiIntentPostureCommit>,
    pub(super) admitted_count: usize,
}

pub(super) struct WorthUiIntentConsequenceObservationStop {
    pub(super) reason: crate::runtime::intent_execution::UiIntentConsequenceStopReason,
    pub(super) batch: Box<crate::runtime::observation::UiIntentConsequenceObservationBatch>,
}

pub(super) fn prepare_intent_consequence_observation(
    session: &mut super::WorthUiActiveApplicationSession,
    batch: crate::runtime::observation::UiIntentConsequenceObservationBatch,
) -> Result<WorthUiPreparedIntentConsequenceObservation, WorthUiIntentConsequenceObservationStop> {
    let mut turn = match session.begin_observation_turn() {
        Ok(turn) => turn,
        Err(denial) => {
            return Err(WorthUiIntentConsequenceObservationStop {
                reason:
                    crate::runtime::intent_execution::UiIntentConsequenceStopReason::ObservationTurn(
                        denial,
                    ),
                batch: Box::new(batch),
            });
        }
    };
    let admission = match turn.admit_intent_consequence_batch(batch) {
        Ok(admission) => admission,
        Err(stop) => {
            drop(turn);
            let (reason, batch) = stop.into_parts();
            return Err(WorthUiIntentConsequenceObservationStop {
                reason: map_observation_stop(reason),
                batch,
            });
        }
    };
    let admitted_count = admission.admitted().len();
    let posture = admission.into_posture_commit();
    let (set, progress) = turn
        .prepare_seal()
        .expect("successful nonempty consequence admission prepares one deferred seal");
    Ok(WorthUiPreparedIntentConsequenceObservation {
        set,
        progress,
        posture,
        admitted_count,
    })
}

fn map_observation_stop(
    reason: crate::runtime::observation::UiIntentConsequenceObservationAdmissionReason,
) -> crate::runtime::intent_execution::UiIntentConsequenceStopReason {
    match reason {
        crate::runtime::observation::UiIntentConsequenceObservationAdmissionReason::Observation(
            denial,
        ) => crate::runtime::intent_execution::UiIntentConsequenceStopReason::ObservationAdmission(
            denial,
        ),
        crate::runtime::observation::UiIntentConsequenceObservationAdmissionReason::Query(
            denial,
        ) => {
            crate::runtime::intent_execution::UiIntentConsequenceStopReason::QueryAdmission(denial)
        }
    }
}
