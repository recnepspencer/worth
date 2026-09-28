use worth_foundational::{
    prepare_canonical_basis_sequence, CanonicalBasisConstructionDenial, CanonicalBasisDomain,
    CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisReadyArtifact,
    CanonicalBasisValue, CanonicalIntegerWidth, CanonicalizationRuleVersion,
};
use worth_proof::TransitionOutcome;

use crate::{OfflineAllocationClass, OfflineDurableManifestWalk, OfflineRecordPlacement};

const DOMAIN: CanonicalBasisDomain = CanonicalBasisDomain::Future("store.physical.record-topology");
const FIELD: CanonicalBasisEntryKind = CanonicalBasisEntryKind::Field;

pub type OfflineRecordTopologyCanonicalBasisOutcome =
    TransitionOutcome<CanonicalBasisReadyArtifact, CanonicalBasisConstructionDenial>;

pub fn lower_offline_record_publication_canonical_basis(
    walk: &OfflineDurableManifestWalk,
) -> OfflineRecordTopologyCanonicalBasisOutcome {
    let mut entries = root_entries(walk);
    let mut placements = walk.placements().to_vec();
    placements.sort_unstable_by_key(placement_order);
    for placement in placements {
        append_placement(&mut entries, placement);
    }
    let mut segment_pages = walk.segment_pages().to_vec();
    segment_pages.sort_unstable_by_key(|page| (page.segment(), page.page()));
    for page in segment_pages {
        let prefix = format!("segment.{}.page.{}", page.segment(), page.page());
        entries.extend([
            unsigned(format!("{prefix}.page_generation"), page.page_generation()),
            unsigned(format!("{prefix}.data_generation"), page.data_generation()),
            unsigned(
                format!("{prefix}.data_page_count"),
                u64::from(page.data_page_count()),
            ),
            unsigned(
                format!("{prefix}.frame_index"),
                u64::from(page.frame_index()),
            ),
        ]);
    }
    let mut free_space = walk.free_space().to_vec();
    free_space.sort_unstable_by_key(|entry| {
        (
            allocation_class(entry.class()),
            entry.owner(),
            entry.first_unallocated(),
        )
    });
    for free in free_space {
        let class = allocation_class(free.class());
        let prefix = format!("free.{class}.{}.{}", free.owner(), free.first_unallocated());
        entries.extend([
            unsigned(
                format!("{prefix}.first_unallocated"),
                free.first_unallocated(),
            ),
            unsigned(
                format!("{prefix}.unallocated_count"),
                free.unallocated_count(),
            ),
            unsigned(format!("{prefix}.generation"), free.generation()),
        ]);
    }
    prepare_canonical_basis_sequence(
        CanonicalizationRuleVersion::new("store.physical.record-topology.v2").unwrap(),
        DOMAIN,
        entries,
    )
}

fn root_entries(walk: &OfflineDurableManifestWalk) -> Vec<CanonicalBasisEntry> {
    vec![
        entry(
            "store.identity",
            CanonicalBasisValue::UuidBytes(walk.store_identity()),
        ),
        entry(
            "format.identity",
            CanonicalBasisValue::ExactText(hex(&walk.format_identity()).into()),
        ),
        unsigned("root.generation", walk.root_generation()),
        unsigned("root.tree_identity", walk.tree_identity()),
        unsigned("root.node_capacity", u64::from(walk.node_capacity())),
        unsigned(
            "root.routing_level",
            walk.routing_level().map_or(0, u64::from),
        ),
    ]
}

fn append_placement(entries: &mut Vec<CanonicalBasisEntry>, placement: OfflineRecordPlacement) {
    let fields = PlacementFields::from(placement);
    let prefix = format!(
        "record.{}.{}",
        hex(&fields.allocation_epoch),
        fields.ordinal
    );
    entries.extend([
        unsigned(format!("{prefix}.class"), fields.class),
        unsigned(format!("{prefix}.owner"), fields.owner),
        unsigned(format!("{prefix}.secondary"), fields.secondary),
        unsigned(
            format!("{prefix}.owner_generation"),
            fields.owner_generation,
        ),
        unsigned(
            format!("{prefix}.secondary_generation"),
            fields.secondary_generation,
        ),
        unsigned(format!("{prefix}.slot_generation"), fields.slot_generation),
        unsigned(format!("{prefix}.capacity"), fields.capacity),
        unsigned(format!("{prefix}.payload_bytes"), fields.payload_bytes),
    ]);
    if let OfflineRecordPlacement::Extent {
        arena,
        arena_offset,
        arena_length,
        ..
    } = placement
    {
        entries.extend([
            unsigned(format!("{prefix}.arena"), arena),
            unsigned(format!("{prefix}.arena_offset"), arena_offset),
            unsigned(format!("{prefix}.arena_length"), arena_length),
        ]);
    }
}

struct PlacementFields {
    allocation_epoch: [u8; 16],
    ordinal: u64,
    class: u64,
    owner: u64,
    secondary: u64,
    owner_generation: u64,
    secondary_generation: u64,
    slot_generation: u64,
    capacity: u64,
    payload_bytes: u64,
}

impl From<OfflineRecordPlacement> for PlacementFields {
    fn from(placement: OfflineRecordPlacement) -> Self {
        match placement {
            OfflineRecordPlacement::Inline {
                record,
                segment,
                page,
                segment_generation,
                page_generation,
                slot_generation,
                payload_bytes,
                segment_page_capacity,
                ..
            } => Self {
                allocation_epoch: record.allocation_epoch(),
                ordinal: record.ordinal(),
                class: 1,
                owner: segment,
                secondary: page,
                owner_generation: segment_generation,
                secondary_generation: page_generation,
                slot_generation,
                capacity: u64::from(segment_page_capacity),
                payload_bytes,
            },
            OfflineRecordPlacement::Extent {
                record,
                extent,
                generation,
                payload_bytes,
                ..
            } => Self {
                allocation_epoch: record.allocation_epoch(),
                ordinal: record.ordinal(),
                class: 2,
                owner: extent,
                secondary: 0,
                owner_generation: generation,
                secondary_generation: 0,
                slot_generation: 0,
                capacity: 0,
                payload_bytes,
            },
        }
    }
}

fn placement_order(placement: &OfflineRecordPlacement) -> ([u8; 16], u64) {
    match placement {
        OfflineRecordPlacement::Inline { record, .. }
        | OfflineRecordPlacement::Extent { record, .. } => {
            (record.allocation_epoch(), record.ordinal())
        }
    }
}

const fn allocation_class(class: OfflineAllocationClass) -> u64 {
    match class {
        OfflineAllocationClass::InlinePage => 1,
        OfflineAllocationClass::ExtentArena => 2,
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
