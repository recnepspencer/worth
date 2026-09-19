use worth_ui_host_contract::{UiHostObservationCanonicalCore, UiHostObservationReport};

use crate::runtime::interaction::draft::{UiDraftProcessingOutcome, UiDraftRuntimeState};

pub(super) fn process(
    draft: &mut UiDraftRuntimeState,
    pointer_presence_denied: bool,
    core: UiHostObservationCanonicalCore,
    report: &UiHostObservationReport,
    mounted: &crate::mounting::WorthUiMountedSessionState,
    generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
) -> Vec<UiDraftProcessingOutcome> {
    if pointer_presence_denied {
        Vec::new()
    } else {
        draft.process_report(core, report, mounted, generation)
    }
}
