pub(crate) use worth_store_contracts::{
    AcceptedHandoffReadiness, HandoffEvidenceDigestSet, StableDigest, ROADMAP_2_S1_SCOPE,
};
pub(crate) use worth_store_physical_format::{
    AllocationClassKind, InMemoryPhysicalFormatModel, InMemoryPhysicalFormatModelDenialKind,
    InMemoryPhysicalFormatModelRequest, PhysicalExtentId, PhysicalFreeSpaceSearchPolicy,
    PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageId, PhysicalRecordSlot,
    PhysicalSegmentId, PlatformPhysicalAppendRequest,
};

pub(crate) fn open_facade() -> InMemoryPhysicalFormatModel {
    worth_store_physical_format::InMemoryPhysicalFormatModel::start_empty_model(
        readiness(),
        worth_store_physical_format::InMemoryPhysicalFormatModelRequest::physical_format_canonical(
        ),
    )
    .expect("open S.1 facade")
}

pub(crate) fn readiness() -> AcceptedHandoffReadiness {
    worth_store_contracts::AcceptedHandoffReadiness::from_foundational_handoff_artifacts(
        ROADMAP_2_S1_SCOPE,
        digest_set(),
    )
    .expect("S.1 handoff readiness")
}

pub(crate) fn digest_set() -> HandoffEvidenceDigestSet {
    worth_store_contracts::HandoffEvidenceDigestSet::new(
        digest("backend"),
        digest("deferred"),
        digest("harness"),
        digest("terms"),
        digest("audit"),
        digest("complexity"),
        digest("provenance"),
    )
}

pub(crate) fn digest(name: &str) -> StableDigest {
    worth_store_contracts::StableDigest::new(format!("sha256:{name}")).expect("non-empty digest")
}

pub(crate) fn slot_cell(value: u16) -> worth_store_physical_format::SlotGenerationCell {
    worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format()
        .slot_cell(segment(1), page(1), slot(value))
        .with_slot_generation(generation(5))
}

pub(crate) fn extent_cell(value: u64) -> worth_store_physical_format::ExtentGenerationCell {
    worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format()
        .extent_cell(segment(1), extent(value))
        .with_extent_generation(generation(7))
}

pub(crate) fn segment(value: u64) -> PhysicalSegmentId {
    worth_store_physical_format::PhysicalSegmentId::from_raw(value).unwrap()
}

pub(crate) fn page(value: u64) -> PhysicalPageId {
    worth_store_physical_format::PhysicalPageId::from_raw(value).unwrap()
}

pub(crate) fn extent(value: u64) -> PhysicalExtentId {
    worth_store_physical_format::PhysicalExtentId::from_raw(value).unwrap()
}

pub(crate) fn slot(value: u16) -> PhysicalRecordSlot {
    worth_store_physical_format::PhysicalRecordSlot::from_raw(value).unwrap()
}

pub(crate) fn generation(value: u64) -> PhysicalGeneration {
    worth_store_physical_format::PhysicalGeneration::from_raw(value).unwrap()
}
