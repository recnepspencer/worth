#[allow(unused_imports)] use crate::physical_record_journeys::physical_work::residency_pressure_projection::public_pre_effect_basis::*;
use {worth_store::physical_runtime::AdmittedPhysicalRecordFormat, worth_store::physical_runtime::AdmittedPhysicalRecordResidencyPolicy, worth_store::physical_runtime::PhysicalOperationAllocationScope, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalRecordResidencyPolicy, worth_store::physical_runtime::PhysicalResidencyAllocationSnapshot, worth_store::physical_runtime::PhysicalResidencyCounterSnapshot, worth_store::physical_runtime::PhysicalResidencyDimension, worth_store::physical_runtime::PhysicalResidencyRetryPosture, worth_store::physical_runtime::PhysicalSpeculativeWorkKind, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadDenial, worth_store::physical_runtime::RecordReadLimits};

use worth_store_testkit_store::durable_publication;
use {
    crate::physical_record_journeys::physical_work::residency_pressure_projection::nonzero,
    crate::physical_record_journeys::physical_work::residency_pressure_projection::nonzero_count,
};
use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::success,
};

#[test]
fn public_read_pressure_retains_exact_pre_effect_basis() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("public-residency-pressure");
    let (format, placement, access) = configuration();
    let seeded = success(initialize_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )
    }));
    let published = worth_store_testkit_store::durable_publication::root::publish_single(
        &seeded,
        placement,
        worth_store_testkit_store::durable_publication::material::certification_material(
            "public-read-pressure",
            1,
        ),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([
            b"first".as_slice(),
            b"second".as_slice(),
        ])
        .unwrap(),
    );
    let member = &published.settled_members()[0];
    let first_record = member.record_id(0).unwrap();
    let second_record = member.record_id(1).unwrap();
    assert!(!seeded.close().residency().requires_inspection());

    let policy = one_page_foreground_read_policy(format);
    let serving = success(open_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
            .with_residency_policy(policy)
    },));
    let store = serving.store_identity();
    let observation = serving.residency_observation();
    let generation = observation.store_generation();
    let counters: PhysicalResidencyCounterSnapshot = observation.counters();
    let allocations: PhysicalResidencyAllocationSnapshot = observation.allocations();
    assert_eq!(observation.store_identity(), store);
    assert_eq!(observation.admitted_policy(), policy);
    assert_eq!(allocations.store_identity(), store);
    assert_eq!(
        allocations
            .for_dimension(worth_store::physical_runtime::PhysicalResidencyDimension::MetadataBytes)
            .active_units(),
        counters.metadata_bytes(),
    );
    let page_bytes = u64::from(format.declaration().page_size().bytes());
    let read_limits = worth_store::physical_runtime::RecordReadLimits::new(
        worth_store::physical_runtime::RecordByteLimit::new(16).unwrap(),
    );

    let held = serving
        .records()
        .expect("read protection admission")
        .open(first_record, read_limits)
        .unwrap();
    let before_read_denial = serving.media_counters();
    let read_error = match serving
        .records()
        .expect("read protection admission")
        .open(second_record, read_limits)
    {
        Err(error) => error,
        Ok(_) => panic!("the second read must not spend the held read allocation"),
    };
    assert_eq!(
        read_error.denial(),
        worth_store::physical_runtime::RecordReadDenial::PhysicalPressure
    );
    let read_pressure = read_error.pressure().unwrap();
    assert_eq!(read_pressure.basis().store_identity(), store);
    assert_eq!(read_pressure.basis().record(), Some(second_record));
    assert_eq!(read_pressure.store_generation(), generation);
    assert_eq!(
        read_pressure.scope(),
        worth_store::physical_runtime::PhysicalOperationAllocationScope::ForegroundRead
    );
    assert_eq!(
        read_pressure.dimension(),
        worth_store::physical_runtime::PhysicalResidencyDimension::OperationScope(
            worth_store::physical_runtime::PhysicalOperationAllocationScope::ForegroundRead,
        )
    );
    assert_eq!(read_pressure.requested(), page_bytes);
    assert_eq!(read_pressure.admitted(), page_bytes);
    assert_eq!(read_pressure.limit(), page_bytes);
    assert_eq!(
        read_pressure.retry_posture(),
        worth_store::physical_runtime::PhysicalResidencyRetryPosture::AfterAllocationRelease
    );
    assert!(!read_pressure.effect_may_have_started());
    assert_eq!(serving.media_counters(), before_read_denial);
    drop(held);

    assert!(!serving.close().residency().requires_inspection());
}
