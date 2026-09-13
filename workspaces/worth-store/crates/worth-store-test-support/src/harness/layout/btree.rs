use worth_store_contracts::{
    AcceptedHandoffReadiness, HandoffEvidenceDigestSet, StableDigest, ROADMAP_2_S1_SCOPE,
};
use worth_store_layout_indexes::{
    encode_baseline_btree_leaf_record, encode_baseline_btree_root_record,
    BaselineBTreeCorruptionMarker, BaselineBTreeExecutionWitness, BaselineBTreeReadPreflight,
    BaselineBTreeReadSource,
};
use worth_store_physical_format::{
    InMemoryPhysicalFormatModel, InMemoryPhysicalFormatModelRequest,
    InMemoryPhysicalFormatReplayArtifact, PhysicalGeneration, PhysicalGenerationAuthority,
    PhysicalPageId, PhysicalRecordSlot, PhysicalReference, PhysicalSegmentId,
    PhysicalStoreIdentity, PlatformPhysicalAppendRequest, SlotGenerationCell,
};

use super::bootstrap::foreign_layout_physical_store_identity;

#[derive(Debug, Clone)]
pub struct DeterministicBTreeReplayWorld {
    readiness: AcceptedHandoffReadiness,
    root_reference: PhysicalReference,
    replay_artifact: InMemoryPhysicalFormatReplayArtifact,
}

impl DeterministicBTreeReplayWorld {
    pub const fn readiness(&self) -> &AcceptedHandoffReadiness {
        &self.readiness
    }
    pub const fn root_reference(&self) -> PhysicalReference {
        self.root_reference
    }
    pub const fn replay_artifact(&self) -> &InMemoryPhysicalFormatReplayArtifact {
        &self.replay_artifact
    }
}

pub fn deterministic_baseline_btree_read_preflight() -> BaselineBTreeReadPreflight {
    deterministic_btree_read_preflight(BTreeFixtureDamage::None, None)
}

pub fn deterministic_cross_store_btree_read_preflight() -> BaselineBTreeReadPreflight {
    deterministic_btree_read_preflight(
        BTreeFixtureDamage::None,
        Some(foreign_layout_physical_store_identity()),
    )
}

pub fn deterministic_corrupt_leaf_btree_read_preflight() -> BaselineBTreeReadPreflight {
    deterministic_btree_read_preflight(BTreeFixtureDamage::CorruptLeftLeaf, None)
}

pub fn deterministic_stale_child_btree_read_preflight() -> BaselineBTreeReadPreflight {
    deterministic_btree_read_preflight(BTreeFixtureDamage::StaleLeftReference, None)
}

pub fn deterministic_baseline_btree_read_source() -> BaselineBTreeReadSource {
    admit_read_source(
        deterministic_baseline_btree_read_preflight(),
        super::super::physical_isolation::epoch_scope::physical_authority_from_complete_closeout(),
    )
}

pub fn deterministic_cross_store_btree_read_source() -> BaselineBTreeReadSource {
    let store = foreign_layout_physical_store_identity();
    admit_read_source(
        deterministic_cross_store_btree_read_preflight(),
        super::super::physical_isolation::epoch_scope::
            physical_authority_from_complete_closeout_for_store(&store),
    )
}

pub fn deterministic_corrupt_leaf_btree_read_source() -> BaselineBTreeReadSource {
    admitted_source(BTreeFixtureDamage::CorruptLeftLeaf)
}

pub fn deterministic_stale_child_btree_read_source() -> BaselineBTreeReadSource {
    admitted_source(BTreeFixtureDamage::StaleLeftReference)
}

pub fn deterministic_noncanonical_leaf_btree_read_source() -> BaselineBTreeReadSource {
    admitted_source(BTreeFixtureDamage::NoncanonicalLeftLeaf)
}

pub fn deterministic_left_partition_violation_btree_read_source() -> BaselineBTreeReadSource {
    admitted_source(BTreeFixtureDamage::LeftChildCrossesSeparator)
}

pub fn deterministic_right_partition_violation_btree_read_source() -> BaselineBTreeReadSource {
    admitted_source(BTreeFixtureDamage::RightChildPrecedesSeparator)
}

fn admitted_source(damage: BTreeFixtureDamage) -> BaselineBTreeReadSource {
    admit_read_source(
        deterministic_btree_read_preflight(damage, None),
        super::super::physical_isolation::epoch_scope::physical_authority_from_complete_closeout(),
    )
}

fn admit_read_source(
    preflight: BaselineBTreeReadPreflight,
    authority: worth_store_physical_isolation::PhysicalReadStabilityAuthority,
) -> BaselineBTreeReadSource {
    let root =
        super::super::physical_isolation::epoch_scope::current_root_from_authority(&authority);
    let references = super::super::physical_isolation::read_plan::protected_set(
        preflight.protected_references(),
        3,
    );
    let plan = super::super::physical_isolation::read_plan::admit_plan(
        &authority, root, references, 12_288, 3,
    );
    preflight.admit(plan).unwrap()
}

fn deterministic_btree_read_preflight(
    damage: BTreeFixtureDamage,
    store_identity: Option<PhysicalStoreIdentity>,
) -> BaselineBTreeReadPreflight {
    let world = deterministic_btree_replay_world_with_damage(damage, store_identity);
    BaselineBTreeExecutionWitness::admit_published_layout(
        world.readiness,
        world.root_reference,
        world.replay_artifact,
    )
    .expect("deterministic B-tree fixture must admit")
    .preflight_stable_read()
    .expect("published B-tree root must produce a bounded read preflight")
}

pub fn deterministic_btree_replay_world() -> DeterministicBTreeReplayWorld {
    deterministic_btree_replay_world_with_damage(BTreeFixtureDamage::None, None)
}

#[cfg(feature = "certification-world")]
pub fn deterministic_admitted_btree_replay_physical_source(
) -> worth_store_layout_indexes::AdmittedBTreeReplayPhysicalSource {
    let world = deterministic_btree_replay_world();
    let root = world.root_reference();
    worth_store_layout_indexes::AdmittedBTreeReplayPhysicalSource::admit(
        world.readiness().clone(),
        root,
        world.replay_artifact().clone(),
        world.replay_artifact().store_identity().clone(),
        super::super::recovery::deterministic_checkpoint_plus_tail_source(),
    )
    .expect("deterministic B-tree replay source must admit through recovery physics")
}

fn deterministic_btree_replay_world_with_damage(
    damage: BTreeFixtureDamage,
    store_identity: Option<PhysicalStoreIdentity>,
) -> DeterministicBTreeReplayWorld {
    let readiness = readiness();
    let open_request = store_identity.map_or_else(
        InMemoryPhysicalFormatModelRequest::physical_format_canonical,
        InMemoryPhysicalFormatModelRequest::physical_format_for_store,
    );
    let mut facade =
        InMemoryPhysicalFormatModel::start_empty_model(readiness.clone(), open_request)
            .expect("open deterministic B-tree fixture");
    let left_payload = match damage {
        BTreeFixtureDamage::CorruptLeftLeaf => *b"broken",
        BTreeFixtureDamage::NoncanonicalLeftLeaf => {
            encode_baseline_btree_leaf_record([slot(11), slot(10)], false, false)
        }
        BTreeFixtureDamage::LeftChildCrossesSeparator => {
            encode_baseline_btree_leaf_record([slot(11), slot(12)], false, false)
        }
        _ => encode_baseline_btree_leaf_record([slot(10), slot(11)], false, false),
    };
    facade
        .append_physical_record(PlatformPhysicalAppendRequest::page_slot(
            left_slot_cell(),
            &left_payload,
        ))
        .expect("append left B-tree leaf");
    facade
        .append_physical_record(PlatformPhysicalAppendRequest::page_slot(
            right_slot_cell(),
            &right_leaf_payload(damage),
        ))
        .expect("append right B-tree leaf");
    let left_reference = if damage == BTreeFixtureDamage::StaleLeftReference {
        cell(11, 1, 99)
    } else {
        left_slot_cell()
    };
    let root = facade
        .append_physical_record(PlatformPhysicalAppendRequest::page_slot(
            root_slot_cell(),
            &encode_baseline_btree_root_record(
                BaselineBTreeCorruptionMarker::Header,
                slot(12),
                left_reference,
                right_slot_cell(),
            ),
        ))
        .expect("append B-tree root");
    let published = facade.publish_physical_root().expect("publish B-tree root");
    DeterministicBTreeReplayWorld {
        readiness,
        root_reference: root.reference(),
        replay_artifact: published.replay_artifact(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BTreeFixtureDamage {
    None,
    CorruptLeftLeaf,
    StaleLeftReference,
    NoncanonicalLeftLeaf,
    LeftChildCrossesSeparator,
    RightChildPrecedesSeparator,
}

fn right_leaf_payload(damage: BTreeFixtureDamage) -> [u8; 6] {
    let slots = if damage == BTreeFixtureDamage::RightChildPrecedesSeparator {
        [slot(11), slot(13)]
    } else {
        [slot(12), slot(13)]
    };
    encode_baseline_btree_leaf_record(slots, false, false)
}

pub fn baseline_btree_probe_slot() -> PhysicalRecordSlot {
    slot(11)
}

fn readiness() -> AcceptedHandoffReadiness {
    AcceptedHandoffReadiness::from_foundational_handoff_artifacts(
        ROADMAP_2_S1_SCOPE,
        HandoffEvidenceDigestSet::new(
            digest("backend"),
            digest("deferred"),
            digest("harness"),
            digest("terms"),
            digest("audit"),
            digest("complexity"),
            digest("provenance"),
        ),
    )
    .expect("fixture readiness")
}

fn digest(name: &str) -> StableDigest {
    StableDigest::new(format!("sha256:{name}")).expect("fixture digest")
}
fn root_slot_cell() -> SlotGenerationCell {
    cell(9, 1, 17)
}
fn left_slot_cell() -> SlotGenerationCell {
    cell(11, 1, 11)
}
fn right_slot_cell() -> SlotGenerationCell {
    cell(13, 1, 13)
}
fn cell(page: u64, slot_value: u16, generation: u64) -> SlotGenerationCell {
    PhysicalGenerationAuthority::for_canonical_physical_format()
        .slot_cell(segment(7), page_id(page), slot(slot_value))
        .with_slot_generation(PhysicalGeneration::from_raw(generation).unwrap())
}
fn segment(value: u64) -> PhysicalSegmentId {
    PhysicalSegmentId::from_raw(value).unwrap()
}
fn page_id(value: u64) -> PhysicalPageId {
    PhysicalPageId::from_raw(value).unwrap()
}
fn slot(value: u16) -> PhysicalRecordSlot {
    PhysicalRecordSlot::from_raw(value).unwrap()
}
