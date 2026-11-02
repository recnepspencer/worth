//! Return source selection and planning owners before terminal progression.
use crate::entry::{PhysicalRecoveryOpenRequest, PhysicalRecoveryOutcome};
use crate::progression::{
    AdmittedPhysicalRecovery, PlannedPhysicalRecovery, ReopenedPhysicalRecovery,
    SelectedPhysicalRecovery,
};

#[inline(never)]
pub(crate) fn recover(
    request: PhysicalRecoveryOpenRequest,
    yieldpoint: Option<worth_store::physical_runtime::PhysicalRecoveryProcessYieldpoint>,
) -> PhysicalRecoveryOutcome {
    let selected = match select_source(request, yieldpoint) {
        Ok(selected) => selected,
        Err(outcome) => return outcome,
    };
    recover_selected(selected)
}

#[inline(never)]
fn recover_selected(selected: SelectedPhysicalRecovery) -> PhysicalRecoveryOutcome {
    let planned = match selected.plan() {
        Ok(planned) => planned,
        Err(outcome) => return outcome,
    };
    recover_planned(planned)
}

#[inline(never)]
fn recover_planned(planned: PlannedPhysicalRecovery) -> PhysicalRecoveryOutcome {
    match progress(planned) {
        Ok(reopened) => reopened.finish(),
        Err(outcome) => outcome,
    }
}

#[inline(never)]
fn select_source(
    request: PhysicalRecoveryOpenRequest,
    yieldpoint: Option<worth_store::physical_runtime::PhysicalRecoveryProcessYieldpoint>,
) -> Result<SelectedPhysicalRecovery, PhysicalRecoveryOutcome> {
    let admitted = match match yieldpoint {
        Some(yieldpoint) => request.admit_with_process_yieldpoint(yieldpoint),
        None => request.admit(),
    } {
        Ok(admitted) => admitted,
        Err(refusal) => return Err(PhysicalRecoveryOutcome::Refused(Box::new(refusal))),
    };
    discover_source(admitted)
}

#[inline(never)]
fn discover_source(
    admitted: AdmittedPhysicalRecovery,
) -> Result<SelectedPhysicalRecovery, PhysicalRecoveryOutcome> {
    let discovered = match admitted.discover() {
        Ok(discovered) => discovered,
        Err(outcome) => return Err(outcome),
    };
    discovered.select()
}

#[inline(never)]
fn progress(
    planned: PlannedPhysicalRecovery,
) -> Result<ReopenedPhysicalRecovery, PhysicalRecoveryOutcome> {
    let staged = match planned.stage() {
        Ok(staged) => staged,
        Err(outcome) => return Err(outcome),
    };
    let published = match staged.publish() {
        Ok(published) => published,
        Err(outcome) => return Err(outcome),
    };
    published.reopen()
}
