//! No-movement refusals return the original admitted definition publication.
use super::{runtime_with_contract, successor_definition};
use crate::branch::owner_services::{
    SignalConditionalDefinitionPublicationPort, SignalOwnerCancellationSource,
};
use crate::branch::SignalBranchAdvanceDenial;
use crate::data::error::SignalError;

#[test]
fn original_admitted_publication_survives_stops_until_actual_movement() {
    let serial = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let (mut runtime, claimant, contract, source_owner, _) = runtime_with_contract();
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    let services = runtime.owner_component_services().unwrap();
    let reference = services
        .basis_port()
        .issue_managed_branch_reference(&basis)
        .unwrap();
    let definition = runtime
        .issue_conditional_execution_service(&basis, &claimant, &source_owner.authority())
        .unwrap();
    let (operation, request) = definition
        .prepare_installation_extension(
            contract.node(),
            contract.generation(),
            successor_definition(),
        )
        .unwrap()
        .into_parts();
    let publication =
        SignalConditionalDefinitionPublicationPort::new(services.mutation_port().clone());
    let admitted = publication.admit(operation, &basis).unwrap();
    let stopped = SignalOwnerCancellationSource::new();
    stopped.cancel();
    let (completion, admitted) = publication.advance_exact_with_completion(
        execution,
        admitted,
        &basis,
        &mut (),
        &stopped.token(),
        |_| panic!("cancelled publication cannot enter the installation callback"),
    );
    assert!(matches!(
        completion.into_result(),
        Err(SignalBranchAdvanceDenial::CancelledNoMovement)
    ));
    let admitted = admitted.expect("cancellation returns the original admitted permit");
    assert_eq!(
        services
            .basis_port()
            .observe_current(&reference)
            .unwrap()
            .observation(),
        basis.observation()
    );

    let live = SignalOwnerCancellationSource::new();
    let mut contacts = 0;
    let (completion, admitted) = publication.advance_exact_with_completion(
        execution,
        admitted,
        &basis,
        &mut (),
        &live.token(),
        |_| {
            contacts += 1;
            Err(SignalError::internal(
                "refuse before applying the retained installation",
            ))
        },
    );
    assert!(matches!(
        completion.into_result(),
        Err(SignalBranchAdvanceDenial::MutationFailedNoMovement { .. })
    ));
    let admitted = admitted.expect("no movement preserves the same publication permit");
    assert_eq!(contacts, 1);
    assert_eq!(
        services
            .basis_port()
            .observe_current(&reference)
            .unwrap()
            .observation(),
        basis.observation()
    );

    let (mut unwound, admitted) = publication.advance_exact_with_completion(
        execution,
        admitted,
        &basis,
        &mut (),
        &live.token(),
        |_| panic!("unwind before applying the retained installation"),
    );
    assert!(unwound.take_result().is_none());
    let admitted = admitted.expect("unwinding returns custody before its payload resumes");
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unwound.resume_unwind())).is_err()
    );
    assert_eq!(
        services
            .basis_port()
            .observe_current(&reference)
            .unwrap()
            .observation(),
        basis.observation()
    );

    let mut installed = None;
    let (completion, remaining) = publication.advance_exact_with_completion(
        execution,
        admitted,
        &basis,
        &mut (),
        &live.token(),
        |transaction| {
            contacts += 1;
            installed = Some(
                definition
                    .apply_installation_extension(transaction, request)
                    .unwrap(),
            );
            Ok(())
        },
    );
    assert!(
        remaining.is_none(),
        "performed movement consumes its one-use permit"
    );
    let mut advanced = completion.into_result().unwrap();
    assert_eq!(contacts, 2);
    assert_ne!(advanced.advanced_basis().observation(), basis.observation());
    let binding = advanced
        .take_conditional_definition_advance_binding()
        .unwrap();
    let installed = installed.expect("the actual installation callback completed");
    assert_eq!(installed.contract().generation(), contract.generation() + 1);
    installed.into_retained_parts(binding).unwrap();
}
