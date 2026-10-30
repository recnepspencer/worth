#[allow(unused_imports)]
use crate::scheduler_queue_execution::secure_io_admission::*;
use worth_store_contracts::QueueProducerResourceShape;
use worth_store_test_support::harness::scheduling::scheduler_foreground_read_work;
use {
    worth_store_io_scheduler::admit_backend_capability_for_scheduler_claim,
    worth_store_io_scheduler::admit_queue_execution_plan,
    worth_store_io_scheduler::admit_secure_io_scope_for_scheduler,
    worth_store_io_scheduler::reject_lower_authority_secure_io_scope_source,
    worth_store_io_scheduler::IoSchedulerBackendCapabilityRequirement,
    worth_store_io_scheduler::QueueExecutionAdmissionRequest,
    worth_store_io_scheduler::QueueWorkDeclaration, worth_store_io_scheduler::SecureIoOperation,
    worth_store_io_scheduler::SecureIoPostureRequirement,
    worth_store_io_scheduler::SecureIoPreservationDenial,
    worth_store_io_scheduler::SecureIoPreservationRequest,
};
use {
    worth_store_security::classify_iam_role_as_security_scope_source,
    worth_store_security::classify_identity_provider_claim_as_security_scope_source,
    worth_store_security::classify_kms_key_id_as_security_scope_source,
    worth_store_security::classify_operator_identity_as_security_scope_source,
    worth_store_security::classify_terminal_json_label_as_security_scope_source,
};

use {
    crate::support::backend_witness, crate::support::point_read_budget,
    crate::support::scheduler_security_scope,
};

#[test]
fn secure_io_receipt_is_required_for_secure_queue_admission() {
    let reservation = worth_store_io_scheduler::foreground_reservation::admitted_point_read_reservation_for_certification_test();
    let budget = point_read_budget();
    let work = read_ahead_work(reservation, budget);
    let backend = admit_backend_capability_for_scheduler_claim(
        &backend_witness(),
        worth_store_io_scheduler::IoSchedulerBackendCapabilityRequirement::DirectIo,
    )
    .expect("direct I/O backend should admit");
    let scope = scheduler_security_scope();
    let secure_io = admit_secure_io_scope_for_scheduler(
        worth_store_io_scheduler::SecureIoPreservationRequest::new(
            worth_store_io_scheduler::SecureIoOperation::ReadAhead,
            &scope,
            &backend,
        )
        .require_posture(worth_store_io_scheduler::SecureIoPostureRequirement::ScopePreserving),
    )
    .expect("scope-preserving direct I/O should admit secure-I/O preservation");
    let work = work.with_secure_io_scope(secure_io);
    let policy = worth_store_io_scheduler::admit_queue_policy_receipt(
        work,
        crate::support::policy_receipt(budget),
    )
    .expect("policy receipt should bind the exact queue work");

    let plan = admit_queue_execution_plan(
        worth_store_io_scheduler::QueueExecutionAdmissionRequest::new(policy, &backend),
    )
    .expect("queue work should preserve admitted secure-I/O scope");

    assert_eq!(plan.work().secure_io(), Some(secure_io));
}
#[test]
fn ordinary_read_ahead_queue_admission_requires_secure_io_receipt() {
    let reservation = worth_store_io_scheduler::foreground_reservation::admitted_point_read_reservation_for_certification_test();
    let budget = point_read_budget();
    let work = read_ahead_work(reservation, budget);
    let backend = admit_backend_capability_for_scheduler_claim(
        &backend_witness(),
        worth_store_io_scheduler::IoSchedulerBackendCapabilityRequirement::DirectIo,
    )
    .expect("direct I/O backend should admit");
    let policy = worth_store_io_scheduler::admit_queue_policy_receipt(
        work,
        crate::support::policy_receipt(budget),
    )
    .expect("policy receipt should bind the exact queue work");

    let denial = admit_queue_execution_plan(
        worth_store_io_scheduler::QueueExecutionAdmissionRequest::new(policy, &backend),
    )
    .expect_err("read-ahead must not admit without secure-I/O preservation");

    assert_eq!(
        denial,
        worth_store_io_scheduler::QueueExecutionAdmissionDenial::MissingSecureIoPreservation
    );
}

#[test]
fn secure_io_receipt_operation_cannot_be_laundered() {
    let reservation = worth_store_io_scheduler::foreground_reservation::admitted_point_read_reservation_for_certification_test();
    let budget = point_read_budget();
    let work = read_ahead_work(reservation, budget);
    let backend = admit_backend_capability_for_scheduler_claim(
        &backend_witness(),
        worth_store_io_scheduler::IoSchedulerBackendCapabilityRequirement::DirectIo,
    )
    .expect("direct I/O backend should admit");
    let scope = scheduler_security_scope();
    let secure_io = admit_secure_io_scope_for_scheduler(
        worth_store_io_scheduler::SecureIoPreservationRequest::new(
            worth_store_io_scheduler::SecureIoOperation::BackgroundLease,
            &scope,
            &backend,
        )
        .require_posture(worth_store_io_scheduler::SecureIoPostureRequirement::ScopePreserving),
    )
    .expect("background secure-I/O receipt should admit");
    let work = work.with_secure_io_scope(secure_io);
    let policy = worth_store_io_scheduler::admit_queue_policy_receipt(
        work,
        crate::support::policy_receipt(budget),
    )
    .expect("policy receipt should bind the exact queue work");

    let denial = admit_queue_execution_plan(
        worth_store_io_scheduler::QueueExecutionAdmissionRequest::new(policy, &backend),
    )
    .expect_err("background receipt must not satisfy read-ahead work");

    assert_eq!(
        denial,
        worth_store_io_scheduler::QueueExecutionAdmissionDenial::SecureIoDenied(
            worth_store_io_scheduler::SecureIoPreservationDenial::OperationMismatch {
                expected: worth_store_io_scheduler::SecureIoOperation::ReadAhead,
                actual: worth_store_io_scheduler::SecureIoOperation::BackgroundLease,
            }
        )
    );
}

#[test]
fn unsupported_secure_io_posture_denies_typed() {
    let backend = admit_backend_capability_for_scheduler_claim(
        &backend_witness(),
        worth_store_io_scheduler::IoSchedulerBackendCapabilityRequirement::DirectIo,
    )
    .expect("direct I/O backend should admit");
    let scope = scheduler_security_scope();

    let denial = admit_secure_io_scope_for_scheduler(
        worth_store_io_scheduler::SecureIoPreservationRequest::new(
            worth_store_io_scheduler::SecureIoOperation::BatchedWrite,
            &scope,
            &backend,
        )
        .require_posture(
            worth_store_io_scheduler::SecureIoPostureRequirement::SecureFrameCompatible,
        ),
    )
    .expect_err("ordinary direct I/O must not satisfy secure-frame posture");

    assert_eq!(
        denial,
        worth_store_io_scheduler::SecureIoPreservationDenial::UnsupportedSecureIoPosture {
            operation: worth_store_io_scheduler::SecureIoOperation::BatchedWrite,
            requirement:
                worth_store_io_scheduler::IoSchedulerBackendCapabilityRequirement::DirectIo,
        }
    );
}

#[test]
fn lower_authority_sources_report_secure_io_classifier_denials() {
    for source in [
        classify_identity_provider_claim_as_security_scope_source(),
        classify_kms_key_id_as_security_scope_source(),
        classify_iam_role_as_security_scope_source(),
        classify_operator_identity_as_security_scope_source(),
        classify_terminal_json_label_as_security_scope_source(),
    ] {
        let denial = reject_lower_authority_secure_io_scope_source(source)
            .expect_err("lower authority source must not admit secure-I/O scope");
        assert!(matches!(
            denial,
            worth_store_io_scheduler::SecureIoPreservationDenial::LowerAuthoritySecurityScopeSource(
                _
            )
        ));
    }
}
