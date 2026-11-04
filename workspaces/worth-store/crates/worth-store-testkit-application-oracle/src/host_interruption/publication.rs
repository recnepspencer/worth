//! Interrupt only the installed receiver's actual transitions.
use crate::host_observation::{
    requests::{self, MutationOutcome},
    OracleRuntime,
};

pub fn abandon_after_owner(runtime: &OracleRuntime, key: u64, value: u64) -> MutationOutcome {
    runtime
        .runtime()
        .world_operation_control_for_test()
        .panic_before_product_compare_once_for_test();
    // The production application boundary owns unwind isolation and its typed
    // outcome. An escaped panic must fail the court, never become a fake fate.
    requests::set(runtime, Some(key), value).expect("the genuine keyed mutation entry exists")
}
pub fn settlement_failure(runtime: &OracleRuntime, key: u64, value: u64) -> MutationOutcome {
    let _hold = runtime
        .runtime()
        .hold_post_commit_snapshot_failure_for_test()
        .expect("one actual snapshot settlement fault is armed");
    requests::set(runtime, Some(key), value).expect("the genuine keyed mutation entry exists")
}
fn handles(
    runtime: &OracleRuntime,
) -> Vec<worth_runtime_world::facade::ProductUnpublishedRecoveryHandle> {
    let mut cursor = None;
    let mut handles = Vec::new();
    loop {
        let page = runtime
            .runtime()
            .product_publication_recovery_page(
                cursor.as_ref(),
                std::num::NonZeroUsize::new(64).unwrap(),
            )
            .expect("actual recovery catalog");
        handles.extend(page.rows().iter().map(|r| r.handle().clone()));
        cursor = page.next_after().cloned();
        if cursor.is_none() {
            break;
        }
    }
    handles
}
pub fn continue_custody(
    runtime: &OracleRuntime,
) -> Result<
    Vec<worth_runtime_world::facade::RecoveryContinuationContract>,
    worth_runtime_world::facade::RuntimeWorldRecoveryDenial,
> {
    handles(runtime)
        .iter()
        .map(|h| {
            runtime
                .runtime()
                .readmit_product_publication_recovery(h)?
                .continue_owner_settlement()
        })
        .collect()
}
pub fn release_custody(
    runtime: &OracleRuntime,
) -> Result<usize, worth_runtime_world::facade::RuntimeWorldRecoveryDenial> {
    let handles = handles(runtime);
    for handle in &handles {
        let recovery = runtime
            .runtime()
            .readmit_product_publication_recovery(handle)?;
        runtime
            .runtime()
            .release_product_publication_recovery(recovery, 0)
            .unwrap_or_else(|failure| panic!("original custody release: {failure:?}"));
    }
    Ok(handles.len())
}
