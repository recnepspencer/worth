use worth_store::physical_runtime::{
    AdmittedPhysicalWork, PhysicalReadSubmission, PhysicalReadWorkRequest,
    PhysicalWorkPreEffectDenial, PhysicalWorkReadiness, PhysicalWorkSemanticBasis,
    PhysicalWorkSubmissionOutcome, ServingPhysicalRuntime,
};

fn submit_owned_request(
    submission: &PhysicalReadSubmission,
    request: PhysicalReadWorkRequest,
) -> PhysicalWorkSubmissionOutcome {
    submission.submit(request)
}

fn accept_typed_semantic_basis(basis: PhysicalWorkSemanticBasis) -> PhysicalWorkSemanticBasis {
    basis
}

fn request_admitted_work(
    serving: &ServingPhysicalRuntime,
    admitted: AdmittedPhysicalWork,
) -> Result<PhysicalWorkReadiness, PhysicalWorkPreEffectDenial> {
    serving.request_physical_work(admitted)
}

fn main() {}
