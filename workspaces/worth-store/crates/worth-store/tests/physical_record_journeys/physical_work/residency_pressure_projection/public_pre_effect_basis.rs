use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy,
    PhysicalOperationAllocationScope, PhysicalRecordInitialization, PhysicalRecordOpen,
    PhysicalRecordResidencyPolicy, PhysicalResidencyAllocationSnapshot,
    PhysicalResidencyCounterSnapshot, PhysicalResidencyDimension, PhysicalResidencyRetryPosture,
    PhysicalSpeculativeWorkKind, RecordAppendBatch, RecordByteLimit, RecordReadDenial,
    RecordReadLimits,
};

use super::super::super::durable_publication;
use super::super::{configuration, media, success};
use super::{nonzero, nonzero_count};

#[test]
fn public_read_pressure_retains_exact_pre_effect_basis() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("public-residency-pressure");
    let (format, placement, access) = configuration();
    let seeded = success(initialize_record_store!(media(&root), |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    let published = durable_publication::publish_single(
        &seeded,
        placement,
        durable_publication::certification_material("public-read-pressure", 1),
        RecordAppendBatch::try_from_iter([b"first".as_slice(), b"second".as_slice()]).unwrap(),
    );
    let member = &published.settled_members()[0];
    let first_record = member.record_id(0).unwrap();
    let second_record = member.record_id(1).unwrap();
    assert!(!seeded.close().residency().requires_inspection());

    let policy = one_page_operation_policy(format);
    let serving = success(open_record_store!(media(&root), |durability| {
        PhysicalRecordOpen::new(format, access, durability).with_residency_policy(policy)
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
            .for_dimension(PhysicalResidencyDimension::MetadataBytes)
            .active_units(),
        counters.metadata_bytes(),
    );
    let page_bytes = u64::from(format.declaration().page_size().bytes());
    let read_limits = RecordReadLimits::new(RecordByteLimit::new(16).unwrap());

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
    assert_eq!(read_error.denial(), RecordReadDenial::PhysicalPressure);
    let read_pressure = read_error.pressure().unwrap();
    assert_eq!(read_pressure.basis().store_identity(), store);
    assert_eq!(read_pressure.basis().record(), Some(second_record));
    assert_eq!(read_pressure.store_generation(), generation);
    assert_eq!(
        read_pressure.scope(),
        PhysicalOperationAllocationScope::ForegroundRead
    );
    assert_eq!(
        read_pressure.dimension(),
        PhysicalResidencyDimension::OperationScope(
            PhysicalOperationAllocationScope::ForegroundRead,
        )
    );
    assert_eq!(read_pressure.requested(), page_bytes);
    assert_eq!(read_pressure.admitted(), page_bytes);
    assert_eq!(read_pressure.limit(), page_bytes);
    assert_eq!(
        read_pressure.retry_posture(),
        PhysicalResidencyRetryPosture::AfterAllocationRelease
    );
    assert!(!read_pressure.effect_may_have_started());
    assert_eq!(serving.media_counters(), before_read_denial);
    drop(held);

    assert!(!serving.close().residency().requires_inspection());
}

fn one_page_operation_policy(
    format: AdmittedPhysicalRecordFormat,
) -> AdmittedPhysicalRecordResidencyPolicy {
    use PhysicalOperationAllocationScope as Scope;
    use PhysicalSpeculativeWorkKind as Kind;

    let page = u64::from(format.declaration().page_size().bytes());
    let metadata = 16 * 1024;
    let resident = page * 4;
    let mut builder = PhysicalRecordResidencyPolicy::builder()
        .total_bytes(nonzero(metadata + resident + page + page))
        .resident_bytes(nonzero(resident))
        .metadata_bytes(nonzero(metadata))
        .frame_entries(nonzero_count(4))
        .pinned_frames(nonzero_count(4))
        .pin_leases(nonzero_count(4))
        .dirty_frames(nonzero_count(2))
        .dirty_replacement_bytes(nonzero(page))
        .operation_bytes(nonzero(page));
    for scope in [
        Scope::ForegroundRead,
        Scope::ForegroundWrite,
        Scope::Recovery,
        Scope::Scrub,
        Scope::Maintenance,
        Scope::Verification,
        Scope::Blob,
    ] {
        builder = builder.scope_bytes(scope, nonzero(page));
    }
    for kind in [Kind::ReadAhead, Kind::Prefetch, Kind::WriteBehind] {
        builder = builder.speculative_frames(kind, nonzero_count(2));
    }
    builder.admit(format).into_result().unwrap()
}
