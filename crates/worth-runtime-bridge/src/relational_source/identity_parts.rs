use std::sync::Arc;

use crate::identity::{
    BridgeIdentity, BridgeIdentityPayload, HistoricalResolvedLineageIdentityTag,
    HistoricalResolvedRecordIdentityTag, TruthBranchTag, TruthCommitTag, TruthPatchTag,
    TruthSnapshotTag,
};
use crate::input::envelope::{BridgeCommittedPatchItem, BridgeCommittedPatchTarget};
use crate::snapshot::{SnapshotReadContract, SnapshotReadRequest};

const RELATIONAL_BRANCH_PREFIX: &str = "relational-branch:";
const RELATIONAL_COMMIT_PREFIX: &str = "relational-commit:";
const RELATIONAL_LINEAGE_PREFIX: &str = "relational-lineage:";
const RELATIONAL_PATCH_PREFIX: &str = "relational-patch:";
const RELATIONAL_RECORD_ENTITY_PREFIX: &str = "relational-record:entity:";
const RELATIONAL_RECORD_RELATION_PREFIX: &str = "relational-record:relation:";
const RELATIONAL_SNAPSHOT_PREFIX: &str = "relational-snapshot:";
const RELATIONAL_SNAPSHOT_VERSION_SEPARATOR: &str = ":version:";

/// Whether a Relational record is an entity or a relation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelationalBridgeRecordIdentityKind {
    /// An entity record.
    Entity,
    /// A relation record.
    Relation,
}

/// The parts of a Relational record identity, as the Bridge carries them.
///
/// A record is named by its kind, its partition, its local slot in that
/// partition, and the slot's generation. The Bridge writes these as
/// `relational-record:entity:{partition}:{slot}:{generation}` (or
/// `relational-record:relation:…`) and parses them back with
/// [`Self::from_bridge_entity_identity`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelationalBridgeRecordIdentityParts {
    kind: RelationalBridgeRecordIdentityKind,
    partition_id: u32,
    local_slot: u64,
    generation: u32,
}

impl RelationalBridgeRecordIdentityParts {
    /// The parts of an entity record.
    pub const fn entity(partition_id: u32, local_slot: u64, generation: u32) -> Self {
        Self::new(
            RelationalBridgeRecordIdentityKind::Entity,
            partition_id,
            local_slot,
            generation,
        )
    }

    /// The parts of a relation record.
    pub const fn relation(partition_id: u32, local_slot: u64, generation: u32) -> Self {
        Self::new(
            RelationalBridgeRecordIdentityKind::Relation,
            partition_id,
            local_slot,
            generation,
        )
    }

    /// The parts of a record of the given kind.
    pub const fn new(
        kind: RelationalBridgeRecordIdentityKind,
        partition_id: u32,
        local_slot: u64,
        generation: u32,
    ) -> Self {
        Self {
            kind,
            partition_id,
            local_slot,
            generation,
        }
    }

    /// Whether the record is an entity or a relation.
    pub const fn kind(self) -> RelationalBridgeRecordIdentityKind {
        self.kind
    }

    /// The partition that holds the record.
    pub const fn partition_id(self) -> u32 {
        self.partition_id
    }

    /// The record's slot within its partition.
    pub const fn local_slot(self) -> u64 {
        self.local_slot
    }

    /// The slot's generation, which tells a reused slot from its earlier record.
    pub const fn generation(self) -> u32 {
        self.generation
    }

    /// Parse a Bridge record identity string back into its parts.
    ///
    /// Returns `None` if the string does not have the Relational record shape.
    pub fn from_bridge_entity_identity(identity: &str) -> Option<Self> {
        let (kind, raw) = if let Some(raw) = identity.strip_prefix(RELATIONAL_RECORD_ENTITY_PREFIX)
        {
            (RelationalBridgeRecordIdentityKind::Entity, raw)
        } else {
            (
                RelationalBridgeRecordIdentityKind::Relation,
                identity.strip_prefix(RELATIONAL_RECORD_RELATION_PREFIX)?,
            )
        };
        let mut parts = raw.split(':');
        let partition_id = parts.next()?.parse::<u32>().ok()?;
        let local_slot = parts.next()?.parse::<u64>().ok()?;
        let generation = parts.next()?.parse::<u32>().ok()?;
        if parts.next().is_some() {
            return None;
        }
        Some(Self::new(kind, partition_id, local_slot, generation))
    }

    pub(crate) fn bridge_entity_identity(self) -> String {
        let prefix = match self.kind {
            RelationalBridgeRecordIdentityKind::Entity => RELATIONAL_RECORD_ENTITY_PREFIX,
            RelationalBridgeRecordIdentityKind::Relation => RELATIONAL_RECORD_RELATION_PREFIX,
        };
        format!(
            "{prefix}{}:{}:{}",
            self.partition_id, self.local_slot, self.generation
        )
    }

    /// The Bridge record identity string, for reports and diagnostics.
    pub fn terminal_projection_for_reporting(self) -> String {
        self.bridge_entity_identity()
    }
}

/// The parts of a Relational snapshot identity, as the Bridge carries them: the
/// snapshot id and the version it observes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelationalBridgeSnapshotIdentityParts {
    snapshot_id: u64,
    version_id: u64,
}

impl RelationalBridgeSnapshotIdentityParts {
    /// The parts of a snapshot identity.
    pub const fn new(snapshot_id: u64, version_id: u64) -> Self {
        Self {
            snapshot_id,
            version_id,
        }
    }

    /// The Relational snapshot id.
    pub const fn snapshot_id(self) -> u64 {
        self.snapshot_id
    }

    /// The Relational version the snapshot observes.
    pub const fn version_id(self) -> u64 {
        self.version_id
    }

    fn bridge_snapshot_identity(self) -> String {
        format!(
            "{RELATIONAL_SNAPSHOT_PREFIX}{}{RELATIONAL_SNAPSHOT_VERSION_SEPARATOR}{}",
            self.snapshot_id, self.version_id
        )
    }
}

impl BridgeIdentity<TruthBranchTag> {
    /// The Bridge identity of a Relational branch.
    pub fn from_relational_branch_id(branch_id: impl Into<Arc<str>>) -> Self {
        let branch_id = branch_id.into();
        Self::with_payload(
            format!("{RELATIONAL_BRANCH_PREFIX}{branch_id}"),
            BridgeIdentityPayload::RelationalBranch { branch_id },
        )
    }

    /// The Relational branch id, if this identity names a Relational branch.
    pub fn relational_branch_id(&self) -> Option<&str> {
        match self.payload() {
            BridgeIdentityPayload::RelationalBranch { branch_id } => Some(branch_id.as_ref()),
            _ => None,
        }
    }

    /// A test branch identity whose branch id is the fixture label itself.
    pub fn from_bridge_harness_label(label: impl Into<Arc<str>>) -> Self {
        Self::from_relational_branch_id(label)
    }
}

impl BridgeIdentity<TruthCommitTag> {
    /// The Bridge identity of a Relational commit.
    pub fn from_relational_commit_id(commit_id: u64) -> Self {
        Self::with_payload(
            format!("{RELATIONAL_COMMIT_PREFIX}{commit_id}"),
            BridgeIdentityPayload::RelationalCommit { commit_id },
        )
    }

    /// The Relational commit id, if this identity names a Relational commit.
    pub fn relational_commit_id(&self) -> Option<u64> {
        match self.payload() {
            BridgeIdentityPayload::RelationalCommit { commit_id } => Some(*commit_id),
            _ => None,
        }
    }

    /// A deterministic test identity derived from a fixture label.
    ///
    /// Labels ending in `-a` to `-f` or a number map to small positions;
    /// other labels are hashed.
    pub fn from_bridge_harness_label(label: impl Into<String>) -> Self {
        Self::from_relational_commit_id(fixture_position(label))
    }
}

impl BridgeIdentity<TruthPatchTag> {
    /// The Bridge identity of a Relational patch position.
    pub fn from_relational_patch_position(patch_position: u64) -> Self {
        Self::with_payload(
            format!("{RELATIONAL_PATCH_PREFIX}{patch_position}"),
            BridgeIdentityPayload::RelationalPatch { patch_position },
        )
    }

    /// The Relational patch position, if this identity names one.
    pub fn relational_patch_position(&self) -> Option<u64> {
        match self.payload() {
            BridgeIdentityPayload::RelationalPatch { patch_position } => Some(*patch_position),
            _ => None,
        }
    }

    /// A deterministic test identity derived from a fixture label.
    ///
    /// Labels ending in `-a` to `-f` or a number map to small positions;
    /// other labels are hashed.
    pub fn from_bridge_harness_label(label: impl Into<String>) -> Self {
        Self::from_relational_patch_position(fixture_position(label))
    }
}

impl BridgeIdentity<TruthSnapshotTag> {
    /// The Bridge identity of a Relational snapshot.
    pub fn from_relational_snapshot(parts: RelationalBridgeSnapshotIdentityParts) -> Self {
        Self::with_payload(
            parts.bridge_snapshot_identity(),
            BridgeIdentityPayload::RelationalSnapshot {
                snapshot_id: parts.snapshot_id(),
                version_id: parts.version_id(),
            },
        )
    }

    /// The Relational snapshot parts, if this identity names a Relational snapshot.
    pub fn relational_snapshot_parts(&self) -> Option<RelationalBridgeSnapshotIdentityParts> {
        match self.payload() {
            BridgeIdentityPayload::RelationalSnapshot {
                snapshot_id,
                version_id,
            } => Some(RelationalBridgeSnapshotIdentityParts::new(
                *snapshot_id,
                *version_id,
            )),
            _ => None,
        }
    }

    /// A deterministic test identity derived from a fixture label.
    ///
    /// Labels ending in `-a` to `-f` or a number map to small positions;
    /// other labels are hashed.
    pub fn from_bridge_harness_label(label: impl Into<String>) -> Self {
        Self::from_relational_snapshot(RelationalBridgeSnapshotIdentityParts::new(
            fixture_position(label),
            1,
        ))
    }
}

impl BridgeIdentity<HistoricalResolvedLineageIdentityTag> {
    /// The Bridge identity of a resolved Relational lineage.
    pub fn from_relational_lineage_id(lineage_id: u64) -> Self {
        Self::admit_bridge_owned(format!("{RELATIONAL_LINEAGE_PREFIX}{lineage_id}"))
    }

    /// A deterministic test identity derived from a fixture label.
    ///
    /// Labels ending in `-a` to `-f` or a number map to small positions;
    /// other labels are hashed.
    pub fn from_bridge_harness_label(label: impl Into<String>) -> Self {
        Self::from_relational_lineage_id(fixture_position(label))
    }
}

impl BridgeIdentity<HistoricalResolvedRecordIdentityTag> {
    /// The Bridge identity of a resolved Relational record.
    pub fn from_relational_record(parts: RelationalBridgeRecordIdentityParts) -> Self {
        Self::admit_bridge_owned(parts.bridge_entity_identity())
    }

    /// A deterministic test identity derived from a fixture label.
    ///
    /// Labels ending in `-a` to `-f` or a number map to small positions;
    /// other labels are hashed.
    pub fn from_bridge_harness_label(label: impl Into<String>) -> Self {
        Self::from_relational_record(RelationalBridgeRecordIdentityParts::entity(
            1,
            fixture_position(label),
            1,
        ))
    }
}

impl BridgeCommittedPatchItem {
    /// A committed-patch item for a Relational record, with no semantic change
    /// attached.
    pub fn with_relational_record_target(
        record_identity: RelationalBridgeRecordIdentityParts,
        target: BridgeCommittedPatchTarget,
    ) -> Self {
        Self::from_relational_record_parts(
            record_identity.bridge_entity_identity(),
            record_identity,
            target,
            None,
        )
    }

    /// A committed-patch item for a Relational record, carrying the semantic
    /// aspect change Relational reported for it.
    pub fn with_relational_semantic_change(
        record_identity: RelationalBridgeRecordIdentityParts,
        target: BridgeCommittedPatchTarget,
        semantic_change: crate::input::envelope::BridgeSemanticAspectChange,
    ) -> Self {
        Self::from_relational_record_parts(
            record_identity.bridge_entity_identity(),
            record_identity,
            target,
            Some(semantic_change),
        )
    }
}

impl SnapshotReadRequest {
    /// A snapshot read request for one Relational record.
    pub fn for_relational_record(
        record_identity: RelationalBridgeRecordIdentityParts,
        target: SnapshotReadContract,
    ) -> Self {
        Self::for_coarse_relational_record(
            record_identity.bridge_entity_identity(),
            record_identity,
            target,
        )
    }
}

fn fixture_position(label: impl Into<String>) -> u64 {
    let label = label.into();
    if let Some(position) = fixture_suffix_position(&label) {
        return position;
    }
    label.bytes().fold(17_u64, |acc, byte| {
        acc.wrapping_mul(131).wrapping_add(u64::from(byte))
    })
}

fn fixture_suffix_position(label: &str) -> Option<u64> {
    if label.split(['-', ':']).count() != 2 {
        return None;
    }
    let suffix = label.rsplit(['-', ':']).next()?;
    match suffix {
        "a" => Some(1),
        "b" => Some(2),
        "c" => Some(3),
        "d" => Some(4),
        "e" => Some(5),
        "f" => Some(6),
        _ => suffix.parse::<u64>().ok(),
    }
}
