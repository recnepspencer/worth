//! Return source selection and planning owners before terminal progression.
use crate::entry::{PhysicalRecoveryOpenRequest, PhysicalRecoveryOutcome};
use crate::progression::{PlannedPhysicalRecovery, SelectedPhysicalRecovery};

#[inline(never)]
pub(crate) fn recover(
    request: PhysicalRecoveryOpenRequest,
    yieldpoint: Option<worth_store::physical_runtime::PhysicalRecoveryProcessYieldpoint>,
) -> PhysicalRecoveryOutcome {
    let selected = match select_source(request, yieldpoint) {
        Ok(selected) => selected,
        Err(outcome) => return outcome,
    };
    let planned = match selected.plan() {
        Ok(planned) => planned,
        Err(outcome) => return outcome,
    };
    progress(planned)
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
    let discovered = match admitted.discover() {
        Ok(discovered) => discovered,
        Err(outcome) => return Err(outcome),
    };
    let selected = match discovered.select() {
        Ok(selected) => selected,
        Err(outcome) => return Err(outcome),
    };
    Ok(selected)
}

#[inline(never)]
fn progress(planned: PlannedPhysicalRecovery) -> PhysicalRecoveryOutcome {
    let staged = match planned.stage() {
        Ok(staged) => staged,
        Err(outcome) => return outcome,
    };
    let published = match staged.publish() {
        Ok(published) => published,
        Err(outcome) => return outcome,
    };
    match published.reopen() {
        Ok(reopened) => reopened.finish(),
        Err(outcome) => outcome,
    }
}
