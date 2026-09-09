use crate::branch::owner_services::SignalOwnerCancellationSource;
use crate::branch::SignalBranchAdvanceDenial;
use crate::data::comparator::DefaultComparatorPolicyResolver;
use crate::data::error::SignalError;
use crate::data::output::NodeEvaluationResult;
use worth_proof::ConditionalEvaluationSource;

use super::nested_execution_tests::{runtime_with_contract, NoPredicate};
use super::SignalConditionalServiceExecutionRequest as Request;

#[test]
fn nested_provider_unwind_with_failed_rollback_quarantines_the_cell() {
    let (mut runtime, claimant, contract, source_owner, [rollback_target, _, _]) =
        runtime_with_contract();
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    let (_, mutation, _) = runtime.owner_port_slots().unwrap();
    let service = runtime
        .issue_conditional_execution_service(&basis, &claimant, &source_owner.authority())
        .unwrap();
    let evaluation = service
        .admit_evaluation(
            &contract,
            ConditionalEvaluationSource::AdmittedRelationalSource(source_owner.admit("source")),
        )
        .unwrap();
    let cancellation = SignalOwnerCancellationSource::new();
    let mut provider_called = false;

    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = mutation.advance_exact(&basis, &mut (), &cancellation.token(), |transaction| {
            transaction.inject_stale_graph_patch_rollback_packet_for_test(rollback_target);
            let _ = service.execute_within_transaction(
                transaction,
                &evaluation,
                Request::new(1),
                &mut NoPredicate,
                &mut DefaultComparatorPolicyResolver::default(),
                || -> Result<NodeEvaluationResult, SignalError> {
                    provider_called = true;
                    panic!("provider panic reaches failing rollback")
                },
            );
            Ok(())
        });
    }));
    assert!(unwind.is_err());
    assert!(provider_called);

    let mut retry_callback_called = false;
    let retry = mutation.advance_exact(&basis, &mut (), &cancellation.token(), |_| {
        retry_callback_called = true;
        Ok(())
    });
    assert!(matches!(
        retry,
        Err(SignalBranchAdvanceDenial::QuarantinedBranch { branch_id })
            if branch_id == basis.branch_id()
    ));
    assert!(!retry_callback_called);
}
