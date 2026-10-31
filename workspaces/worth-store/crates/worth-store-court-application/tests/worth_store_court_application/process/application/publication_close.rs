//! Real durable performed custody prevents Home teardown until settlement.
use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_relational::facade::{
    mvcc::{RelationalPublicationOutcome, RelationalTransactionIntent},
    runtime::RelationalRuntimeCloseDenial,
};
use worth_store_application::{
    ApplicationHomeCloseDenial, ApplicationHomeCloseOutcome, ApplicationHomeClosePosture,
};

#[test]
fn performed_publication_custody_refuses_close_until_its_real_settlement() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let home = crate::durable_run::open(directory.path(), &execution).unwrap();
    let home_identity = home.home_identity();
    let runtime = home.relational().runtime();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    drop(basis);
    transaction
        .push_batch(
            crate::batches::create(0),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(&execution, transaction)
        .unwrap();
    let performed = match runtime
        .publication_port()
        .compare_and_publish(&execution, candidate)
    {
        RelationalPublicationOutcome::Performed(performed) => performed,
        outcome => panic!("a real uncontended durable publication performs: {outcome:?}"),
    };
    let identity = performed.canonical_commit().commit.commit_id;
    let port = runtime.settlement_port();
    let home = match home.close(&execution) {
        ApplicationHomeCloseOutcome::CloseRefused {
            reason:
                ApplicationHomeCloseDenial::Relational(
                    RelationalRuntimeCloseDenial::PublicationCustodyRetained,
                ),
            posture: ApplicationHomeClosePosture::Serving(home),
        } => home,
        ApplicationHomeCloseOutcome::Closed { .. }
        | ApplicationHomeCloseOutcome::CloseRefused { .. } => {
            panic!("unsettled performed custody must preserve the serving home")
        }
    };
    assert_eq!(home.home_identity(), home_identity);
    assert!(port.retains_pending_settlement(identity));
    let settled = port.settle_performed_publication(performed).unwrap();
    assert_eq!(settled.commit.commit_id, identity);
    assert!(!port.retains_pending_settlement(identity));
    // Settlement returns a real read pin. Its consumer releases it separately;
    // releasing publication custody never implicitly releases snapshot custody.
    home.relational()
        .runtime()
        .snapshots()
        .release_snapshot(&settled.snapshot)
        .unwrap();
    drop(settled);
    drop(port);
    assert!(matches!(
        home.close(&execution),
        ApplicationHomeCloseOutcome::Closed { .. }
    ));
}
