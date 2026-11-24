use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};

use crate::branch_lifecycle::{Posture, State};
use crate::media_case::{Ended, Target};
use crate::workload::{Script, Step};

pub(crate) fn hash_byte_slices(items: &[Vec<u8>]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update((items.len() as u64).to_le_bytes());
    for item in items {
        hasher.update((item.len() as u64).to_le_bytes());
        hasher.update(item);
    }
    hasher.finalize().into()
}

pub(crate) fn hash_logical_state(state: &State) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update((state.postures.len() as u64).to_le_bytes());
    for (branch, posture) in &state.postures {
        hasher.update((branch.len() as u64).to_le_bytes());
        hasher.update(branch.as_bytes());
        let tag: u8 = match posture {
            Posture::Absent => 0,
            Posture::Live => 1,
            Posture::Archived => 2,
            Posture::Deleting => 3,
        };
        hasher.update([tag]);
    }
    hasher.update((state.records.len() as u64).to_le_bytes());
    for (branch, rendered) in &state.records {
        hasher.update((branch.len() as u64).to_le_bytes());
        hasher.update(branch.as_bytes());
        hasher.update((rendered.len() as u64).to_le_bytes());
        for value in rendered {
            hasher.update((value.len() as u64).to_le_bytes());
            hasher.update(value.as_bytes());
        }
    }
    hasher.finalize().into()
}

/// Hash sorted relative dirs + `(relative_file_path, readonly, len)` of the
/// quiescent post-drop home directory (complementing content-verified
/// Relational manifest, checkpoint member bytes, and journal artifact hashes).
pub(crate) fn hash_disk_layout(root: &Path) -> [u8; 32] {
    let mut dirs: Vec<(PathBuf, bool)> = Vec::new();
    let mut files: Vec<(PathBuf, bool, u64)> = Vec::new();
    let mut queue = vec![PathBuf::new()];
    let mut next = 0;
    while let Some(relative) = queue.get(next).cloned() {
        let mut entries: Vec<_> = std::fs::read_dir(root.join(&relative))
            .expect("read recovered home directory")
            .map(|entry| entry.expect("recovered home entry"))
            .collect();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let rel = relative.join(entry.file_name());
            let meta = std::fs::symlink_metadata(entry.path()).expect("recovered entry metadata");
            let kind = meta.file_type();
            assert!(!kind.is_symlink(), "unexpected symlink: {}", rel.display());
            let ro = meta.permissions().readonly();
            if kind.is_dir() {
                dirs.push((rel.clone(), ro));
                queue.push(rel);
            } else {
                files.push((rel, ro, meta.len()));
            }
        }
        next += 1;
    }
    dirs.sort();
    files.sort();
    let mut hasher = Sha256::new();
    hasher.update((dirs.len() as u64).to_le_bytes());
    for (dir, ro) in dirs {
        let s = dir.to_string_lossy();
        hasher.update((s.len() as u64).to_le_bytes());
        hasher.update(s.as_bytes());
        hasher.update([u8::from(ro)]);
    }
    hasher.update((files.len() as u64).to_le_bytes());
    for (file, ro, len) in files {
        let s = file.to_string_lossy();
        hasher.update((s.len() as u64).to_le_bytes());
        hasher.update(s.as_bytes());
        hasher.update([u8::from(ro)]);
        hasher.update(len.to_le_bytes());
    }
    hasher.finalize().into()
}

#[derive(Default)]
pub(crate) struct PhaseTimings {
    pub(crate) prep_us: AtomicU64,
    pub(crate) restore_us: AtomicU64,
    pub(crate) open_fault_us: AtomicU64,
    pub(crate) target_us: AtomicU64,
    pub(crate) recover_us: AtomicU64,
    pub(crate) oracle_us: AtomicU64,
    pub(crate) eq_us: AtomicU64,
    pub(crate) continues_us: AtomicU64,
}

impl PhaseTimings {
    pub(crate) fn summary(&self) -> String {
        let ms = |a: &AtomicU64| a.load(Ordering::Relaxed) / 1000;
        format!(
            "prep={}ms restore={}ms open_fault={}ms target={}ms recover={}ms oracle={}ms eq={}ms continues={}ms",
            ms(&self.prep_us),
            ms(&self.restore_us),
            ms(&self.open_fault_us),
            ms(&self.target_us),
            ms(&self.recover_us),
            ms(&self.oracle_us),
            ms(&self.eq_us),
            ms(&self.continues_us),
        )
    }
}

/// How many cases ran and how the target ended in them.
#[derive(Default, Debug)]
pub(crate) struct Tally {
    pub(crate) cases: usize,
    pub(crate) skipped: usize,
    pub(crate) acknowledged: usize,
    pub(crate) refused: usize,
    pub(crate) unresolved_absent: usize,
    pub(crate) unresolved_recovered: usize,
}

impl Tally {
    pub(crate) fn add(&mut self, ended: Ended) {
        match ended {
            Ended::Skipped => self.skipped += 1,
            Ended::Acknowledged => self.acknowledged += 1,
            Ended::Refused => self.refused += 1,
            Ended::Unresolved { recovered: false } => self.unresolved_absent += 1,
            Ended::Unresolved { recovered: true } => self.unresolved_recovered += 1,
        }
        if !matches!(ended, Ended::Skipped) {
            self.cases += 1;
        }
    }
}

pub(crate) fn first_commit(script: &Script) -> Target {
    let index = script.steps.iter().position(Step::is_commit).unwrap();
    Target::Commit(index)
}

pub(crate) fn first_feature_commit(script: &Script) -> Target {
    let fork = script
        .steps
        .iter()
        .position(|step| matches!(step, Step::Fork { .. }))
        .expect("the script forks");
    Target::Commit(fork + 1)
}

pub(crate) fn transition_commit(script: &Script) -> Target {
    let index = script
        .steps
        .iter()
        .position(|step| matches!(step, Step::Transition { .. }))
        .expect("the script transitions");
    Target::Commit(index)
}

/// The script's second checkpoint, which replaces a selected one.
pub(crate) fn replacement_checkpoint(script: &Script) -> Target {
    let index = script
        .steps
        .iter()
        .enumerate()
        .filter(|(_, step)| matches!(step, Step::Checkpoint))
        .nth(1)
        .expect("the script checkpoints twice")
        .0;
    Target::Checkpoint(index)
}

/// Per-seed worker cap: partitioning workers across seeds means each worker
/// builds only ONE root-bound home (never both seeds!), cutting prefix build
/// work in half while keeping both seeds running concurrently.
pub(crate) fn workers_per_seed(
    seed_count: usize,
    scheduled_cases: usize,
    prefix_len: usize,
) -> usize {
    let total_budget = std::thread::available_parallelism()
        .map_or(8, std::num::NonZeroUsize::get)
        .clamp(8, 24);
    let max_per_seed = (total_budget / seed_count).max(4);
    if scheduled_cases <= 32 && prefix_len <= 6 {
        return 4.min(max_per_seed);
    }
    let min_cases_per_worker = if prefix_len >= 12 { 11 } else { 10 };
    (scheduled_cases / min_cases_per_worker).clamp(1, max_per_seed)
}
