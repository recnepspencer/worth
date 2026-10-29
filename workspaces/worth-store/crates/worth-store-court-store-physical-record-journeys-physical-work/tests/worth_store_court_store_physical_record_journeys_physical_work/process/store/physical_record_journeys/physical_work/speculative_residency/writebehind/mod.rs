#[allow(unused_imports)] use crate::physical_record_journeys::physical_work::speculative_residency::writebehind::*;
use {worth_store::physical_runtime::AdmittedDirtyFrame, worth_store::physical_runtime::PhysicalOperationAllocationScope, worth_store::physical_runtime::PhysicalResidencyCertification, worth_store::physical_runtime::PhysicalResidencyDimension, worth_store::physical_runtime::PhysicalSpeculativeWorkKind, worth_store::physical_runtime::PhysicalWorkEffectFate, worth_store::physical_runtime::PhysicalWorkOperationFamily, worth_store::physical_runtime::PhysicalWorkSignalFamily, worth_store::physical_runtime::PhysicalWritebackExecution, worth_store::physical_runtime::PhysicalWritebackFailureCause, worth_store::physical_runtime::PhysicalWritebackSettlement, worth_store::physical_runtime::PreparedPhysicalWriteback};
use {worth_store_buffer_pool::PhysicalResidencyDenial};
use {worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement};
use {worth_store_physical_format::RecordFrameCoordinate};

use {crate::physical_record_journeys::physical_work::speculative_residency::fixture::causal_record, crate::physical_record_journeys::physical_work::speculative_residency::fixture::coordinate, crate::physical_record_journeys::physical_work::speculative_residency::fixture::initialize_store, crate::physical_record_journeys::physical_work::speculative_residency::fixture::open_store_with_writebehind};

#[test]
fn denied_writebehind_retains_dirty_authority_without_work_or_media_and_retries_cleanly() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("writebehind-pressure");
    initialize_store(&root);
    let serving = open_store_with_writebehind(&root, 2, 2, 1);
    let residency = serving.certification_physical_residency();
    let first = dirty_frame(&residency, coordinate(0), 0x61);
    let second = dirty_frame(&residency, coordinate(1), 0x62);
    let first_prepared = residency
        .prepare_writeback(
            first,
            worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
        )
        .unwrap();
    let work_before_denial = serving.physical_work_counters();
    let causal_before_denial = serving.physical_work_observer().causal().records().len();
    let scheduler_before_denial = serving.physical_scheduler_capacity();
    let media_before_denial = serving.media_counters();

    let failure = match residency.prepare_writeback(
        second,
        worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
    ) {
        Err(failure) => failure,
        Ok(_) => panic!("one-past writebehind must be denied before work submission"),
    };
    let pressure = match failure.cause() {
        worth_store::physical_runtime::PhysicalWritebackFailureCause::Residency(worth_store_buffer_pool::PhysicalResidencyDenial::Pressure(pressure)) => {
            pressure
        }
        cause => panic!("one-past writebehind must expose exact residency pressure: {cause:?}"),
    };
    assert_eq!(
        pressure.dimension(),
        worth_store::physical_runtime::PhysicalResidencyDimension::SpeculativeFrames(worth_store::physical_runtime::PhysicalSpeculativeWorkKind::WriteBehind)
    );
    assert_eq!(
        pressure.scope(),
        worth_store::physical_runtime::PhysicalOperationAllocationScope::ForegroundWrite
    );
    assert_eq!(pressure.requested(), 1);
    assert_eq!(pressure.current(), 1);
    assert_eq!(pressure.limit(), 1);
    assert!(!pressure.effect_may_have_started());
    let retained_second = failure.into_dirty();
    assert_eq!(retained_second.coordinate(), coordinate(1));
    assert_eq!(serving.physical_work_counters(), work_before_denial);
    assert_eq!(
        serving.physical_work_observer().causal().records().len(),
        causal_before_denial
    );
    assert_eq!(
        serving.physical_scheduler_capacity(),
        scheduler_before_denial
    );
    assert_eq!(serving.media_counters(), media_before_denial);

    let live = residency.counters();
    assert_eq!(live.dirty_frames(), 2);
    assert_eq!(
        live.speculative_attempts(worth_store::physical_runtime::PhysicalSpeculativeWorkKind::WriteBehind),
        2
    );
    assert_eq!(
        live.speculative_admissions(worth_store::physical_runtime::PhysicalSpeculativeWorkKind::WriteBehind),
        1
    );
    assert_eq!(
        live.speculative_denials(worth_store::physical_runtime::PhysicalSpeculativeWorkKind::WriteBehind),
        1
    );
    assert_eq!(
        live.active_speculative_frames(worth_store::physical_runtime::PhysicalSpeculativeWorkKind::WriteBehind),
        1
    );
    assert_eq!(
        live.peak_speculative_frames(worth_store::physical_runtime::PhysicalSpeculativeWorkKind::WriteBehind),
        1
    );

    let first_settlement = execute_clean(&residency, first_prepared);
    let second_prepared = residency
        .prepare_writeback(
            retained_second,
            worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
        )
        .unwrap();
    let second_settlement = execute_clean(&residency, second_prepared);
    for settlement in [first_settlement, second_settlement] {
        let record = causal_record(&serving, settlement.identity());
        assert_eq!(
            record.operation(),
            worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactRangeWrite
        );
        assert_eq!(
            record.signal_family(),
            worth_store::physical_runtime::PhysicalWorkSignalFamily::ExactWriteback
        );
        assert!(record.backend_operation().is_some());
        assert_eq!(record.effect_fate(), worth_store::physical_runtime::PhysicalWorkEffectFate::WriteCompleted);
        assert_eq!(record.derived_completion(), Some(settlement.signal()));
    }

    let settled = residency.counters();
    assert_eq!(settled.dirty_frames(), 0);
    assert_eq!(
        settled.speculative_attempts(worth_store::physical_runtime::PhysicalSpeculativeWorkKind::WriteBehind),
        3
    );
    assert_eq!(
        settled.speculative_admissions(worth_store::physical_runtime::PhysicalSpeculativeWorkKind::WriteBehind),
        2
    );
    assert_eq!(
        settled.speculative_completions(worth_store::physical_runtime::PhysicalSpeculativeWorkKind::WriteBehind),
        2
    );
    assert_eq!(
        settled.speculative_denials(worth_store::physical_runtime::PhysicalSpeculativeWorkKind::WriteBehind),
        1
    );
    assert_eq!(
        settled.active_speculative_frames(worth_store::physical_runtime::PhysicalSpeculativeWorkKind::WriteBehind),
        0
    );
    assert_eq!(
        settled.peak_speculative_frames(worth_store::physical_runtime::PhysicalSpeculativeWorkKind::WriteBehind),
        1
    );
    assert!(!serving.close().residency().requires_inspection());
}
