#[allow(unused_imports)]
use crate::physical_record_journeys::physical_work::scan_continuation_damage::*;
use {std::path::Path, std::time::Duration, std::time::Instant};

use worth_proof::TransitionOutcome;
use {
    worth_store::physical_runtime::PhysicalManifestCapacityTransition,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationPreparationDenial,
    worth_store::physical_runtime::PhysicalSignalAspectBindingDigest,
    worth_store::physical_runtime::PhysicalSignalAspectBindingObservation,
    worth_store::physical_runtime::PhysicalWorkEffectFate,
    worth_store::physical_runtime::PhysicalWorkOperationFamily,
    worth_store::physical_runtime::PhysicalWorkRecoveryDisposition,
    worth_store::physical_runtime::RecordAppendBatch,
    worth_store::physical_runtime::RecordAppendDenial,
    worth_store::physical_runtime::RecordCountLimit,
    worth_store::physical_runtime::RecordReadDenial,
    worth_store::physical_runtime::RecordScanDenial,
    worth_store::physical_runtime::RecordScanOutcome,
    worth_store::physical_runtime::RecordScanRequest,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
use {
    worth_store_physical_backend::MediaCounterSnapshot,
    worth_store_physical_backend::MediaFaultDirective,
    worth_store_physical_backend::MediaOperationRole,
};

use worth_store_testkit_store::durable_publication;
use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization,
};

#[test]
fn continuation_damage_revokes_shared_health_and_fences_mutation() {
    let calibration = calibrate();
    let (parent, serving) = open_faulted(calibration);
    let (before, invalidations_before, causal_start) =
        fail_first_continuation(&serving, calibration);
    await_signal_cleanup(&serving);
    assert_failed_effect(&serving, before, invalidations_before);
    assert_failed_scan_route(&serving, causal_start);
    assert_mutation_fenced(&serving);
    assert!(serving.close_plan().execute().requires_inspection());
    drop(parent);
}
