//! No-effect profile and source-lifecycle refusals use the original owner path.
use super::*;

#[test]
fn retained_creation_rejects_foreign_cancelled_reused_and_retired_sources_before_forks() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let court = CompositeSupplyChainCourt::compile();
    let foreign = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let other = foreign.bootstrap();
    let branch = court.world.branch_port();
    assert!(matches!(
        branch.admit_retained_branch_source(&other),
        Err(RuntimeWorldServiceDenial::Denied(
            RuntimeWorldBranchAdmissionDenial::ForeignOwner
        ))
    ));
    let retained = branch.admit_retained_branch_source(&root).unwrap();
    let token = RuntimeWorldCancellationSource::new().token();
    assert!(matches!(
        branch.create_product_branch_from_retained_source(
            execution,
            retained,
            ProductBranchCreationIntent::from_source("reused", plans(true, false, "reused"))
                .unwrap(),
            &token
        ),
        Err(RuntimeWorldServiceDenial::Denied(
            RuntimeWorldBranchAdmissionDenial::UnsupportedRetainedSourcePlan
        ))
    ));
    let retained = branch.admit_retained_branch_source(&root).unwrap();
    let cancel = RuntimeWorldCancellationSource::new();
    cancel.cancel();
    assert!(matches!(
        branch.create_product_branch_from_retained_source(
            execution,
            retained,
            ProductBranchCreationIntent::from_source("cancelled", plans(true, true, "cancelled"))
                .unwrap(),
            &cancel.token()
        ),
        Err(RuntimeWorldServiceDenial::Denied(
            RuntimeWorldBranchAdmissionDenial::CancelledBeforeEffect
        ))
    ));
    for name in ["reused", "cancelled"] {
        assert!(court
            .records
            .runtime
            .branch_identity(&BranchId(name.into()))
            .is_err());
    }
    let RuntimeWorldBranchCreationOutcome::Performed(child) = branch
        .create_product_branch(
            execution,
            root.clone(),
            ProductBranchCreationIntent::from_source("retiring", plans(true, true, "retiring"))
                .unwrap(),
            &token,
        )
        .unwrap()
    else {
        panic!("healthy source twin")
    };
    let retained = branch.admit_retained_branch_source(&child).unwrap();
    let retired = branch.retire_product_branch(&child).unwrap();
    assert!(matches!(
        branch.create_product_branch_from_retained_source(
            execution,
            retained,
            ProductBranchCreationIntent::from_source(
                "retired-loser",
                plans(true, true, "retired-loser")
            )
            .unwrap(),
            &token
        ),
        Err(RuntimeWorldServiceDenial::Denied(
            RuntimeWorldBranchAdmissionDenial::RetiredBranch
        ))
    ));
    assert!(court
        .records
        .runtime
        .branch_identity(&BranchId("retired-loser".into()))
        .is_err());
    drop((retired, child, root, other));
    foreign.finish();
    court.finish();
}
