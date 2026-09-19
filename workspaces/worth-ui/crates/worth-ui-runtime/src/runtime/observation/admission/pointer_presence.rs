use super::super::progress::UiObservationProgress;
use super::super::turn::{
    UiAdmittedObservation, UiAdmittedObservationPayload, UiAdmittedObservationSeal,
    UiObservationAdmissionDenial, UiObservationAdmissionReceipt, UiObservationTurn,
};
use super::super::UiObservationFamily;

impl UiObservationTurn<'_> {
    pub fn admit_pointer_presence_transition(
        &mut self,
        transition: crate::runtime::interaction::UiPointerPresenceTargetTransition,
    ) -> Result<UiObservationAdmissionReceipt, UiObservationAdmissionDenial> {
        let owner_order = transition.owner_revision();
        let progress = UiObservationProgress::pointer_presence(transition.pointer(), owner_order);
        self.admit(UiAdmittedObservation::seal(UiAdmittedObservationSeal {
            family: UiObservationFamily::PointerPresenceTarget,
            owner_order,
            retained_bytes: std::mem::size_of_val(&transition),
            session: self.session,
            source_basis: self.source_basis,
            progress: Some(progress),
            payload: UiAdmittedObservationPayload::PointerPresence(transition),
        }))
    }
}
