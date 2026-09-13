use worth_store_physical_format::{
    BootstrapCatalog, CurrentRootCatalogEntry, CurrentRootCatalogGeneration, RecordArtifactFile,
    RecordFrameCoordinate,
};
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalByteRange};

use super::super::{
    load::ResidentAdmissionContext, root_protocol::admit_resident_bootstrap_catalog,
};
use super::support::{format, lifecycle, loaded_frame, store};
use crate::physical_runtime::ResidentAdmissionCounterCells;

#[test]
fn corrupt_bootstrap_resident_rejects_before_owner_projection() {
    let store = store(81);
    let format = format();
    let mut bytes = catalog(store, format);
    bytes[44] ^= 0x80;
    let (_pool, _allocation, lease) = loaded_catalog(store, &bytes);
    let counters = ResidentAdmissionCounterCells::default();

    assert!(admit_resident_bootstrap_catalog(
        &lease,
        scope(store, format, bytes.len()),
        ResidentAdmissionContext::new(lifecycle().observation_state(), &counters),
    )
    .is_err());
    assert_rejected_before_projection(&counters);
}

#[test]
fn wrong_scope_bootstrap_resident_rejects_before_owner_projection() {
    let source_store = store(82);
    let expected_store = store(83);
    let format = format();
    let bytes = catalog(source_store, format);
    let (_pool, _allocation, lease) = loaded_catalog(source_store, &bytes);
    let counters = ResidentAdmissionCounterCells::default();

    assert!(admit_resident_bootstrap_catalog(
        &lease,
        scope(expected_store, format, bytes.len()),
        ResidentAdmissionContext::new(lifecycle().observation_state(), &counters),
    )
    .is_err());
    let observed = counters.snapshot();
    assert_eq!(observed.fresh_validations(), 0);
    assert_eq!(observed.refusals_before_owner_entry(), 1);
    assert_eq!(observed.failed_rechecks_after_owner_entry(), 0);
    assert_eq!(observed.owner_projection_entries(), 0);
    assert_eq!(observed.owner_decoder_entries(), 0);
}

#[test]
fn admitted_bootstrap_projects_without_raw_owner_decoder_entry() {
    let store = store(84);
    let format = format();
    let bytes = catalog(store, format);
    let (_pool, _allocation, lease) = loaded_catalog(store, &bytes);
    let lifecycle = lifecycle();
    let counters = ResidentAdmissionCounterCells::default();
    let context = ResidentAdmissionContext::new(lifecycle.observation_state(), &counters);
    let admitted = admit_resident_bootstrap_catalog(
        &lease,
        scope(store, format, bytes.len()),
        context.clone(),
    )
    .unwrap();
    let projection = admitted.project(context).unwrap();

    assert_eq!(projection.record_format, format);
    assert_eq!(projection.current_root.generation().get(), 1);
    assert!(lease.integrity_validation().is_some());
    let observed = counters.snapshot();
    assert_eq!(observed.fresh_validations(), 1);
    assert_eq!(observed.owner_projection_entries(), 1);
    assert_eq!(observed.owner_decoder_entries(), 0);
}

fn assert_rejected_before_projection(counters: &ResidentAdmissionCounterCells) {
    let observed = counters.snapshot();
    assert_eq!(observed.fresh_validations(), 1);
    assert_eq!(observed.refusals_before_owner_entry(), 1);
    assert_eq!(observed.failed_rechecks_after_owner_entry(), 0);
    assert_eq!(observed.owner_projection_entries(), 0);
    assert_eq!(observed.owner_decoder_entries(), 0);
}

fn loaded_catalog(
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    bytes: &[u8],
) -> (
    worth_store_buffer_pool::PhysicalResidencyPool,
    worth_store_buffer_pool::OperationAllocationGrant,
    worth_store_buffer_pool::PhysicalFrameLease,
) {
    loaded_frame(
        store,
        RecordFrameCoordinate::new(RecordArtifactFile::BootstrapCatalog, 0, bytes.len() as u32)
            .unwrap(),
        bytes,
    )
}

fn catalog(
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
) -> Vec<u8> {
    BootstrapCatalog::new(
        store,
        format,
        CurrentRootCatalogEntry::new(CurrentRootCatalogGeneration::new(1).unwrap()),
    )
    .encode()
    .to_vec()
}

fn scope(
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    length: usize,
) -> PhysicalArtifactScope {
    PhysicalArtifactScope::bootstrap_catalog(
        store,
        format,
        PhysicalByteRange::new(0, length as u64).unwrap(),
    )
}
