#[derive(Clone, Copy)]
struct PlatformPulseIntentEvidenceEntry {
    attempt_generation: u64,
    idempotency_session: u64,
    idempotency_lineage: u64,
    reference: worth_ui::facade::inspection::UiIntentEvidenceReference,
    mounted_instance: worth_ui::facade::app::UiMountedInstanceIdentity,
}

pub(in crate::native_application) struct PlatformPulseIntentEvidenceIndex {
    slots: [Option<PlatformPulseIntentEvidenceEntry>;
        worth_ui::facade::intent::UI_INTENT_MAXIMUM_APPLICATION_ATTEMPTS],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PlatformPulseIntentEvidenceIndexDenial {
    MissingReference,
    SlotOutOfRange,
    SlotOccupied,
}

impl PlatformPulseIntentEvidenceIndex {
    pub(in crate::native_application) const fn new() -> Self {
        Self {
            slots: [None; worth_ui::facade::intent::UI_INTENT_MAXIMUM_APPLICATION_ATTEMPTS],
        }
    }

    pub(super) fn retain(
        &mut self,
        dispatch: worth_ui::facade::intent::UiIntentExecutionDispatchReceipt,
        mounted_instance: worth_ui::facade::app::UiMountedInstanceIdentity,
    ) -> Result<(), PlatformPulseIntentEvidenceIndexDenial> {
        let reference = dispatch
            .evidence_reference()
            .ok_or(PlatformPulseIntentEvidenceIndexDenial::MissingReference)?;
        let slot = self
            .slots
            .get_mut(usize::from(dispatch.attempt().slot()))
            .ok_or(PlatformPulseIntentEvidenceIndexDenial::SlotOutOfRange)?;
        if slot.is_some() {
            return Err(PlatformPulseIntentEvidenceIndexDenial::SlotOccupied);
        }
        *slot = Some(PlatformPulseIntentEvidenceEntry {
            attempt_generation: dispatch.attempt().generation(),
            idempotency_session: dispatch.idempotency().session(),
            idempotency_lineage: dispatch.idempotency().lineage(),
            reference,
            mounted_instance,
        });
        Ok(())
    }

    pub(super) fn reference_for_product(
        &self,
        product: worth_ui_platform_pulse::intent::PlatformPulseActionAttemptReference,
    ) -> Option<worth_ui::facade::inspection::UiIntentEvidenceReference> {
        let entry = self
            .slots
            .get(usize::from(product.attempt_slot()))
            .copied()
            .flatten()?;
        (entry.attempt_generation == product.attempt_generation()
            && entry.idempotency_session == product.idempotency_session()
            && entry.idempotency_lineage == product.idempotency_lineage())
        .then_some(entry.reference)
    }

    pub(super) fn target_for_execution(
        &self,
        attempt: worth_ui::facade::intent::UiIntentExecutionAttemptIdentity,
        idempotency: worth_ui::facade::intent::UiIntentExecutionIdempotencyIdentity,
    ) -> Option<worth_ui::facade::app::UiMountedInstanceIdentity> {
        let entry = self.slots.get(usize::from(attempt.slot()))?.as_ref()?;
        (entry.attempt_generation == attempt.generation()
            && entry.idempotency_session == idempotency.session()
            && entry.idempotency_lineage == idempotency.lineage())
        .then_some(entry.mounted_instance)
    }

    pub(super) fn retire_execution(
        &mut self,
        attempt: worth_ui::facade::intent::UiIntentExecutionAttemptIdentity,
        idempotency: worth_ui::facade::intent::UiIntentExecutionIdempotencyIdentity,
    ) -> Option<worth_ui::facade::inspection::UiIntentEvidenceReference> {
        let slot = self.slots.get_mut(usize::from(attempt.slot()))?;
        let entry = slot.as_ref()?;
        if entry.attempt_generation != attempt.generation()
            || entry.idempotency_session != idempotency.session()
            || entry.idempotency_lineage != idempotency.lineage()
        {
            return None;
        }
        slot.take().map(|entry| entry.reference)
    }
}
