use crate::branch::owner_services::SignalOwnerCancellationSource;
use crate::branch::validate_signal_branch_name;
use crate::data::aspect::{Aspect, AspectMask, AspectVersion, SignalAspectLoweringOwner};
use crate::data::comparator::DefaultComparatorPolicyResolver;
use crate::data::conditional_execution::{
    InstalledSignalConditionDecision, InstalledSignalConditionResolver,
    InstalledSignalConditionalContract, SignalConditionalArtifactReuse, SignalConditionalCondition,
    SignalConditionalContractDefinition, SignalConditionalDecisionClass,
    SignalConditionalVersionComparator,
};
use crate::data::dependency::DependencyEdge;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::data::output::NodeEvaluationResult;
use crate::logic::transaction::SignalRuntime;
use worth_proof::{ConditionalEvaluationSource, ConditionalSourceObservationOwner};

use super::{
    SignalConditionalServiceExecutionDenial as Denial,
    SignalConditionalServiceExecutionRequest as Request,
};

pub(super) struct NoPredicate;

impl InstalledSignalConditionResolver for NoPredicate {
    fn resolve(
        &mut self,
        _: &crate::data::node::InstalledSignalConditionIdentity,
        _: &crate::logic::evaluation::ConditionEvaluationContext,
    ) -> Result<InstalledSignalConditionDecision, SignalError> {
        panic!("Always does not contact a predicate provider")
    }
}

pub(super) fn runtime_with_contract() -> (
    SignalRuntime<(), (), (), (), ()>,
    SignalAspectLoweringOwner,
    InstalledSignalConditionalContract,
    ConditionalSourceObservationOwner,
    [NodeId; 3],
) {
    let mut graph = SignalGraph::new();
    let claimant = SignalAspectLoweringOwner::fresh();
    graph.claim_aspect_lowering_owner(&claimant).unwrap();
    let initial_dependency = graph.create_node();
    let replacement_dependency = graph.create_node();
    let rollback_target = graph.create_node();
    let node = graph.node().build();
    let worth_proof::TransitionOutcome::Success(capability) = graph.admit_installed_node(node)
    else {
        panic!("fresh node must admit")
    };
    graph
        .set_dependencies(
            rollback_target,
            [DependencyEdge::new(initial_dependency, Aspect::new(1))],
        )
        .unwrap();
    let contract = graph
        .install_conditional_contract(
            &claimant,
            capability,
            SignalConditionalContractDefinition {
                condition: SignalConditionalCondition::Always,
                dependency_aspects: AspectMask::from_aspect(Aspect::new(1)),
                trigger_aspects: AspectMask::from_aspect(Aspect::new(1)),
                dependency_comparator: SignalConditionalVersionComparator::Exact,
                output_comparator: SignalConditionalVersionComparator::Exact,
                artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
            },
        )
        .unwrap();
    (
        SignalRuntime::build_for::<()>(graph),
        claimant,
        contract,
        ConditionalSourceObservationOwner::fresh(),
        [rollback_target, initial_dependency, replacement_dependency],
    )
}

fn output(value: u64) -> NodeEvaluationResult {
    NodeEvaluationResult::from_version(AspectVersion::from_updates([(Aspect::new(0), value)]))
}

#[test]
fn nested_execution_requires_the_exact_owner_transaction_and_adds_no_cell_contact() {
    let (mut runtime, claimant, contract, source_owner, _) = runtime_with_contract();
    let root_basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    let (basis_port, mutation, _) = runtime.owner_port_slots().unwrap();
    let owner = basis_port.upgrade_owner().unwrap();
    let service = runtime
        .issue_conditional_execution_service(&root_basis, &claimant, &source_owner.authority())
        .unwrap();
    let evaluation = service
        .admit_evaluation(
            &contract,
            ConditionalEvaluationSource::AdmittedRelationalSource(
                source_owner.admit("nested-source"),
            ),
        )
        .unwrap();
    let cancellation = SignalOwnerCancellationSource::new();
    let sibling = mutation
        .fork_exact(
            validate_signal_branch_name("nested-sibling").unwrap(),
            &root_basis,
            &cancellation.token(),
        )
        .unwrap();

    let mut sibling_computes = 0;
    let sibling_advanced = mutation
        .advance_exact(
            sibling.created_basis(),
            &mut (),
            &cancellation.token(),
            |transaction| {
                let result = service.execute_within_transaction(
                    transaction,
                    &evaluation,
                    Request::new(1),
                    &mut NoPredicate,
                    &mut DefaultComparatorPolicyResolver::default(),
                    || {
                        sibling_computes += 1;
                        Ok(output(5))
                    },
                );
                assert!(matches!(result, Err(Denial::NestedOperationScopeMismatch)));
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(sibling_computes, 0);

    let before = owner.cost_snapshot();
    let mut completion = None;
    let root_advanced = mutation
        .advance_exact(&root_basis, &mut (), &cancellation.token(), |transaction| {
            completion = Some(
                service
                    .execute_within_transaction(
                        transaction,
                        &evaluation,
                        Request::new(1),
                        &mut NoPredicate,
                        &mut DefaultComparatorPolicyResolver::default(),
                        || Ok(output(9)),
                    )
                    .unwrap(),
            );
            Ok(())
        })
        .unwrap();
    let after = owner.cost_snapshot();
    assert_eq!(
        after.owner_upgrade_attempts(),
        before.owner_upgrade_attempts() + 1
    );
    assert_eq!(
        after.branch_registry_lookups(),
        before.branch_registry_lookups() + 1
    );
    assert_eq!(
        after.target_cell_contacts(),
        before.target_cell_contacts() + 1
    );
    assert_eq!(after.target_cell_waits(), before.target_cell_waits());
    assert_eq!(
        after.canonical_movements(),
        before.canonical_movements() + 1
    );

    let (decision, observation) = completion.unwrap().into_parts();
    let decision = decision.unwrap();
    assert_eq!(
        decision.class(),
        SignalConditionalDecisionClass::ComputedChanged
    );
    assert_eq!(decision.output_version(), 9);
    assert!(observation.unwrap().is_some());
    assert_ne!(
        sibling_advanced.advanced_basis().branch_id(),
        root_advanced.advanced_basis().branch_id()
    );
    assert_eq!(
        sibling_advanced
            .advanced_basis()
            .observation()
            .generation()
            .get(),
        1
    );
    assert_eq!(
        root_advanced
            .advanced_basis()
            .observation()
            .generation()
            .get(),
        1
    );
}

#[test]
fn plain_runtime_transaction_is_denied_before_provider_work() {
    let (mut runtime, claimant, contract, source_owner, _) = runtime_with_contract();
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    runtime.owner_port_slots().unwrap();
    let service = runtime
        .issue_conditional_execution_service(&basis, &claimant, &source_owner.authority())
        .unwrap();
    let evaluation = service
        .admit_evaluation(
            &contract,
            ConditionalEvaluationSource::AdmittedRelationalSource(source_owner.admit("source")),
        )
        .unwrap();
    let mut plain = SignalRuntime::<(), (), (), (), ()>::build_for::<()>(SignalGraph::new());
    let mut context = ();
    let mut transaction = plain.begin(&mut context);
    let mut computes = 0;

    let result = service.execute_within_transaction(
        &mut transaction,
        &evaluation,
        Request::new(1),
        &mut NoPredicate,
        &mut DefaultComparatorPolicyResolver::default(),
        || {
            computes += 1;
            Ok(output(3))
        },
    );
    assert!(matches!(result, Err(Denial::NestedOperationScopeMismatch)));
    assert_eq!(computes, 0);
}

#[test]
fn nested_provider_unwind_rolls_back_and_preserves_the_cell_and_evaluation_slot() {
    let (
        mut runtime,
        claimant,
        contract,
        source_owner,
        [rollback_target, initial_dependency, replacement_dependency],
    ) = runtime_with_contract();
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    let (basis_port, mutation, _) = runtime.owner_port_slots().unwrap();
    let owner = basis_port.upgrade_owner().unwrap();
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
    let before = owner.cost_snapshot();

    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = mutation.advance_exact(&basis, &mut (), &cancellation.token(), |transaction| {
            transaction.set_dependencies(
                rollback_target,
                [DependencyEdge::new(replacement_dependency, Aspect::new(1))],
            )?;
            let _ = service.execute_within_transaction(
                transaction,
                &evaluation,
                Request::new(1),
                &mut NoPredicate,
                &mut DefaultComparatorPolicyResolver::default(),
                || -> Result<NodeEvaluationResult, SignalError> {
                    panic!("nested provider unwind")
                },
            );
            Ok(())
        });
    }));
    assert!(unwind.is_err());
    assert_eq!(
        owner.cost_snapshot().canonical_movements(),
        before.canonical_movements()
    );

    let admission = owner.admit().unwrap();
    let cell = owner.lookup_cell(&admission, basis.branch_id()).unwrap();
    let dependencies = cell
        .with_state(&admission, |state, _| {
            state
                .state()
                .graph()
                .dependency_sources_of(rollback_target)
                .unwrap()
        })
        .unwrap();
    assert_eq!(dependencies, vec![initial_dependency]);
    drop(admission);

    let mut completion = None;
    let advanced = mutation
        .advance_exact(&basis, &mut (), &cancellation.token(), |transaction| {
            completion = Some(
                service
                    .execute_within_transaction(
                        transaction,
                        &evaluation,
                        Request::new(2),
                        &mut NoPredicate,
                        &mut DefaultComparatorPolicyResolver::default(),
                        || Ok(output(12)),
                    )
                    .unwrap(),
            );
            Ok(())
        })
        .unwrap();
    let (decision, observation) = completion.unwrap().into_parts();
    assert_eq!(decision.unwrap().output_version(), 12);
    assert!(observation.unwrap().is_some());
    assert_eq!(
        advanced.advanced_basis().observation().generation().get(),
        1
    );
    assert_eq!(
        owner.cost_snapshot().canonical_movements(),
        before.canonical_movements() + 1
    );
}
