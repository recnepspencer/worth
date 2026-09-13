use std::collections::BTreeMap;
use std::path::Path;

use sha2::{Digest, Sha256};

use super::{observe_artifact_at_path, wal_topology, WalFacts};

const CHECKPOINT_PATH: &str = "families/checkpoint.current";
const WAL_PREFIX: &str = "families/wal/";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CleanupWalIdentity {
    pub(crate) path: String,
    pub(crate) digest: [u8; 32],
    pub(crate) segment: u64,
    pub(crate) generation: u64,
    pub(crate) first: u64,
    pub(crate) last: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CleanupCandidateProof {
    pub(crate) checkpoint_frontier: u64,
    pub(crate) covered_segments: u64,
    pub(crate) retained_segments: u64,
    pub(crate) covered: Box<[CleanupWalIdentity]>,
    pub(crate) retained: Box<[CleanupWalIdentity]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CleanupTransitionProof {
    pub(crate) removed_covered: Box<[String]>,
    pub(crate) retained: Box<[CleanupWalIdentity]>,
}

pub(crate) fn capture(root: &Path) -> Result<CleanupCandidateProof, String> {
    let files = collect_root_files(root)?;
    prove(&files)
}

pub(crate) fn verify_removed_covered(
    root: &Path,
    before: &CleanupCandidateProof,
) -> Result<CleanupTransitionProof, String> {
    let files = collect_root_files(root)?;
    let after = wal_identities(&files)?;
    let after_by_path = after
        .into_iter()
        .map(|identity| (identity.path.clone(), identity))
        .collect::<BTreeMap<_, _>>();
    let covered_paths = before
        .covered
        .iter()
        .map(|identity| identity.path.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let before_paths = before
        .covered
        .iter()
        .chain(before.retained.iter())
        .map(|identity| identity.path.clone())
        .collect::<std::collections::BTreeSet<_>>();
    for identity in &before.covered {
        if after_by_path.contains_key(&identity.path) {
            return Err(format!(
                "cleanup did not remove whole covered WAL segment {}",
                identity.path
            ));
        }
    }
    for identity in &before.retained {
        let Some(current) = after_by_path.get(&identity.path) else {
            return Err(format!(
                "cleanup removed retained WAL tail {}",
                identity.path
            ));
        };
        if current != identity {
            return Err(format!(
                "cleanup changed retained WAL tail {}",
                identity.path
            ));
        }
    }
    if after_by_path
        .keys()
        .any(|path| !before_paths.contains(path))
    {
        return Err("cleanup created an unexpected WAL artifact".to_owned());
    }
    Ok(CleanupTransitionProof {
        removed_covered: covered_paths.into_iter().collect(),
        retained: before.retained.clone(),
    })
}

pub(crate) fn verify_preserved(root: &Path, before: &CleanupCandidateProof) -> Result<(), String> {
    let files = collect_root_files(root)?;
    let after = wal_identities(&files)?;
    let after_by_path = after
        .into_iter()
        .map(|identity| (identity.path.clone(), identity))
        .collect::<BTreeMap<_, _>>();
    for identity in before.covered.iter().chain(before.retained.iter()) {
        if after_by_path.get(&identity.path) != Some(identity) {
            return Err(format!(
                "cancelled cleanup changed candidate WAL artifact {}",
                identity.path
            ));
        }
    }
    Ok(())
}

pub(crate) fn prove(files: &[(String, Vec<u8>)]) -> Result<CleanupCandidateProof, String> {
    let checkpoint = files
        .iter()
        .find(|(path, _)| path == CHECKPOINT_PATH)
        .ok_or_else(|| "cleanup candidate oracle cannot find current checkpoint".to_owned())?;
    let checkpoint_facts = observe_artifact_at_path(&checkpoint.0, &checkpoint.1)
        .checkpoint
        .ok_or_else(|| "cleanup candidate oracle rejected current checkpoint".to_owned())?;
    let frontier = checkpoint_facts.covered.1;
    let wal = wal_identities(files)?;
    let covered = wal
        .iter()
        .filter(|identity| identity.last <= frontier)
        .cloned()
        .collect::<Vec<_>>();
    let retained = wal
        .iter()
        .filter(|identity| identity.first <= frontier && identity.last > frontier)
        .cloned()
        .collect::<Vec<_>>();
    if covered.is_empty() {
        return Err(format!(
            "cleanup candidate oracle found no whole checkpoint-covered WAL segment at frontier {frontier}"
        ));
    }
    if retained.is_empty() {
        return Err(format!(
            "cleanup candidate oracle found no retained WAL tail after frontier {frontier}"
        ));
    }
    if wal.len() < 2 {
        return Err(
            "cleanup candidate oracle requires distinct covered and retained segments".to_owned(),
        );
    }
    Ok(CleanupCandidateProof {
        checkpoint_frontier: frontier,
        covered_segments: covered.len() as u64,
        retained_segments: retained.len() as u64,
        covered: covered.into_boxed_slice(),
        retained: retained.into_boxed_slice(),
    })
}

fn collect_root_files(root: &Path) -> Result<Vec<(String, Vec<u8>)>, String> {
    let root = root
        .canonicalize()
        .map_err(|error| format!("canonicalize cleanup candidate root: {error}"))?;
    let mut files = Vec::new();
    super::super::artifacts::collect_files(&root, &root, &mut files)?;
    files.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(files)
}

fn wal_identities(files: &[(String, Vec<u8>)]) -> Result<Vec<CleanupWalIdentity>, String> {
    let mut facts = Vec::<WalFacts>::new();
    let mut identities = Vec::new();
    for (path, bytes) in files
        .iter()
        .filter(|(path, _)| path.starts_with(WAL_PREFIX) && path.ends_with(".wal"))
    {
        let observed = observe_artifact_at_path(path, bytes);
        let wal = observed
            .wal
            .ok_or_else(|| format!("cleanup candidate oracle rejected WAL artifact {path}"))?;
        if observed.wal_residue.is_some() || wal.valid_bytes != wal.observed_bytes {
            return Err(format!(
                "cleanup candidate oracle found torn or interrupted WAL artifact {path}"
            ));
        }
        let (Some(segment), Some(generation), Some(first), Some(last)) =
            (wal.segment, wal.generation, wal.first, wal.last)
        else {
            return Err(format!(
                "cleanup candidate oracle found an unparseable WAL artifact {path}"
            ));
        };
        if wal.frames == 0 {
            return Err(format!(
                "cleanup candidate oracle found an empty WAL artifact {path}"
            ));
        }
        facts.push(wal);
        identities.push(CleanupWalIdentity {
            path: path.clone(),
            digest: Sha256::digest(bytes).into(),
            segment,
            generation,
            first,
            last,
        });
    }
    if identities.is_empty() {
        return Err("cleanup candidate oracle found no WAL segments".to_owned());
    }
    wal_topology::validate(&facts)?;
    identities.sort_by_key(|identity| (identity.segment, identity.generation));
    Ok(identities)
}
