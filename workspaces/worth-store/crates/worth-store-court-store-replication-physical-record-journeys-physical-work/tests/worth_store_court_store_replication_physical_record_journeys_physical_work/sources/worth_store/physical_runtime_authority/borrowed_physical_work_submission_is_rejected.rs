use worth_store::physical_runtime::{PhysicalReadSubmission, PhysicalReadWorkRequest};

fn submit_borrowed(
    submission: &PhysicalReadSubmission,
    request: &PhysicalReadWorkRequest,
) {
    let _ = submission.submit(request);
}

fn main() {}
