use worth_ui::facade::app::{
    WorthUiActiveApplicationSession, WorthUiNativeApplicationShell,
    WorthUiNativeManagedSourceRebindOutcome, WorthUiNativeSourceRebindDenial,
};
use worth_ui::facade::inspection::{UiRebindDecisionLookup, UiRebindDecisionRecord};
use worth_ui::facade::observation::UiChangeClassificationOutcome;
use worth_ui::facade::rebind::{
    UiRebindExecutionPolicy, UiRebindPlanningDenial, UiRebindReceipt,
    UiSourceRebindRequest,
};
use worth_ui::facade::source::WorthUiSettledSourceSnapshot;

fn begin_settled_source_rebind(
    shell: &mut WorthUiNativeApplicationShell,
    snapshot: WorthUiSettledSourceSnapshot,
    now_tick: u64,
) -> Result<WorthUiNativeManagedSourceRebindOutcome, WorthUiNativeSourceRebindDenial> {
    let request = UiSourceRebindRequest::new(snapshot)
        .with_deadline(shell.rebind_deadline_at(now_tick.saturating_add(1)))
        .observed_at_tick(now_tick);
    shell.begin_source_rebind(request)
}

fn inspect_rebind_decision(receipt: &UiRebindReceipt) -> Option<UiRebindDecisionRecord> {
    let record = receipt.decision_record();
    match receipt.decision_index().lookup(record.key()) {
        UiRebindDecisionLookup::Found(exact) => Some(*exact),
        UiRebindDecisionLookup::Expired | UiRebindDecisionLookup::Unavailable => None,
    }
}

fn compile_owner_issued_change(
    session: &WorthUiActiveApplicationSession,
    outcome: UiChangeClassificationOutcome,
) -> Result<(), UiRebindPlanningDenial> {
    match outcome {
        UiChangeClassificationOutcome::Changed(change) => {
            let scope = session
                .resolve_affected_scope(change)
                .expect("owner-issued change resolves");
            let lifecycle = scope
                .resolve_identity_lifecycle()
                .expect("resolved scope advances one phase");
            session
                .compile_rebind_plan(lifecycle, UiRebindExecutionPolicy::ordinary())
                .map(|_| ())
        }
        UiChangeClassificationOutcome::EvidenceOnly(evidence) => session
            .compile_preservation_rebind(evidence, UiRebindExecutionPolicy::ordinary())
            .map(|_| ()),
        UiChangeClassificationOutcome::ObservedNoChange(_) => Ok(()),
    }
}

fn main() {
    let _ = (
        begin_settled_source_rebind,
        inspect_rebind_decision,
        compile_owner_issued_change,
    );
}
