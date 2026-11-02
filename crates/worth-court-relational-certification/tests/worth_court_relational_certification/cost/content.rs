mod bounds;
mod controls;
mod directory;
mod frames;
mod invoice;
mod leased;
mod stimulus;

use crate::content::{
    fixture::grid,
    original,
    transaction::{fixture, FORMAT},
};
use worth_execution::{CancellationSource, ExecutionRequest};
use worth_relational::facade::mvcc::RelationalTransactionIntent;

#[test]
fn content_staging_growth_is_bounded_by_persistent_paths_and_cancellation_polls_first() {
    fn sample(count: usize) -> u64 {
        let cancel = CancellationSource::new();
        let original = original(&cancel);
        let execution = ExecutionRequest::serial(&original);
        let (runtime, _) = fixture();
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        let mut transaction = runtime
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        let (result, report) = worth_execution::ExecutionWorkCeiling::new(None)
            .run_with_checkpoint(None, Some(&original), |_| {
                for _ in 0..count {
                    transaction.admit_content(&execution, FORMAT, &grid())?;
                }
                Ok::<_, worth_relational::facade::content::RelationalContentDenial>(())
            })
            .unwrap();
        result.unwrap();
        cancel.cancel();
        assert!(matches!(
            transaction.admit_content(&execution, FORMAT, &grid()),
            Err(worth_relational::facade::content::RelationalContentDenial::Execution(_))
        ));
        let work = report.charged_work();
        drop(transaction);
        assert_eq!(
            original.memory().unwrap().available_memory_bytes(),
            Some(64 * 1024 * 1024)
        );
        work
    }
    let small = sample(256);
    let large = sample(512);
    assert!(small > 0);
    assert!(large > small);
    assert!(
        large < small * 3,
        "doubling staged frames must not quadruple work: {small} -> {large}"
    );
    eprintln!("content staging charged Work: 256={small}, 512={large}");
}
