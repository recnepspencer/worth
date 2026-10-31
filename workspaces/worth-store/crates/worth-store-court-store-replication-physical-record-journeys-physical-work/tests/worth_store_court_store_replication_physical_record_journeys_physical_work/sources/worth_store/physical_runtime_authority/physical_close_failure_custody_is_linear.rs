use worth_store::physical_runtime::PhysicalStoreCloseFailure;

fn clone_failure(failure: PhysicalStoreCloseFailure) {
    let _ = failure.clone();
}

fn extract_private_outcome(failure: PhysicalStoreCloseFailure) {
    let _ = failure.actual;
}

fn extract_twice(failure: PhysicalStoreCloseFailure) {
    let _ = failure.into_parts();
    let _ = failure.into_parts();
}

fn extract_borrowed(failure: &PhysicalStoreCloseFailure) {
    let _ = failure.into_parts();
}

fn main() {}
