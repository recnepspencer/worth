pub(crate) use {
    worth_store_contracts::AcceptedHandoffReadiness,
    worth_store_contracts::DurableArtifactFamilyId,
    worth_store_contracts::HandoffEvidenceDigestSet, worth_store_contracts::StableDigest,
    worth_store_contracts::ROADMAP_2_S1_SCOPE,
};
pub(crate) use {
    worth_store_layout_indexes::bootstrap_catalog,
    worth_store_layout_indexes::declarations::layout_declarations,
    worth_store_layout_indexes::BootstrapOnlyAccessDenied,
    worth_store_layout_indexes::BootstrapOnlyAccessPath,
};
pub(crate) use {
    worth_store_physical_certification::FixtureCapabilityDeclaration,
    worth_store_physical_certification::FixtureMutationBoundary,
    worth_store_physical_certification::LargeStoreFixtureProfile,
    worth_store_physical_certification::PhysicalFixtureBuilder,
    worth_store_physical_certification::ProductionBackedFixtureMaterialization,
};
pub(crate) use {
    worth_store_physical_format::physical_bootstrap_catalog,
    worth_store_physical_format::InMemoryPhysicalFormatModel,
    worth_store_physical_format::InMemoryPhysicalFormatModelRequest,
    worth_store_physical_format::PhysicalExtentId, worth_store_physical_format::PhysicalGeneration,
    worth_store_physical_format::PhysicalGenerationAuthority,
    worth_store_physical_format::PhysicalPageId, worth_store_physical_format::PhysicalRecordSlot,
    worth_store_physical_format::PhysicalRootReference,
    worth_store_physical_format::PhysicalSegmentId,
    worth_store_physical_format::PlatformPhysicalAppendRequest,
};

pub(crate) fn published_layout(
) -> worth_store_physical_format::PlatformPhysicalRootPublicationReport {
    let mut facade = worth_store_physical_format::InMemoryPhysicalFormatModel::start_empty_model(
        readiness(),
        worth_store_physical_format::InMemoryPhysicalFormatModelRequest::physical_format_canonical(
        ),
    )
    .expect("open S.1 facade");
    let generations =
        worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format();
    facade
        .append_physical_record(
            worth_store_physical_format::PlatformPhysicalAppendRequest::page_slot(
                generations
                    .slot_cell(segment(7), page(3), slot(1))
                    .with_slot_generation(generation(2)),
                b"small",
            ),
        )
        .expect("append page-backed record");
    facade
        .append_physical_record(
            worth_store_physical_format::PlatformPhysicalAppendRequest::extent(
                generations
                    .extent_cell(
                        segment(7),
                        worth_store_physical_format::PhysicalExtentId::from_raw(20).unwrap(),
                    )
                    .with_extent_generation(generation(3)),
                b"large",
            ),
        )
        .expect("append extent-backed record");
    facade
        .publish_physical_root()
        .expect("publish physical root")
}

pub(crate) fn republished_layout(
) -> worth_store_physical_format::PlatformPhysicalRootPublicationReport {
    let mut facade = worth_store_physical_format::InMemoryPhysicalFormatModel::start_empty_model(
        readiness(),
        worth_store_physical_format::InMemoryPhysicalFormatModelRequest::physical_format_canonical(
        ),
    )
    .expect("open S.1 facade");
    let generations =
        worth_store_physical_format::PhysicalGenerationAuthority::for_canonical_physical_format();
    facade
        .append_physical_record(
            worth_store_physical_format::PlatformPhysicalAppendRequest::page_slot(
                generations
                    .slot_cell(segment(7), page(3), slot(1))
                    .with_slot_generation(generation(2)),
                b"small",
            ),
        )
        .expect("append initial page-backed record");
    facade
        .append_physical_record(
            worth_store_physical_format::PlatformPhysicalAppendRequest::extent(
                generations
                    .extent_cell(
                        segment(7),
                        worth_store_physical_format::PhysicalExtentId::from_raw(20).unwrap(),
                    )
                    .with_extent_generation(generation(3)),
                b"large",
            ),
        )
        .expect("append initial extent-backed record");
    facade
        .publish_physical_root()
        .expect("publish initial physical root");
    facade
        .append_physical_record(
            worth_store_physical_format::PlatformPhysicalAppendRequest::page_slot(
                generations
                    .slot_cell(segment(8), page(4), slot(2))
                    .with_slot_generation(generation(4)),
                b"second",
            ),
        )
        .expect("append second page-backed record");
    facade
        .publish_physical_root()
        .expect("republish physical root")
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
