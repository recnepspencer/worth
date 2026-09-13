use super::UiInteractionRuntimeState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiInteractionApplicationRebindDenial {
    OwnerChanged,
    GenerationExhausted,
}

pub(crate) struct UiPreparedInteractionApplicationRebind {
    pointer: crate::runtime::interaction::gesture::UiPreparedPointerGestureCancellation,
    predecessor_presence_revision: Option<u64>,
    pointer_presence: Option<crate::runtime::interaction::UiPointerPresenceOwner>,
    predecessor_generation: worth_ui_host_contract::UiHostApplicationGeneration,
    successor_generation: worth_ui_host_contract::UiHostApplicationGeneration,
}

impl UiPreparedInteractionApplicationRebind {
    pub(crate) fn pressed(
        &self,
    ) -> crate::runtime::interaction::gesture::UiPressedAppearanceOwnerSnapshot {
        self.pointer.appearance_snapshot()
    }

    pub(crate) fn pointer_presence(
        &self,
    ) -> Option<crate::runtime::interaction::UiPointerPresenceAppearanceOwnerSnapshot> {
        self.pointer_presence
            .as_ref()
            .map(|owner| owner.appearance_snapshot())
    }
}

impl UiInteractionRuntimeState {
    pub(crate) fn prepare_application_rebind(
        &self,
        pointer_presence_enabled: bool,
        pressed_enabled: bool,
    ) -> Result<UiPreparedInteractionApplicationRebind, UiInteractionApplicationRebindDenial> {
        let successor_generation = self
            .application_generation
            .get()
            .checked_add(1)
            .and_then(worth_ui_host_contract::UiHostApplicationGeneration::new)
            .ok_or(UiInteractionApplicationRebindDenial::GenerationExhausted)?;
        Ok(UiPreparedInteractionApplicationRebind {
            pointer: self.pointer.prepare_cancel_all_with_appearance(
                crate::runtime::interaction::UiPointerGestureStopReason::ApplicationRebound,
                pressed_enabled,
            ),
            predecessor_presence_revision: self
                .pointer_presence
                .as_ref()
                .map(|owner| owner.revision()),
            pointer_presence: pointer_presence_enabled.then(|| {
                self.pointer_presence.as_ref().map_or_else(
                    || {
                        crate::runtime::interaction::UiPointerPresenceOwner::new(
                            self.pointer_presence_capacity,
                        )
                    },
                    |owner| owner.prepare_cleared(),
                )
            }),
            predecessor_generation: self.application_generation,
            successor_generation,
        })
    }

    pub(crate) fn validate_application_rebind(
        &self,
        prepared: &UiPreparedInteractionApplicationRebind,
    ) -> Result<(), UiInteractionApplicationRebindDenial> {
        if self.application_generation != prepared.predecessor_generation
            || !self.pointer.admits_prepared_cancellation(&prepared.pointer)
            || self.pointer_presence.as_ref().map(|owner| owner.revision())
                != prepared.predecessor_presence_revision
        {
            return Err(UiInteractionApplicationRebindDenial::OwnerChanged);
        }
        Ok(())
    }

    pub(crate) fn commit_application_rebind(
        &mut self,
        prepared: UiPreparedInteractionApplicationRebind,
    ) -> crate::runtime::interaction::UiInteractionLifecycleSettlementReceipt {
        self.validate_application_rebind(&prepared)
            .expect("accepted replacement retains prepared interaction owners");
        let pointer = self.pointer.commit_prepared_cancellation(prepared.pointer);
        self.pointer_presence = prepared.pointer_presence;
        let draft = self
            .draft
            .cancel_all(crate::runtime::interaction::UiLocalInputStopReason::ApplicationRebound);
        self.application_generation = prepared.successor_generation;
        self.settlement(pointer, draft)
    }
}
