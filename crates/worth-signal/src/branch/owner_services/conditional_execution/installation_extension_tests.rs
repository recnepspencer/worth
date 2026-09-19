use crate::branch::owner_services::SignalOwnerCancellationSource;
use crate::data::aspect::{Aspect, AspectMask};
use crate::data::conditional_execution::{
    SignalConditionalArtifactReuse, SignalConditionalCondition,
    SignalConditionalContractDefinition, SignalConditionalVersionComparator,
};
use crate::data::retained_storage::SignalConditionalRetentionReservation;
use crate::runtime_policy::{SignalConditionalEvaluationBudget, SignalRuntimePolicy};

use super::nested_execution_tests::runtime_with_contract;

fn successor_definition() -> SignalConditionalContractDefinition {
    SignalConditionalContractDefinition {
        condition: SignalConditionalCondition::Always,
        dependency_aspects: AspectMask::from_aspect(Aspect::new(1)),
        trigger_aspects: AspectMask::from_aspect(Aspect::new(1)),
        dependency_comparator: SignalConditionalVersionComparator::Exact,
        output_comparator: SignalConditionalVersionComparator::Exact,
        artifact_reuse: SignalConditionalArtifactReuse::OutputEquivalent,
    }
}

fn prepare_with_budget(
    maximum_slots: usize,
    maximum_bytes: u64,
    maximum_visits: usize,
) -> (
    Result<
        super::SignalPreparedConditionalInstallationExtension,
        super::SignalConditionalInstallationExtensionDenial,
    >,
    crate::branch::AdmittedSignalBranchBasis,
    crate::branch::AdmittedSignalBranchBasis,
    u64,
    (usize, u64),
    (usize, u64),
) {
    let (mut runtime, claimant, contract, source_owner, _) = runtime_with_contract();
    runtime.set_runtime_policy(
        SignalRuntimePolicy::development().with_conditional_evaluation_budget(
            SignalConditionalEvaluationBudget {
                maximum_retained_slots: maximum_slots,
                maximum_retained_bytes: maximum_bytes,
                maximum_attempt_visits: maximum_visits.max(100_000),
            },
        ),
    );
    let before = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    let services = runtime.owner_component_services().unwrap();
    let reference = services
        .basis_port()
        .issue_managed_branch_reference(&before)
        .unwrap();
    let port = runtime
        .issue_conditional_execution_service(&before, &claimant, &source_owner.authority())
        .unwrap();
    if maximum_visits < 100_000 {
        port.set_installation_work_budget_for_test(maximum_visits);
    }
    let ledger = services
        .mutation_port()
        .upgrade_owner()
        .unwrap()
        .conditional_retention
        .clone();
    let baseline = ledger.usage();
    let result = port.prepare_installation_extension(
        contract.node(),
        contract.generation(),
        successor_definition(),
    );
    let after = services.basis_port().observe_current(&reference).unwrap();
    let final_usage = ledger.usage();
    (
        result,
        before,
        after,
        contract.generation(),
        baseline,
        final_usage,
    )
}

#[test]
fn installation_extension_applies_only_inside_exact_owner_transaction_and_rolls_back() {
    let (mut runtime, claimant, contract, source_owner, _) = runtime_with_contract();
    let node = contract.node();
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    let services = runtime.owner_component_services().unwrap();
    let port = runtime
        .issue_conditional_execution_service(&basis, &claimant, &source_owner.authority())
        .unwrap();
    let cancellation = SignalOwnerCancellationSource::new();
    let prepared = port
        .prepare_installation_extension(node, contract.generation(), successor_definition())
        .unwrap();
    let (_publication, request) = prepared.into_parts();
    let mut scope_denial = None;
    let failed = services.mutation_port().advance_exact(
        &basis,
        &mut (),
        &cancellation.token(),
        |transaction| {
            scope_denial = match port.apply_installation_extension(transaction, request) {
                Ok(_) => panic!("ordinary Signal advancement must not publish a definition"),
                Err(denial) => Some(denial),
            };
            Err(crate::data::error::SignalError::internal(
                "reject after apply",
            ))
        },
    );
    assert!(matches!(
        failed,
        Err(crate::branch::SignalBranchAdvanceDenial::MutationFailedNoMovement { .. })
    ));
    assert!(matches!(
        scope_denial,
        Some(super::SignalConditionalInstallationExtensionDenial::TransactionScopeMismatch)
    ));

    let retry = port
        .prepare_installation_extension(node, contract.generation(), successor_definition())
        .unwrap();
    let (publication, retry) = retry.into_parts();
    let mut completion = None;
    let mut advanced = services
        .mutation_port()
        .advance_conditional_definition_exact_with_completion(
            publication,
            &basis,
            &mut (),
            &cancellation.token(),
            |transaction| {
                completion = Some(
                    port.apply_installation_extension(transaction, retry)
                        .unwrap(),
                );
                Ok(())
            },
        )
        .into_result()
        .unwrap();
    let installed = completion.expect("successful owner callback returns exact completion");
    assert_eq!(installed.predecessor_generation(), contract.generation());
    assert_eq!(installed.contract().generation(), contract.generation() + 1);
    assert_ne!(advanced.advanced_basis().observation(), basis.observation());

    let ledger = services
        .mutation_port()
        .upgrade_owner()
        .unwrap()
        .conditional_retention
        .clone();
    let binding = advanced
        .take_conditional_definition_advance_binding()
        .expect("definition advance mints its exact completion binding");
    let (_, custody, _successor_service) = installed.into_retained_parts(binding).unwrap();
    let held = ledger.usage();
    let second_holder = custody.clone();
    assert_eq!(
        ledger.usage(),
        held,
        "cloning shares one custody reservation"
    );
    drop(custody);
    assert_eq!(
        ledger.usage(),
        held,
        "the first drop cannot release shared definition custody"
    );
    drop(second_holder);
    assert!(
        ledger.usage().1 < held.1,
        "the final holder releases definition custody"
    );
}

#[test]
fn installation_extension_preflights_exact_bytes_before_prepared_contract_allocation() {
    let (calibration, _, _, _, baseline, _) = prepare_with_budget(64, 64 * 1024 * 1024, 100_000);
    drop(calibration.unwrap());
    // This Always definition has no dynamic condition payload. Its retained
    // allocation carries the definition and the four Arc bookkeeping words.
    let payload = std::mem::size_of::<SignalConditionalContractDefinition>()
        .checked_add(4 * std::mem::size_of::<usize>())
        .unwrap() as u64;
    let handle = std::mem::size_of::<SignalConditionalRetentionReservation>() as u64;
    let exact_slots = baseline.0 + 1;
    let exact_bytes = baseline.1 + payload + handle;

    let (exact, exact_before, exact_after, generation, _, exact_usage) =
        prepare_with_budget(exact_slots, exact_bytes, 100_000);
    let prepared = exact.expect("the independently derived exact byte budget must admit");
    assert_eq!(exact_before.observation(), exact_after.observation());
    assert_eq!(exact_usage, (exact_slots, exact_bytes));
    drop(prepared);

    let (short, short_before, short_after, short_generation, short_baseline, short_final) =
        prepare_with_budget(exact_slots, exact_bytes - 1, 100_000);
    assert!(matches!(
        short,
        Err(super::SignalConditionalInstallationExtensionDenial::CapacityExhausted)
    ));
    assert_eq!(short_before.observation(), short_after.observation());
    assert_eq!(generation, short_generation);
    assert_eq!(short_baseline, baseline);
    assert_eq!(short_final, short_baseline);
}

#[test]
fn installation_extension_one_work_short_moves_no_basis_or_definition() {
    let (result, before, after, generation, baseline, final_usage) =
        prepare_with_budget(64, 64 * 1024 * 1024, 1);
    assert!(matches!(
        result,
        Err(
            super::SignalConditionalInstallationExtensionDenial::WorkExhausted {
                maximum_visits: 1
            }
        )
    ));
    assert_eq!(before.observation(), after.observation());
    assert_eq!(final_usage, baseline);
    assert_eq!(baseline.0, 0);
    assert_eq!(generation, 1);
}

#[test]
fn completion_rejects_a_genuine_same_owner_sibling_advance_binding() {
    let (mut runtime, claimant, contract, source_owner, _) = runtime_with_contract();
    let root = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    let services = runtime.owner_component_services().unwrap();
    let cancellation = SignalOwnerCancellationSource::new();
    let sibling = services
        .mutation_port()
        .fork_exact(
            crate::branch::validate_signal_branch_name("definition-binding-sibling").unwrap(),
            &root,
            &cancellation.token(),
        )
        .unwrap();
    let root_port = runtime
        .issue_conditional_execution_service(&root, &claimant, &source_owner.authority())
        .unwrap();
    let sibling_port = runtime
        .issue_conditional_execution_service(
            sibling.created_basis(),
            &claimant,
            &source_owner.authority(),
        )
        .unwrap();

    let (root_publication, root_request) = root_port
        .prepare_installation_extension(
            contract.node(),
            contract.generation(),
            successor_definition(),
        )
        .unwrap()
        .into_parts();
    let (sibling_publication, sibling_request) = sibling_port
        .prepare_installation_extension(
            contract.node(),
            contract.generation(),
            successor_definition(),
        )
        .unwrap()
        .into_parts();
    let mut root_completion = None;
    let mut root_advance = services
        .mutation_port()
        .advance_conditional_definition_exact_with_completion(
            root_publication,
            &root,
            &mut (),
            &cancellation.token(),
            |transaction| {
                root_completion = Some(
                    root_port
                        .apply_installation_extension(transaction, root_request)
                        .unwrap(),
                );
                Ok(())
            },
        )
        .into_result()
        .unwrap();
    let mut sibling_completion = None;
    let mut sibling_advance = services
        .mutation_port()
        .advance_conditional_definition_exact_with_completion(
            sibling_publication,
            sibling.created_basis(),
            &mut (),
            &cancellation.token(),
            |transaction| {
                sibling_completion = Some(
                    sibling_port
                        .apply_installation_extension(transaction, sibling_request)
                        .unwrap(),
                );
                Ok(())
            },
        )
        .into_result()
        .unwrap();
    let sibling_binding = sibling_advance
        .take_conditional_definition_advance_binding()
        .unwrap();
    let denial = match root_completion
        .unwrap()
        .into_retained_parts(sibling_binding)
    {
        Ok(_) => panic!("a sibling's genuine advance binding opened the root completion"),
        Err(denial) => denial,
    };
    assert!(matches!(
        denial,
        super::SignalConditionalInstallationExtensionDenial::SuccessorBindingMismatch
    ));
    let root_binding = root_advance
        .take_conditional_definition_advance_binding()
        .unwrap();
    let sibling_binding = sibling_completion.unwrap();
    assert!(matches!(
        sibling_binding.into_retained_parts(root_binding),
        Err(super::SignalConditionalInstallationExtensionDenial::SuccessorBindingMismatch)
    ));
}
