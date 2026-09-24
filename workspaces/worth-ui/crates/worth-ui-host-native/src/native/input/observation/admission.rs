use super::{
    UiNativeInputObservationEventFamily, UiNativeInputObservationState,
    UiNativeInputObservationStop,
};

impl UiNativeInputObservationState {
    pub(in crate::native::input) fn admit_input(
        &mut self,
        family: UiNativeInputObservationEventFamily,
    ) -> bool {
        if self.completed.is_none() {
            self.record_stop(UiNativeInputObservationStop::NoPresentationBasis);
            return false;
        }
        if self.profile.profile().is_none() {
            self.record_stop(UiNativeInputObservationStop::MissingEventProfile);
            return false;
        }
        if self.profile.awaits_completion() {
            self.record_stop(UiNativeInputObservationStop::StalePresentationAffinity);
            return false;
        }
        if !requires_recipient(family) {
            return true;
        }
        let Some(recipient) = self.input_recipient else {
            self.record_stop(UiNativeInputObservationStop::MissingInputRecipientAffinity);
            return false;
        };
        let Some((_, host_session, presentation)) = self.completed else {
            unreachable!("completed presentation was admitted above")
        };
        if recipient.host_session() != host_session || recipient.binding() != presentation.binding()
        {
            self.record_stop(UiNativeInputObservationStop::StaleInputRecipientAffinity);
            return false;
        }
        true
    }
}

fn requires_recipient(family: UiNativeInputObservationEventFamily) -> bool {
    matches!(
        family,
        UiNativeInputObservationEventFamily::Text | UiNativeInputObservationEventFamily::Ime
    )
}
