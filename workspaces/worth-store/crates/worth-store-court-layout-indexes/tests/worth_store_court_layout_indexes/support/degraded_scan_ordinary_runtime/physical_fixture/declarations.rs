pub(crate) use {worth_store_contracts::AcceptedHandoffReadiness, worth_store_contracts::HandoffEvidenceDigestSet, worth_store_contracts::StableDigest, worth_store_contracts::ROADMAP_2_S1_SCOPE};
pub(crate) use {worth_store_layout_indexes::bootstrap_catalog, worth_store_layout_indexes::BootstrapCatalogReadAdmission, worth_store_layout_indexes::BootstrapOnlyAccessPath};
pub(crate) use {worth_store_physical_format::physical_bootstrap_catalog, worth_store_physical_format::InMemoryPhysicalFormatModel, worth_store_physical_format::InMemoryPhysicalFormatModelRequest, worth_store_physical_format::PhysicalExtentId, worth_store_physical_format::PhysicalGeneration, worth_store_physical_format::PhysicalGenerationAuthority, worth_store_physical_format::PhysicalPageId, worth_store_physical_format::PhysicalRecordSlot, worth_store_physical_format::PhysicalSegmentId, worth_store_physical_format::PlatformPhysicalAppendRequest};

pub(crate) fn admitted_catalog() -> BootstrapCatalogReadAdmission {
    admitted_catalog_from_publication(published_layout(false))
}

pub(crate) fn advanced_catalog() -> BootstrapCatalogReadAdmission {
    admitted_catalog_from_publication(published_layout(true))
}

pub(crate) fn open_runtime() -> InMemoryPhysicalFormatModel {
    open_runtime_for_store(
        &worth_store_physical_format::PhysicalStoreIdentity::physical_format_default(),
    )
}

pub(crate) fn open_runtime_for_store(
    store_identity: &worth_store_physical_format::PhysicalStoreIdentity,
) -> InMemoryPhysicalFormatModel {
    worth_store_physical_format::InMemoryPhysicalFormatModel::start_empty_model(
        readiness(),
        worth_store_physical_format::InMemoryPhysicalFormatModelRequest::physical_format_for_store(store_identity.clone()),
    )
    .expect("open degraded scan physical fixture")
}

pub(crate) fn foreign_store_identity() -> worth_store_physical_format::PhysicalStoreIdentity {
    let key = worth_foundational::aspects()
        .vocabulary()
        .key("store.physical.foreign_instance")
        .unwrap();
    worth_store_physical_format::PhysicalStoreIdentity::from_aspect_identity(
        worth_store_aspect_native::StoreAspectIdentity::from_aspect_key(key),
    )
}

pub(crate) fn admitted_catalog_from_publication(
    published: worth_store_physical_format::PlatformPhysicalRootPublicationReport,
) -> BootstrapCatalogReadAdmission {
    let open = published.admit_bootstrap_open_witness().unwrap();
    let discovered = physical_bootstrap_catalog()
        .discover_catalog(&open)
        .unwrap();
    let current_root = physical_bootstrap_catalog()
        .discover_catalog(&open)
        .unwrap()
        .current_root();
    bootstrap_catalog()
        .read_catalog(
            worth_store_layout_indexes::BootstrapOnlyAccessPath::fixed_bootstrap_access_path(),
            discovered,
            current_root,
        )
        .unwrap()
        .1
}

pub(crate) fn published_layout(
    advance_root: bool,
) -> worth_store_physical_format::PlatformPhysicalRootPublicationReport {
    let mut runtime = worth_store_physical_format::InMemoryPhysicalFormatModel::start_empty_model(
        readiness(),
        worth_store_physical_format::InMemoryPhysicalFormatModelRequest::physical_format_canonical(),
    )
    .unwrap();
    let generations = worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format();
    runtime
        .append_physical_record(worth_store_physical_format::PlatformPhysicalAppendRequest::page_slot(
            generations
                .slot_cell(segment(7), page(3), slot(1))
                .with_slot_generation(generation(2)),
            b"small",
        ))
        .unwrap();
    runtime
        .append_physical_record(worth_store_physical_format::PlatformPhysicalAppendRequest::extent(
            generations
                .extent_cell(segment(7), worth_store_physical_format::PhysicalExtentId::from_raw(20).unwrap())
                .with_extent_generation(generation(3)),
            b"large",
        ))
        .unwrap();
    let published = runtime.publish_physical_root().unwrap();
    if advance_root {
        runtime.publish_physical_root().unwrap()
    } else {
        published
    }
}

pub(crate) fn readiness() -> AcceptedHandoffReadiness {
    let digest = |value| worth_store_contracts::StableDigest::new(format!("sha256:{value}")).unwrap();
    worth_store_contracts::AcceptedHandoffReadiness::from_foundational_handoff_artifacts(
        ROADMAP_2_S1_SCOPE,
        worth_store_contracts::HandoffEvidenceDigestSet::new(
            digest("backend"),
            digest("deferred"),
            digest("harness"),
            digest("terms"),
            digest("audit"),
            digest("complexity"),
            digest("provenance"),
        ),
    )
    .unwrap()
}

pub(crate) fn segment(value: u64) -> PhysicalSegmentId {
    worth_store_physical_format::PhysicalSegmentId::from_raw(value).unwrap()
}
pub(crate) fn page(value: u64) -> PhysicalPageId {
    worth_store_physical_format::PhysicalPageId::from_raw(value).unwrap()
}
pub(crate) fn slot(value: u16) -> PhysicalRecordSlot {
    worth_store_physical_format::PhysicalRecordSlot::from_raw(value).unwrap()
}
pub(crate) fn generation(value: u64) -> PhysicalGeneration {
    worth_store_physical_format::PhysicalGeneration::from_raw(value).unwrap()
}