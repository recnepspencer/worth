fn small_example(durability: worth_store::physical_runtime::AdmittedPhysicalDurabilityPolicy) {
    use worth_store::physical_runtime::{
        AdmittedPhysicalRecordFormat, PhysicalRecordAccessPolicy, PhysicalRecordFormatDeclaration,
        PhysicalRecordOpen,
    };

    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let request = PhysicalRecordOpen::new(format, access, durability);
    let _ = request;
}

fn real_example(durability: worth_store::physical_runtime::AdmittedPhysicalDurabilityPolicy) {
    use std::num::{NonZeroU32, NonZeroU64};
    use worth_store::physical_runtime::{
        AdmittedPhysicalRecordFormat, PhysicalOperationAllocationScope as Scope,
        PhysicalRecordAccessPolicy, PhysicalRecordFormatDeclaration, PhysicalRecordOpen,
        PhysicalRecordResidencyPolicy, PhysicalSpeculativeWorkKind as Speculation, RecordReadError,
        ServingPhysicalRuntime,
    };

    let bytes = |value| NonZeroU64::new(value).unwrap();
    let frames = |value| NonZeroU32::new(value).unwrap();

    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();

    let residency = PhysicalRecordResidencyPolicy::builder()
        .total_bytes(bytes(65_536))
        .resident_bytes(bytes(16_384))
        .metadata_bytes(bytes(8_192))
        .frame_entries(frames(8))
        .pinned_frames(frames(8))
        .pin_leases(frames(2))
        .dirty_frames(frames(4))
        .dirty_replacement_bytes(bytes(16_384))
        .operation_bytes(bytes(16_384))
        .scope_bytes(Scope::ForegroundRead, bytes(16_384))
        .scope_bytes(Scope::ForegroundWrite, bytes(16_384))
        .scope_bytes(Scope::Recovery, bytes(16_384))
        .scope_bytes(Scope::Scrub, bytes(16_384))
        .scope_bytes(Scope::Maintenance, bytes(16_384))
        .scope_bytes(Scope::Verification, bytes(16_384))
        .scope_bytes(Scope::Blob, bytes(16_384))
        .speculative_frames(Speculation::Prefetch, frames(8))
        .speculative_frames(Speculation::ReadAhead, frames(8))
        .speculative_frames(Speculation::WriteBehind, frames(4))
        .admit(format)
        .into_result()
        .expect("deployment residency policy must fit the admitted format");

    let request =
        PhysicalRecordOpen::new(format, access, durability).with_residency_policy(residency);

    fn inspect_residency(serving: &ServingPhysicalRuntime) {
        let observation = serving.residency_observation();
        let metadata = observation.allocations().for_dimension(
            worth_store::physical_runtime::PhysicalResidencyDimension::MetadataBytes,
        );

        assert_eq!(observation.store_identity(), serving.store_identity());
        assert_eq!(
            metadata.active_units(),
            observation.counters().metadata_bytes(),
        );
    }

    fn inspect_read_pressure(error: &RecordReadError) {
        if let Some(pressure) = error.pressure() {
            eprintln!(
                "residency pressure: scope={:?} dimension={:?} requested={} limit={}",
                pressure.scope(),
                pressure.dimension(),
                pressure.requested(),
                pressure.limit(),
            );
        }
    }
    let _ = request;
}

fn protected_reader_example() {
    use worth_store::physical_runtime::{
        PhysicalRecordId, RecordReadLimits, ServingPhysicalRuntime,
    };

    fn read_across_publication(
        serving: &ServingPhysicalRuntime,
        record: PhysicalRecordId,
        limits: RecordReadLimits,
    ) {
        let reader = serving.records().expect("read protection admission");
        let root = reader.protected_root();
        let mut session = reader.open(record, limits).expect("record read admission");
        drop(reader); // The session still protects the captured root.
        let chunk = session.next_chunk().expect("stream progress");
        let _ = (root, chunk);
    }
}

fn borrowed_chunk_example() {
    use worth_store::physical_runtime::{RecordReadSession, RecordStreamFailure};

    fn consume(mut session: RecordReadSession) -> Result<(), RecordStreamFailure> {
        while let Some(chunk) = session.next_chunk()? {
            let basis = chunk.basis();
            consume_payload(
                chunk.bytes(),
                chunk.logical_range(),
                basis.record(),
                basis.frame_coordinate(),
            );
        }
        Ok(())
    }
}

fn bounded_copy_example() {
    use worth_store::physical_runtime::{RecordReadSession, RecordStreamFailure};

    fn copy_bounded(
        mut session: RecordReadSession,
        target: &mut [u8],
    ) -> Result<(), RecordStreamFailure> {
        assert!(
            !target.is_empty(),
            "a bounded-copy buffer must distinguish progress from end of record",
        );
        loop {
            let count = session.read_next(target)?;
            if count == 0 {
                break;
            }
            consume_copy(&target[..count]);
        }
        Ok(())
    }
}

fn successor_physical_allocation_example() {
    use std::num::NonZeroU64;
    use worth_store::physical_runtime::{
        PhysicalScopedAllocationFailure, RecoveryPhysicalAllocation, ServingPhysicalRuntime,
    };

    fn admit_recovery_bytes<'runtime>(
        runtime: &'runtime ServingPhysicalRuntime,
        bytes: NonZeroU64,
    ) -> Result<RecoveryPhysicalAllocation<'runtime>, PhysicalScopedAllocationFailure> {
        runtime.physical_allocations().admit_recovery(bytes)
    }
}

fn consume_payload<A, B, C, D>(_: A, _: B, _: C, _: D) {}

fn consume_copy(_: &[u8]) {}

pub(crate) fn run_configuration_examples() {
    let _ = (small_example, real_example);
}

fn main() {
    let _ = (
        protected_reader_example,
        borrowed_chunk_example,
        bounded_copy_example,
        successor_physical_allocation_example,
    );
    run_configuration_examples();
}
