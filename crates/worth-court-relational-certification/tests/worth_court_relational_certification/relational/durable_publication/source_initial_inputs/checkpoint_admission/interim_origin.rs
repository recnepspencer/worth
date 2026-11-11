//! A real empty Memory capture still requires actual receiver admission custody.
use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_foundational::CanonicalDigestId;
use worth_relational::facade::{
    durability::RelationalCheckpointAdmissionRefusal, runtime::RelationalRuntimeApi,
};

fn capture_origin(
    source: &worth_relational::facade::runtime::RelationalRuntime,
    request: &ExecutionRequest<'_, '_>,
) -> worth_relational::facade::durability::CapturedNativeInterimOrigin {
    let (native, mut capture) = source
        .durability_authority()
        .capture_interim_checkpoint(request, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    // The Native-only transport proves no World movement. A World owner must
    // independently authenticate its own section before composing an application.
    let image = capture
        .emit_interim_image(0, CanonicalDigestId::new([0; 32]), request)
        .unwrap();
    let native_len = native.bytes().len();
    let total = native_len + image.as_bytes().len();
    let mut bytes = ExecutionRequestArrayBuilder::allocate(*request, total).unwrap();
    bytes.extend_from_slice(native.bytes()).unwrap();
    bytes.extend_from_slice(image.as_bytes()).unwrap();
    drop(native);
    capture
        .seal_transport(
            bytes.seal_immutable_bytes().unwrap(),
            0..native_len,
            native_len..total,
            request,
        )
        .unwrap()
}

#[test]
fn empty_interim_image_requires_its_actual_recovered_native_receiver() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let source = RelationalRuntimeApi::builder().build().unwrap();
    let origin = capture_origin(&source, request);
    assert_eq!(origin.admission_cut().canonical_through(), 0);
    drop(source);
    let mut receiver = RelationalRuntimeApi::builder().build().unwrap();
    assert!(matches!(
        receiver
            .durability_authority()
            .admit_interim_receiver(&origin, request),
        Err(RelationalCheckpointAdmissionRefusal::MissingCaptureOrigin)
    ));
    receiver
        .durability_recovery()
        .restore_interim_checkpoint_with_authority(request, &origin)
        .unwrap();
    receiver
        .durability_authority()
        .admit_interim_receiver(&origin, request)
        .unwrap();
    let unrelated = RelationalRuntimeApi::builder().build().unwrap();
    assert!(matches!(
        unrelated
            .durability_authority()
            .admit_interim_receiver(&origin, request),
        Err(RelationalCheckpointAdmissionRefusal::MissingCaptureOrigin)
    ));
}

#[test]
fn interim_bootstrap_continuations_share_the_consuming_publication_phase() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let source = RelationalRuntimeApi::builder().build().unwrap();
    let origin = capture_origin(&source, request);
    drop(source);
    let mut receiver = RelationalRuntimeApi::builder().build().unwrap();
    let (_, mut recovered) = receiver
        .durability_recovery()
        .restore_interim_checkpoint_with_authority(request, &origin)
        .unwrap();
    let first = receiver
        .durability_recovery()
        .prepare_interim_bootstrap_continuation(request, &origin, &mut recovered)
        .unwrap();
    let second = receiver
        .durability_recovery()
        .prepare_interim_bootstrap_continuation(request, &origin, &mut recovered)
        .unwrap();
    let (_, capture) = receiver
        .durability_authority()
        .capture_interim_checkpoint(request, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let port = receiver.owner_component_services().basis_port();
    for continuation in [&first, &second] {
        port.capture_interim_bootstrap_component(request, continuation, &capture)
            .unwrap();
    }
    let foreign = RelationalRuntimeApi::builder().build().unwrap();
    let (_, foreign_capture) = foreign
        .durability_authority()
        .capture_interim_checkpoint(request, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(port
        .capture_interim_bootstrap_component(request, &first, &foreign_capture)
        .is_err());
    let (_, basis) = receiver
        .observe_branch(&receiver.main_branch_identity())
        .unwrap();
    let basis = recovered.admit_basis(basis).unwrap().into_basis();
    for continuation in [&first, &second] {
        assert!(matches!(
            port.capture_interim_bootstrap_component(request, continuation, &capture),
            Err(RelationalCheckpointAdmissionRefusal::OwnerUnavailable)
        ));
    }
    port.install_interim_bootstrap_receiver(request, &first, first.original_receiver(), &basis)
        .unwrap();
    assert!(matches!(
        port.install_interim_bootstrap_receiver(
            request,
            &second,
            second.original_receiver(),
            &basis
        ),
        Err(RelationalCheckpointAdmissionRefusal::OwnerUnavailable)
    ));
}

#[test]
fn failed_recovered_basis_consumption_closes_every_interim_continuation() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let source = RelationalRuntimeApi::builder().build().unwrap();
    let origin = capture_origin(&source, request);
    drop(source);
    let mut receiver = RelationalRuntimeApi::builder().build().unwrap();
    let (_, mut recovered) = receiver
        .durability_recovery()
        .restore_interim_checkpoint_with_authority(request, &origin)
        .unwrap();
    let continuation = receiver
        .durability_recovery()
        .prepare_interim_bootstrap_continuation(request, &origin, &mut recovered)
        .unwrap();
    let foreign = RelationalRuntimeApi::builder().build().unwrap();
    let (_, foreign_basis) = foreign
        .observe_branch(&foreign.main_branch_identity())
        .unwrap();
    assert!(recovered.admit_basis(foreign_basis).is_err());
    let (_, capture) = receiver
        .durability_authority()
        .capture_interim_checkpoint(request, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        receiver
            .owner_component_services()
            .basis_port()
            .capture_interim_bootstrap_component(request, &continuation, &capture),
        Err(RelationalCheckpointAdmissionRefusal::OwnerUnavailable)
    ));
}

#[test]
fn interim_bootstrap_continuation_refuses_an_unwitnessed_native_successor() {
    use worth_testkit_relational::durable_publication::{create_entity, memory_runtime};
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let source = memory_runtime();
    let origin = capture_origin(&source, request);
    drop(source);
    let mut receiver = memory_runtime();
    let (_, mut recovered) = receiver
        .durability_recovery()
        .restore_interim_checkpoint_with_authority(request, &origin)
        .unwrap();
    let continuation = receiver
        .durability_recovery()
        .prepare_interim_bootstrap_continuation(request, &origin, &mut recovered)
        .unwrap();
    create_entity(request, &receiver, "ordinary-tail");
    assert!(matches!(
        receiver
            .durability_recovery()
            .prepare_interim_bootstrap_continuation(request, &origin, &mut recovered),
        Err(RelationalCheckpointAdmissionRefusal::BindingMismatch(
            worth_relational::facade::durability::CheckpointAdmissionBindingMismatch::NativeOrigin
        ))
    ));
    let (_, capture) = receiver
        .durability_authority()
        .capture_interim_checkpoint(request, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(receiver
        .owner_component_services()
        .basis_port()
        .capture_interim_bootstrap_component(request, &continuation, &capture)
        .is_err());
}

#[test]
fn abandoning_recovery_closes_its_interim_capture_continuations() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let source = RelationalRuntimeApi::builder().build().unwrap();
    let origin = capture_origin(&source, request);
    drop(source);
    let mut receiver = RelationalRuntimeApi::builder().build().unwrap();
    let (_, mut recovered) = receiver
        .durability_recovery()
        .restore_interim_checkpoint_with_authority(request, &origin)
        .unwrap();
    let continuation = receiver
        .durability_recovery()
        .prepare_interim_bootstrap_continuation(request, &origin, &mut recovered)
        .unwrap();
    drop(recovered);
    let (_, capture) = receiver
        .durability_authority()
        .capture_interim_checkpoint(request, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        receiver
            .owner_component_services()
            .basis_port()
            .capture_interim_bootstrap_component(request, &continuation, &capture),
        Err(RelationalCheckpointAdmissionRefusal::OwnerUnavailable)
    ));
}

#[test]
fn interim_receiver_roster_validation_refuses_an_unfunded_walk_and_retries() {
    use worth_testkit_relational::durable_publication::{create_entity, memory_runtime};
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let source = memory_runtime();
    create_entity(request, &source, "funded-roster-row");
    let single = capture_origin(&source, request);
    for index in 1..4 {
        create_entity(request, &source, &format!("funded-roster-row-{index}"));
    }
    let origin = capture_origin(&source, request);
    drop(source);
    let mut receiver = memory_runtime();
    receiver
        .durability_recovery()
        .restore_interim_checkpoint_with_authority(request, &origin)
        .unwrap();
    let policy = worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, Some(16)),
    );
    let tiny = worth_execution::SerialRequest::from_policy(
        &policy,
        worth_execution::CancellationToken::new(),
        None,
    );
    let mut one_row_receiver = memory_runtime();
    one_row_receiver
        .durability_recovery()
        .restore_interim_checkpoint_with_authority(request, &single)
        .unwrap();
    one_row_receiver
        .durability_authority()
        .admit_interim_receiver(&single, &ExecutionRequest::serial(&tiny))
        .unwrap();
    assert!(matches!(
        receiver
            .durability_authority()
            .admit_interim_receiver(&origin, &ExecutionRequest::serial(&tiny)),
        Err(RelationalCheckpointAdmissionRefusal::Request(
            worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted
        ))
    ));
    receiver
        .durability_authority()
        .admit_interim_receiver(&origin, request)
        .unwrap();
}
