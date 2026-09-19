use super::super::progress::UiObservationProgress;
use super::super::turn::{
    UiAdmittedObservation, UiAdmittedObservationPayload, UiAdmittedObservationSeal,
    UiAdmittedQueryObservation, UiObservationAdmissionDenial, UiObservationAdmissionReceipt,
    UiObservationTurn,
};
use super::super::UiObservationFamily;

pub(crate) struct UiIntentConsequenceObservationBatch {
    posture: Option<(
        crate::mounting::UiIntentPostureObservation,
        crate::mounting::UiIntentPostureCommit,
    )>,
    query: Option<worth_ui_query_binding::WorthUiCollectionChangeConsequence>,
    projection: Option<worth_ui_query_binding::UiProjectionObservation>,
}

pub(crate) struct UiIntentConsequenceObservationAdmissionReceipt {
    admitted: Box<[UiObservationAdmissionReceipt]>,
    posture_commit: Option<crate::mounting::UiIntentPostureCommit>,
}

pub(crate) enum UiIntentConsequenceObservationAdmissionReason {
    Observation(UiObservationAdmissionDenial),
    Query(worth_ui_query_binding::WorthUiCollectionChangeAdmissionDenial),
}

pub(crate) struct UiIntentConsequenceObservationAdmissionStop {
    reason: UiIntentConsequenceObservationAdmissionReason,
    batch: Box<UiIntentConsequenceObservationBatch>,
}

impl UiIntentConsequenceObservationBatch {
    pub(crate) const fn new(
        posture: Option<(
            crate::mounting::UiIntentPostureObservation,
            crate::mounting::UiIntentPostureCommit,
        )>,
        query: Option<worth_ui_query_binding::WorthUiCollectionChangeConsequence>,
        projection: Option<worth_ui_query_binding::UiProjectionObservation>,
    ) -> Self {
        Self {
            posture,
            query,
            projection,
        }
    }

    pub(crate) const fn is_empty(&self) -> bool {
        self.posture.is_none() && self.query.is_none() && self.projection.is_none()
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        Option<(
            crate::mounting::UiIntentPostureObservation,
            crate::mounting::UiIntentPostureCommit,
        )>,
        Option<worth_ui_query_binding::WorthUiCollectionChangeConsequence>,
        Option<worth_ui_query_binding::UiProjectionObservation>,
    ) {
        (self.posture, self.query, self.projection)
    }
}

impl UiObservationTurn<'_> {
    pub(crate) fn admit_intent_consequence_batch(
        &mut self,
        batch: UiIntentConsequenceObservationBatch,
    ) -> Result<
        UiIntentConsequenceObservationAdmissionReceipt,
        UiIntentConsequenceObservationAdmissionStop,
    > {
        let (posture, query, projection) = batch.into_parts();
        if let Some(observation) = projection.as_ref() {
            if let Err(denial) = self.validate_projection_source(observation) {
                return Err(UiIntentConsequenceObservationAdmissionStop {
                    reason: UiIntentConsequenceObservationAdmissionReason::Observation(denial),
                    batch: Box::new(UiIntentConsequenceObservationBatch::new(
                        posture, query, projection,
                    )),
                });
            }
        }
        let (posture_observation, posture_commit) = unzip_posture(posture);
        let query_observation = match query {
            Some(consequence) => match self
                .runtime
                .validate_operation_live_change_observation(consequence)
            {
                Ok(observation) => Some(observation),
                Err(stop) => {
                    self.poison();
                    return Err(UiIntentConsequenceObservationAdmissionStop::query(
                        posture_observation.zip(posture_commit),
                        projection,
                        stop,
                    ));
                }
            },
            None => None,
        };
        let observations = seal_batch(
            posture_observation,
            query_observation,
            projection,
            self.session,
            self.source_basis,
        );
        match self.admit_batch_recoverable(observations) {
            Ok(admitted) => Ok(UiIntentConsequenceObservationAdmissionReceipt {
                admitted,
                posture_commit,
            }),
            Err(stop) => {
                let (denial, observations) = stop.into_parts();
                let (posture, query, projection) = recover_batch(observations, posture_commit);
                Err(UiIntentConsequenceObservationAdmissionStop {
                    reason: UiIntentConsequenceObservationAdmissionReason::Observation(denial),
                    batch: Box::new(UiIntentConsequenceObservationBatch::new(
                        posture, query, projection,
                    )),
                })
            }
        }
    }
}

fn unzip_posture(
    posture: Option<(
        crate::mounting::UiIntentPostureObservation,
        crate::mounting::UiIntentPostureCommit,
    )>,
) -> (
    Option<crate::mounting::UiIntentPostureObservation>,
    Option<crate::mounting::UiIntentPostureCommit>,
) {
    match posture {
        Some((observation, commit)) => (Some(observation), Some(commit)),
        None => (None, None),
    }
}

fn seal_batch(
    posture: Option<crate::mounting::UiIntentPostureObservation>,
    query: Option<worth_ui_query_binding::WorthUiValidatedCollectionChangeObservation>,
    projection: Option<worth_ui_query_binding::UiProjectionObservation>,
    session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    source_basis: u64,
) -> Vec<UiAdmittedObservation> {
    let mut observations = Vec::with_capacity(2);
    if let Some(query) = query {
        let owner_order = query.change_order();
        let progress = UiObservationProgress::query(query.source(), owner_order);
        observations.push(UiAdmittedObservation::seal(UiAdmittedObservationSeal {
            family: UiObservationFamily::Query,
            owner_order,
            retained_bytes: std::mem::size_of_val(&query),
            session,
            source_basis,
            progress: Some(progress),
            payload: UiAdmittedObservationPayload::Query(
                UiAdmittedQueryObservation::OperationLive(query),
            ),
        }));
    }
    if let Some(projection) = projection {
        let owner_order = projection.owner_order();
        let progress =
            UiObservationProgress::query_projection(projection.projection_identity(), owner_order);
        observations.push(UiAdmittedObservation::seal(UiAdmittedObservationSeal {
            family: UiObservationFamily::Query,
            owner_order,
            retained_bytes: projection.retained_bytes(),
            session,
            source_basis,
            progress: Some(progress),
            payload: UiAdmittedObservationPayload::Query(UiAdmittedQueryObservation::Projection(
                projection,
            )),
        }));
    }
    if let Some(posture) = posture {
        let owner_order = posture.owner_order();
        observations.push(UiAdmittedObservation::seal(UiAdmittedObservationSeal {
            family: UiObservationFamily::IntentPosture,
            owner_order,
            retained_bytes: posture.retained_bytes(),
            session,
            source_basis,
            progress: Some(UiObservationProgress::intent_posture(owner_order)),
            payload: UiAdmittedObservationPayload::IntentPosture(posture),
        }));
    }
    observations
}

fn recover_batch(
    observations: Vec<UiAdmittedObservation>,
    posture_commit: Option<crate::mounting::UiIntentPostureCommit>,
) -> (
    Option<(
        crate::mounting::UiIntentPostureObservation,
        crate::mounting::UiIntentPostureCommit,
    )>,
    Option<worth_ui_query_binding::WorthUiCollectionChangeConsequence>,
    Option<worth_ui_query_binding::UiProjectionObservation>,
) {
    let mut posture = None;
    let mut query = None;
    let mut projection = None;
    for observation in observations {
        match observation.into_payload() {
            UiAdmittedObservationPayload::IntentPosture(observation) => {
                posture = Some(observation);
            }
            UiAdmittedObservationPayload::Query(UiAdmittedQueryObservation::OperationLive(
                observation,
            )) => query = Some(observation.into_consequence()),
            UiAdmittedObservationPayload::Query(UiAdmittedQueryObservation::Projection(
                observation,
            )) => projection = Some(observation),
            _ => unreachable!("intent consequence batch contains only owned consequence families"),
        }
    }
    (posture.zip(posture_commit), query, projection)
}

impl UiIntentConsequenceObservationAdmissionReceipt {
    pub(crate) const fn admitted(&self) -> &[UiObservationAdmissionReceipt] {
        &self.admitted
    }

    pub(crate) fn into_posture_commit(self) -> Option<crate::mounting::UiIntentPostureCommit> {
        self.posture_commit
    }
}

impl UiIntentConsequenceObservationAdmissionStop {
    fn query(
        posture: Option<(
            crate::mounting::UiIntentPostureObservation,
            crate::mounting::UiIntentPostureCommit,
        )>,
        projection: Option<worth_ui_query_binding::UiProjectionObservation>,
        stop: worth_ui_query_binding::WorthUiCollectionChangeAdmissionStop,
    ) -> Self {
        let denial = stop.denial();
        let consequence = stop.into_consequence();
        Self {
            reason: UiIntentConsequenceObservationAdmissionReason::Query(denial),
            batch: Box::new(UiIntentConsequenceObservationBatch::new(
                posture,
                Some(consequence),
                projection,
            )),
        }
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        UiIntentConsequenceObservationAdmissionReason,
        Box<UiIntentConsequenceObservationBatch>,
    ) {
        (self.reason, self.batch)
    }
}
