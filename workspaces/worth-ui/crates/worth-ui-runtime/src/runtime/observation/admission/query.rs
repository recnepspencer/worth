use super::super::progress::UiObservationProgress;
use super::super::turn::{
    UiAdmittedObservation, UiAdmittedObservationPayload, UiAdmittedObservationSeal,
    UiAdmittedQueryObservation, UiObservationAdmissionDenial, UiObservationAdmissionReceipt,
    UiObservationTurn, UiQueryObservationAdmissionStop,
};
use super::super::UiObservationFamily;

impl UiObservationTurn<'_> {
    pub fn admit_query(
        &mut self,
        consequence: worth_ui_query_binding::WorthUiCollectionChangeConsequence,
    ) -> Result<UiObservationAdmissionReceipt, UiQueryObservationAdmissionStop> {
        let retained_bytes = std::mem::size_of::<
            worth_ui_query_binding::WorthUiValidatedCollectionChangeObservation,
        >();
        self.can_admit(UiObservationFamily::Query, retained_bytes)
            .map_err(|denial| UiQueryObservationAdmissionStop::Observation(self.reject(denial)))?;
        let observation = match self
            .runtime
            .validate_operation_live_change_observation(consequence)
        {
            Ok(observation) => observation,
            Err(stop) => {
                self.poison();
                return Err(UiQueryObservationAdmissionStop::Query(Box::new(stop)));
            }
        };
        let owner_order = observation.change_order();
        let progress = UiObservationProgress::query(observation.source(), owner_order);
        self.admit(UiAdmittedObservation::seal(UiAdmittedObservationSeal {
            family: UiObservationFamily::Query,
            owner_order,
            retained_bytes,
            session: self.session,
            source_basis: self.source_basis,
            progress: Some(progress),
            payload: UiAdmittedObservationPayload::Query(
                UiAdmittedQueryObservation::OperationLive(observation),
            ),
        }))
        .map_err(UiQueryObservationAdmissionStop::Observation)
    }

    pub fn admit_projection_query(
        &mut self,
        observation: worth_ui_query_binding::UiProjectionObservation,
    ) -> Result<UiObservationAdmissionReceipt, UiObservationAdmissionDenial> {
        self.validate_projection_source(&observation)?;
        let retained_bytes = observation.retained_bytes();
        let owner_order = observation.owner_order();
        let progress =
            UiObservationProgress::query_projection(observation.projection_identity(), owner_order);
        self.admit(UiAdmittedObservation::seal(UiAdmittedObservationSeal {
            family: UiObservationFamily::Query,
            owner_order,
            retained_bytes,
            session: self.session,
            source_basis: self.source_basis,
            progress: Some(progress),
            payload: UiAdmittedObservationPayload::Query(UiAdmittedQueryObservation::Projection(
                observation,
            )),
        }))
    }

    pub(super) fn validate_projection_source(
        &mut self,
        observation: &worth_ui_query_binding::UiProjectionObservation,
    ) -> Result<(), UiObservationAdmissionDenial> {
        if let worth_ui_query_binding::UiProjectionObservation::ApplicationScalar(scalar) =
            observation
        {
            let registration = self
                .runtime
                .query_binding
                .application_scalar_projection_registration(scalar.projection_identity());
            if !registration.is_some_and(|registration| registration.admits(scalar.fact())) {
                return Err(self.reject(UiObservationAdmissionDenial::ForeignQueryProjection));
            }
        }
        Ok(())
    }
}
