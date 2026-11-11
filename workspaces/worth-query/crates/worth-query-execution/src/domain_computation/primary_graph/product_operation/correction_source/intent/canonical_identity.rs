//! Port-free canonical correction identity; Home scope excludes the live receiver.
use crate::domain_computation::application_aftermath::{
    WorthQueryAftermathCausalRole, WorthQueryRecordedCorrectionSource,
};
use sha2::{Digest, Sha256};
use worth_foundational::facade::{DurableWorldOwnerIdentity, WorldOwnerCoordinate};
use worth_relational::facade::{history::CommitId, identity::VersionId};
#[cfg(test)]
mod tests;

// These borrowed encoder inputs carry descriptive DATA, never admission proof.
#[derive(Clone, Copy)]
pub(super) struct Coordinate<'a> {
    pub owner: WorldOwnerCoordinate,
    pub world_commit: u64,
    pub native_commit: CommitId,
    pub native_version: VersionId,
    pub native_branch: &'a str,
}
impl<'a> Coordinate<'a> {
    pub(super) fn from_source(source: &'a WorthQueryRecordedCorrectionSource) -> Self {
        Self {
            owner: source.world_owner_coordinate(),
            world_commit: source.world().ordinal(),
            native_commit: source.relational().commit_id,
            native_version: source.relational().version_id,
            native_branch: &source.relational().branch_id.0,
        }
    }
}
#[derive(Clone, Copy)]
pub(super) struct CorrectionIntentFields<'a> {
    pub bindings: [&'a str; 3],
    pub role: WorthQueryAftermathCausalRole,
    pub source_owner: WorldOwnerCoordinate,
    pub source_commit: u64,
    pub target_owner: WorldOwnerCoordinate,
    pub target_branch: &'a str,
    pub target_lifecycle: u64,
    pub original_intent: &'a [u8; 32],
    pub lineage: [Option<Coordinate<'a>>; 4],
}
pub(super) fn scope(
    durable: Option<DurableWorldOwnerIdentity>,
    observation: u64,
) -> WorldOwnerCoordinate {
    match durable {
        Some(owner) => WorldOwnerCoordinate::Home { owner },
        None => WorldOwnerCoordinate::MemoryOnly { observation },
    }
}
pub(super) fn encode(fields: CorrectionIntentFields<'_>) -> [u8; 32] {
    // Memory retains its active process-local grammar. Home has a distinct
    // namespace/version and encodes each complete Home owner, never its live ID.
    let home = fields.source_owner.home_owner().is_some()
        || fields.target_owner.home_owner().is_some()
        || fields
            .lineage
            .iter()
            .flatten()
            .any(|coordinate| coordinate.owner.home_owner().is_some());
    let domain: &[u8] = if home {
        b"worth-query.recorded-correction.intent.home.v1"
    } else {
        b"worth-query.recorded-correction.intent.v2"
    };
    let (source, source_len) = owner_frame(fields.source_owner, home);
    let (target, target_len) = owner_frame(fields.target_owner, home);
    let mut digest = Sha256::new();
    for part in [
        domain,
        fields.bindings[0].as_bytes(),
        fields.bindings[1].as_bytes(),
        fields.bindings[2].as_bytes(),
        &fields.role.code().to_be_bytes(),
        &source[..source_len],
        &fields.source_commit.to_be_bytes(),
        fields.target_branch.as_bytes(),
        &target[..target_len],
        &fields.target_lifecycle.to_be_bytes(),
        fields.original_intent,
    ] {
        digest.update((part.len() as u64).to_be_bytes());
        digest.update(part);
    }
    for coordinate in fields.lineage {
        digest.update([u8::from(coordinate.is_some())]);
        if let Some(coordinate) = coordinate {
            let (owner, length) = owner_frame(coordinate.owner, home);
            digest.update(&owner[..length]);
            for part in [
                coordinate.world_commit.to_be_bytes(),
                coordinate.native_commit.0.to_be_bytes(),
                coordinate.native_version.0.to_be_bytes(),
            ] {
                digest.update(part);
            }
            let branch = coordinate.native_branch.as_bytes();
            digest.update((branch.len() as u64).to_be_bytes());
            digest.update(branch);
        }
    }
    digest.finalize().into()
}
fn owner_frame(owner: WorldOwnerCoordinate, home_domain: bool) -> ([u8; 25], usize) {
    let mut bytes = [0; 25];
    match owner {
        WorldOwnerCoordinate::Home { owner } => {
            bytes[0] = 1;
            bytes[1..17].copy_from_slice(&owner.home().bytes());
            bytes[17..25].copy_from_slice(&owner.slot().get().to_be_bytes());
            (bytes, 25)
        }
        WorldOwnerCoordinate::MemoryOnly { observation } => {
            let start = usize::from(home_domain);
            bytes[start..start + 8].copy_from_slice(&observation.to_be_bytes());
            (bytes, start + 8)
        }
    }
}
