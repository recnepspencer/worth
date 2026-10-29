use {worth_store_physical_backend::preserve_secure_io_for_backend_completion, worth_store_physical_backend::BackendQueueExecutionAdaptation, worth_store_physical_backend::BackendQueueExecutionCompletion, worth_store_physical_backend::BackendQueueExecutionPosture, worth_store_physical_backend::BackendQueueSpeculativeScope, worth_store_physical_backend::BackendSecureIoPreservationDenial};

use {crate::support::backend_witness, crate::support::scheduler_security_scope, crate::support::secure_backend_binding};

#[test]
fn backend_secure_io_preservation_rejects_wrong_read_ahead_scope() {
    let scope = scheduler_security_scope().permission().identity();
    let binding = secure_backend_binding(scope);
    let witness = backend_witness();
    let posture = worth_store_physical_backend::BackendQueueExecutionPosture::from_admitted_capability(
        &witness,
        worth_store_physical_backend::BackendQueueExecutionAdaptation::None,
    )
    .expect("backend posture should admit");
    let wrong_scope = worth_store_physical_backend::BackendQueueSpeculativeScope::admitted(
        scope,
        worth_store_security::StoreTenantScope::RepairBlastRadius,
        scope.key_scope(),
    );
    let completion = worth_store_physical_backend::BackendQueueExecutionCompletion::for_certification(binding, posture)
        .observe_read_ahead(1, wrong_scope);

    let denial = preserve_secure_io_for_backend_completion(completion)
        .expect_err("backend must reject cross-scope read-ahead observation");

    assert_eq!(
        denial,
        worth_store_physical_backend::BackendSecureIoPreservationDenial::ReadAheadScopeMismatch
    );
}
