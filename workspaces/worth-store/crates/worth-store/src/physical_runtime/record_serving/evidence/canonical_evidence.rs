use worth_foundational::{
    prepare_canonical_basis_sequence, CanonicalBasisConstructionDenial, CanonicalBasisDomain,
    CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisReadyArtifact,
    CanonicalBasisValue, CanonicalIntegerWidth, CanonicalizationRuleVersion,
};
use worth_proof::TransitionOutcome;
use worth_store_physical_format::CurrentPhysicalRecordPlacement;

const DOMAIN: CanonicalBasisDomain = CanonicalBasisDomain::Future("store.physical.record-topology");
const FIELD: CanonicalBasisEntryKind = CanonicalBasisEntryKind::Field;

pub type RecordTopologyCanonicalBasisOutcome =
    TransitionOutcome<CanonicalBasisReadyArtifact, CanonicalBasisConstructionDenial>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordCanonicalObservationDenial {
    ManifestUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalRecordPublicationSummary {
    pub(in crate::physical_runtime::record_serving) store_identity: [u8; 16],
    pub(in crate::physical_runtime::record_serving) format_identity: [u8; 10],
    pub(in crate::physical_runtime::record_serving) root_generation: u64,
    pub(in crate::physical_runtime::record_serving) tree_identity: u64,
    pub(in crate::physical_runtime::record_serving) node_capacity: u16,
    pub(in crate::physical_runtime::record_serving) routing_level: Option<u16>,
    pub(in crate::physical_runtime::record_serving) placements: Vec<CanonicalPlacement>,
    pub(in crate::physical_runtime::record_serving) segment_pages: Vec<CanonicalSegmentPage>,
    pub(in crate::physical_runtime::record_serving) free_space: Vec<CanonicalFreeSpace>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::physical_runtime::record_serving) struct CanonicalPlacement {
    pub(in crate::physical_runtime::record_serving) epoch: [u8; 16],
    pub(in crate::physical_runtime::record_serving) ordinal: u64,
    pub(in crate::physical_runtime::record_serving) class: u8,
    pub(in crate::physical_runtime::record_serving) owner: u64,
    pub(in crate::physical_runtime::record_serving) secondary: u64,
    pub(in crate::physical_runtime::record_serving) owner_generation: u64,
    pub(in crate::physical_runtime::record_serving) secondary_generation: u64,
    pub(in crate::physical_runtime::record_serving) slot_generation: u64,
    pub(in crate::physical_runtime::record_serving) capacity: u64,
    pub(in crate::physical_runtime::record_serving) payload_bytes: u64,
    pub(in crate::physical_runtime::record_serving) arena_route: Option<[u64; 3]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::physical_runtime::record_serving) struct CanonicalSegmentPage {
    pub(in crate::physical_runtime::record_serving) segment: u64,
    pub(in crate::physical_runtime::record_serving) page: u64,
    pub(in crate::physical_runtime::record_serving) page_generation: u64,
    pub(in crate::physical_runtime::record_serving) data_generation: u64,
    pub(in crate::physical_runtime::record_serving) data_page_count: u64,
    pub(in crate::physical_runtime::record_serving) frame_index: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::physical_runtime::record_serving) struct CanonicalFreeSpace {
    pub(in crate::physical_runtime::record_serving) class: u8,
    pub(in crate::physical_runtime::record_serving) owner: u64,
    pub(in crate::physical_runtime::record_serving) first_unallocated: u64,
    pub(in crate::physical_runtime::record_serving) unallocated_count: u64,
    pub(in crate::physical_runtime::record_serving) generation: u64,
}

pub fn lower_record_publication_canonical_basis(
    summary: &PhysicalRecordPublicationSummary,
) -> RecordTopologyCanonicalBasisOutcome {
    lower_normalized(summary)
}

fn lower_normalized(
    summary: &PhysicalRecordPublicationSummary,
) -> RecordTopologyCanonicalBasisOutcome {
    let mut entries = vec![
        entry(
            "store.identity",
            CanonicalBasisValue::UuidBytes(summary.store_identity),
        ),
        entry(
            "format.identity",
            CanonicalBasisValue::ExactText(hex(&summary.format_identity).into()),
        ),
        unsigned("root.generation", summary.root_generation),
        unsigned("root.tree_identity", summary.tree_identity),
        unsigned("root.node_capacity", u64::from(summary.node_capacity)),
        unsigned(
            "root.routing_level",
            summary.routing_level.map_or(0, u64::from),
        ),
    ];
    for placement in &summary.placements {
        let prefix = format!("record.{}.{}", hex(&placement.epoch), placement.ordinal);
        entries.extend([
            unsigned(format!("{prefix}.class"), u64::from(placement.class)),
            unsigned(format!("{prefix}.owner"), placement.owner),
            unsigned(format!("{prefix}.secondary"), placement.secondary),
            unsigned(
                format!("{prefix}.owner_generation"),
                placement.owner_generation,
            ),
            unsigned(
                format!("{prefix}.secondary_generation"),
                placement.secondary_generation,
            ),
            unsigned(
                format!("{prefix}.slot_generation"),
                placement.slot_generation,
            ),
            unsigned(format!("{prefix}.capacity"), placement.capacity),
            unsigned(format!("{prefix}.payload_bytes"), placement.payload_bytes),
        ]);
        if let Some([arena, offset, length]) = placement.arena_route {
            entries.extend([
                unsigned(format!("{prefix}.arena"), arena),
                unsigned(format!("{prefix}.arena_offset"), offset),
                unsigned(format!("{prefix}.arena_length"), length),
            ]);
        }
    }
    for page in &summary.segment_pages {
        let prefix = format!("segment.{}.page.{}", page.segment, page.page);
        entries.extend([
            unsigned(format!("{prefix}.page_generation"), page.page_generation),
            unsigned(format!("{prefix}.data_generation"), page.data_generation),
            unsigned(format!("{prefix}.data_page_count"), page.data_page_count),
            unsigned(format!("{prefix}.frame_index"), page.frame_index),
        ]);
    }
    for free in &summary.free_space {
        let prefix = format!(
            "free.{}.{}.{}",
            free.class, free.owner, free.first_unallocated
        );
        entries.extend([
            unsigned(
                format!("{prefix}.first_unallocated"),
                free.first_unallocated,
            ),
            unsigned(
                format!("{prefix}.unallocated_count"),
                free.unallocated_count,
            ),
            unsigned(format!("{prefix}.generation"), free.generation),
        ]);
    }
    prepare_canonical_basis_sequence(
        CanonicalizationRuleVersion::new("store.physical.record-topology.v2").unwrap(),
        DOMAIN,
        entries,
    )
}

pub(in crate::physical_runtime::record_serving) fn runtime_placement(
    value: CurrentPhysicalRecordPlacement,
) -> CanonicalPlacement {
    match value {
        CurrentPhysicalRecordPlacement::Inline(value) => CanonicalPlacement {
            epoch: value.record().allocation_epoch(),
            ordinal: value.record().ordinal(),
            class: 1,
            owner: value.segment().get(),
            secondary: value.page().get(),
            owner_generation: value.segment_generation(),
            secondary_generation: value.page_generation(),
            slot_generation: value.slot_generation(),
            capacity: u64::from(value.segment_page_capacity()),
            payload_bytes: value.payload_bytes(),
            arena_route: None,
        },
        CurrentPhysicalRecordPlacement::Extent(value) => CanonicalPlacement {
            epoch: value.record().allocation_epoch(),
            ordinal: value.record().ordinal(),
            class: 2,
            owner: value.extent().get(),
            secondary: 0,
            owner_generation: value.extent_generation(),
            secondary_generation: 0,
            slot_generation: 0,
            capacity: 0,
            payload_bytes: value.payload_bytes(),
            arena_route: Some([
                value.arena_range().arena().get(),
                value.arena_range().offset(),
                value.arena_range().length(),
            ]),
        },
    }
}

fn unsigned(locus: impl Into<String>, value: u64) -> CanonicalBasisEntry {
    entry(
        locus,
        CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: u128::from(value),
        },
    )
}

fn entry(locus: impl Into<String>, value: CanonicalBasisValue) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        DOMAIN,
        CanonicalBasisLocus::Named(locus.into().into()),
        FIELD,
        value,
    )
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
