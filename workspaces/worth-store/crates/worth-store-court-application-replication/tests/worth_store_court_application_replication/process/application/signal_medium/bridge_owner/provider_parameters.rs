//! Durable bytes select executable parameters, while fresh installation supplies services.
use super::*;
use program::parameters::{Fresh, Offset};
use worth_foundational::facade::DurableSignalBranchIdentity;
use worth_runtime_bridge::facade::*;

fn readmit_parameters(
    medium: Arc<dyn SignalDurableMedium>,
    calls: Arc<AtomicUsize>,
    execution: &ExecutionRequest<'_, '_>,
) -> Box<BridgeReadmittedAuthoredAssembly> {
    let program = program::parameters::program(calls);
    let mut bindings = Fresh(
        program
            .builder
            .retain_fresh_authored_bindings(execution)
            .unwrap(),
    );
    program
        .builder
        .readmit_authored_owner(
            medium,
            SignalRecordedIdentityContribution::empty(media::owner()),
            &mut bindings,
            execution,
        )
        .unwrap()
}
fn lowering(
    owner: &BridgeReadmittedAuthoredAssembly,
    branch: DurableSignalBranchIdentity,
) -> Arc<BridgeInstalledConditionalLowering> {
    owner
        .with_authored_branch(branch, |view| {
            view.with_conditional_definitions(|definitions| definitions.next().unwrap())
        })
        .unwrap()
}
fn candidate(
    predecessor: &BridgeInstalledConditionalLowering,
    provider: Offset,
) -> BridgeOwnedConditionalInstallationRequest {
    BridgeOwnedConditionalInstallationRequest {
        contract: predecessor.contract().clone(),
        location: predecessor.location().clone(),
        dependencies: Vec::new(),
        providers: BridgeConditionalProviderSet::new().authored_compute(provider),
    }
}
fn demand_parameter(
    owner: &BridgeReadmittedAuthoredAssembly,
    branch: DurableSignalBranchIdentity,
    execution: &ExecutionRequest<'_, '_>,
    expected: u64,
) {
    let lowering = lowering(owner, branch);
    let port = owner.signal_owner().source_port();
    let descriptor = port.observe(branch).unwrap();
    let source = port.readmit_descriptor(&descriptor).unwrap();
    let runtime = owner.conditional_execution(branch).unwrap();
    let binding = runtime
        .admit_conditional_signal_basis(&lowering, source.basis())
        .unwrap();
    let decision = runtime
        .execute(
            *execution,
            &binding,
            BridgeConditionalExecutionRequest {
                lowering: &lowering,
                query_binding_identity: "parameter-query",
                query_capability_identity: 1,
                snapshot_identity: "parameter-snapshot",
                truth_branch_identity: None,
                bridge_snapshot_identity: None,
                execution_identity: "parameter-demand",
                attempt: 1,
            },
            &mut 40u64,
        )
        .unwrap();
    assert_eq!(decision.signal().output_version(), expected);
}
#[test]
fn published_provider_parameters_reopen_per_branch_with_fresh_services_and_cold_values() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let unused_source_calls = Arc::new(AtomicUsize::new(0));
    let reference = source::reference_from_program(
        program::parameters::program(unused_source_calls.clone()),
        &execution,
    );
    let (serving, medium) = media::open(directory.path(), true, &execution);
    source::seed(medium.as_ref(), &reference, &execution);
    let calls = Arc::new(AtomicUsize::new(0));
    let candidate_calls = Arc::new(AtomicUsize::new(0));
    let owner = readmit_parameters(medium.clone(), calls.clone(), &execution);
    let branches = owner.signal_owner().source_port().branches().unwrap();
    for ((branch, _), offset) in branches.iter().zip([7, 13, 1]) {
        demand_parameter(&owner, *branch, &execution, 41);
        if offset != 1 {
            let descriptor = owner.signal_owner().source_port().observe(*branch).unwrap();
            let basis = owner
                .signal_owner()
                .source_port()
                .readmit_descriptor(&descriptor)
                .unwrap();
            let predecessor = lowering(&owner, *branch);
            let task = owner
                .begin_conditional_definition_publication(
                    &execution,
                    &basis,
                    &predecessor,
                    candidate(&predecessor, Offset::new(offset, candidate_calls.clone())),
                )
                .unwrap();
            owner_fixture::finish(task, &execution);
            demand_parameter(&owner, *branch, &execution, 40 + offset);
        }
    }
    assert_eq!(calls.load(Ordering::Relaxed), 5);
    assert_eq!(
        candidate_calls.load(Ordering::Relaxed),
        0,
        "candidate callbacks cannot replace installed services"
    );
    drop(owner);
    media::close(serving, medium, &execution);
    for (incarnation, build) in [
        (2, "11111111-1111-4111-8111-111111111111"),
        (3, "22222222-2222-4222-8222-222222222222"),
    ] {
        let (serving, medium) =
            media::open_with_identity(directory.path(), false, incarnation, build, &execution);
        let fresh_calls = Arc::new(AtomicUsize::new(0));
        let owner = readmit_parameters(medium.clone(), fresh_calls.clone(), &execution);
        assert_eq!(
            fresh_calls.load(Ordering::Relaxed),
            0,
            "readmission executes no provider"
        );
        for ((branch, _), expected) in branches.iter().zip([47, 53, 41]) {
            demand_parameter(&owner, *branch, &execution, expected);
        }
        assert_eq!(fresh_calls.load(Ordering::Relaxed), 3);
        drop(owner);
        media::close(serving, medium, &execution);
    }
    assert_eq!(unused_source_calls.load(Ordering::Relaxed), 0);
    assert_eq!(candidate_calls.load(Ordering::Relaxed), 0);
}
#[test]
fn provider_decoder_family_and_version_refuse_before_a_definition_moves() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let reference = source::reference_from_program(
        program::parameters::program(Arc::new(AtomicUsize::new(0))),
        &execution,
    );
    let (serving, medium) = media::open(directory.path(), true, &execution);
    source::seed(medium.as_ref(), &reference, &execution);
    let owner = readmit_parameters(medium.clone(), Arc::new(AtomicUsize::new(0)), &execution);
    let branch = reference.definitions[0].branch();
    let port = owner.signal_owner().source_port();
    let descriptor = port.observe(branch).unwrap();
    let basis = port.readmit_descriptor(&descriptor).unwrap();
    let predecessor = lowering(&owner, branch);
    for (protocol, version) in [
        ("worth.store.court.foreign", 1),
        ("worth.store.court.offset", 2),
    ] {
        let provider = Offset {
            value: 7,
            protocol,
            version,
            calls: Arc::new(AtomicUsize::new(0)),
        };
        let refusal = owner.begin_conditional_definition_publication(
            &execution,
            &basis,
            &predecessor,
            candidate(&predecessor, provider),
        );
        let Err(BridgeDurableDefinitionPublicationDenial::Section(cause)) = refusal else {
            panic!("expected provider source refusal");
        };
        if version == 2 {
            assert!(matches!(
                *cause,
                BridgeAuthoredSectionDenial::Provider(
                    BridgeAuthoredProviderDenial::UnsupportedVersion(_)
                )
            ));
        } else {
            assert!(matches!(
                *cause,
                BridgeAuthoredSectionDenial::Provider(
                    BridgeAuthoredProviderDenial::MissingDecoder(
                        BridgeConditionalProviderRole::Compute
                    )
                )
            ));
        }
        assert_eq!(port.observe(branch).unwrap(), descriptor);
    }
    drop((predecessor, basis, port, owner));
    media::close(serving, medium, &execution);
}
