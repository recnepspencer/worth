use worth_store::physical_runtime::{
    PhysicalReadProtectionDenial, PhysicalRecordId, RecordReadLimits, RecordReadSession,
    ServingPhysicalRuntime,
};

fn detached_session(
    serving: &ServingPhysicalRuntime,
    record: PhysicalRecordId,
    limits: RecordReadLimits,
) -> Result<RecordReadSession, PhysicalReadProtectionDenial> {
    let reader = serving.records()?;
    let root = reader.protected_root();
    let session = reader.open(record, limits).expect("admitted record");
    drop(reader);
    assert_eq!(
        serving
            .read_protection_observer()
            .acquisitions_for_root(root),
        1
    );
    Ok(session)
}

fn main() {}
