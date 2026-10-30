use worth_signal::facade::{SignalTransaction, StagedDeniedResourceCompletionEffect};

fn commit_denied(
    tx: &mut SignalTransaction<'_, (), (), (), (), ()>,
    staged: StagedDeniedResourceCompletionEffect,
) {
    let _ = tx.commit_staged_resource_completion(staged);
}

fn main() {}
