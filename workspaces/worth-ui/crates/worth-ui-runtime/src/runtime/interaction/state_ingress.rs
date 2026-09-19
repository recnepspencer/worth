#[path = "state_ingress/draft_phase.rs"]
mod draft_phase;
#[path = "state_ingress/emission.rs"]
mod emission;
#[path = "state_ingress/gesture_phase.rs"]
mod gesture_phase;
#[path = "state_ingress/pointer_admission.rs"]
mod pointer_admission;
#[path = "state_ingress/presence_phase.rs"]
mod presence_phase;
#[path = "state_ingress/receipt.rs"]
mod receipt;
#[path = "state_ingress/report.rs"]
mod report;

use crate::runtime::WorthUiActiveApplicationGenerationIdentity;

use crate::runtime::interaction::UiInteractionBatchReceipt;

impl super::UiInteractionRuntimeState {
    pub(crate) fn ingest(
        &mut self,
        batch: crate::facade::observation_report::UiValidatedHostObservationBatch,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        generation: &WorthUiActiveApplicationGenerationIdentity,
    ) -> UiInteractionBatchReceipt {
        let core = batch.canonical_core();
        let mut receipt = receipt::UiInteractionBatchReceiptBuilder::default();
        for validated in batch.reports() {
            receipt.record(report::process(
                self,
                core,
                validated.report(),
                mounted,
                generation,
            ));
        }
        receipt.finish(batch, self)
    }
}
