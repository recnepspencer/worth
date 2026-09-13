use crate::entry::{PhysicalRecoveryOpenRequest, PhysicalRecoveryOutcome};

pub(crate) fn recover(
    request: PhysicalRecoveryOpenRequest,
    yieldpoint: Option<worth_store::physical_runtime::PhysicalRecoveryProcessYieldpoint>,
) -> PhysicalRecoveryOutcome {
    let admitted = match match yieldpoint {
        Some(yieldpoint) => request.admit_with_process_yieldpoint(yieldpoint),
        None => request.admit(),
    } {
        Ok(admitted) => admitted,
        Err(refusal) => return PhysicalRecoveryOutcome::Refused(refusal),
    };
    let discovered = match admitted.discover() {
        Ok(discovered) => discovered,
        Err(outcome) => return outcome,
    };
    let selected = match discovered.select() {
        Ok(selected) => selected,
        Err(outcome) => return outcome,
    };
    let planned = match selected.plan() {
        Ok(planned) => planned,
        Err(outcome) => return outcome,
    };
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
