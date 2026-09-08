use std::time::{Duration, Instant};

use serde_json::json;
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    FilesystemMediaAdmission, PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, RecordAppendBatch,
};
use worth_store_physical_backend::{
    FilesystemAccessPosture, MediaFaultDirective, MediaOperationRole,
};

use super::process_protocol::{emit, Request};

pub(super) fn run(request: &Request) {
    let admission =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let gate = authority.pause_gate();
    let activation = authority.one_shot_activation();
    let schedule = authority
        .schedule(vec![authority
            .rule(
                MediaOperationRole::PositionedWrite,
                1,
                MediaFaultDirective::PauseBefore(gate.clone()),
            )
            .for_next_identified_operation_after_activation(
                activation.clone(),
            )])
        .unwrap();
    let (serving, placement) =
        super::open_store::open(&request.root, admission.with_fault_schedule(schedule));
    assert!(serving.physical_recovery_obligations().is_empty());
    activation.arm().unwrap();
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([0xD9; 32]))
        .unwrap();
    let mutation = PhysicalMutationRequest::platform_durable(
        key,
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    );
    let prepared = match submission
        .prepare_durable_append(
            RecordAppendBatch::try_from_iter([b"C9 pending obligation target".as_slice()]).unwrap(),
            placement,
            mutation,
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        _ => panic!("ordinary PW producer mutation did not prepare"),
    };
    let handle = prepared.start();
    let deadline = Instant::now() + Duration::from_secs(10);
    while gate.reached_context().is_none() {
        assert!(
            Instant::now() < deadline,
            "target effect did not reach admitted backend gate"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
    let context = gate.reached_context().unwrap();
    assert_eq!(context.role(), MediaOperationRole::PositionedWrite);
    assert!(context.operation().is_some());
    assert!(activation.is_consumed());
    let manifest = super::artifact_manifest::PendingArtifact::observe(&request.root);
    assert_eq!(context.store().unwrap().bytes(), manifest.store);
    assert_eq!(context.runtime_incarnation(), Some(manifest.runtime));
    emit(
        request,
        json!({"store": super::super::process_execution::hex(serving.store_identity().bytes()),
            "boundary": "before-identified-target-positioned-write-after-journal-sync",
            "matching_target_operations": activation.matching_operation_count(),
            "target_bytes": context.requested_bytes(),
            "target_offset": context.requested_offset(),
            "pending": manifest,
        }),
    );
    // Retain both owners. Only the parent OS kill ends this producer; there is
    // deliberately no close, drop, settlement, or journal edit on this path.
    let _owners = (serving, gate, handle);
    loop {
        std::thread::park();
    }
}
