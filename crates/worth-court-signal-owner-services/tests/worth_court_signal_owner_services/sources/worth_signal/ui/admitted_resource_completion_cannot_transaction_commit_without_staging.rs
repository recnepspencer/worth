use worth_signal::facade::{AdmittedResourceCompletion, SignalTransaction};

fn admitted_completion() -> AdmittedResourceCompletion {
    loop {}
}

fn commit_admitted_completion(tx: &mut SignalTransaction<'_, (), (), (), (), ()>) {
    let _ = tx.commit_staged_resource_completion(admitted_completion());
}

fn main() {}
