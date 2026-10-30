use worth_store::physical_runtime::{
    AdmittedPhysicalWork, PhysicalReadSubmission, PhysicalReadWorkRequest,
    PhysicalWorkPreEffectDenial, PhysicalWorkReadiness, PhysicalWorkSemanticBasis,
    PhysicalWorkSubmissionOutcome, ServingPhysicalRuntime,
};

#[expect(
    dead_code,
    reason = "Compile-only positive consumer witness; its signature and body are checked without fabricating admitted runtime inputs."
)]
fn submit_owned_request(
    submission: &PhysicalReadSubmission,
    request: PhysicalReadWorkRequest,
) -> PhysicalWorkSubmissionOutcome {
    submission.submit(request)
}

#[expect(
    dead_code,
    reason = "Compile-only positive consumer witness; its signature and body are checked without fabricating admitted runtime inputs."
)]
fn accept_typed_semantic_basis(basis: PhysicalWorkSemanticBasis) -> PhysicalWorkSemanticBasis {
    basis
}

#[expect(
    dead_code,
    reason = "Compile-only positive consumer witness; its signature and body are checked without fabricating admitted runtime inputs."
)]
fn request_admitted_work(
    serving: &ServingPhysicalRuntime,
    admitted: AdmittedPhysicalWork,
) -> Result<PhysicalWorkReadiness, PhysicalWorkPreEffectDenial> {
    serving.request_physical_work(admitted)
}

pub(crate) fn main() {}
