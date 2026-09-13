use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub(crate) use super::file_identity::PhysicalFileIdentity;
use super::{
    OfflineIndeterminatePhysicalReason, OfflineIntegrityObservationCounters,
    OfflineIntegrityObservationLimits, OfflineIntegrityOutcome, OfflineIntegrityReportCompleteness,
    OfflineUnknownPhysicalReason,
};

mod acquisition;
mod path_binding;

pub(crate) struct BoundedMediaWalk {
    limits: OfflineIntegrityObservationLimits,
    store_root: PathBuf,
    counters: OfflineIntegrityObservationCounters,
    started: Instant,
    completeness: OfflineIntegrityReportCompleteness,
    seen_files: BTreeMap<PhysicalFileIdentity, CachedPhysicalFile>,
    cached_lengths: BTreeSet<u64>,
    seen_directories: BTreeMap<PathBuf, DirectoryScan>,
    last_exhaustion: Option<OfflineIndeterminatePhysicalReason>,
    refused_paths: BTreeMap<PathBuf, OfflineIndeterminatePhysicalReason>,
}

#[derive(Clone)]
pub(crate) struct DirectoryScan {
    pub(crate) entries: Vec<PathBuf>,
    pub(crate) incomplete_reason: Option<OfflineIndeterminatePhysicalReason>,
}

#[derive(Debug)]
pub(crate) struct BoundedAcquisition {
    pub(crate) bytes: Arc<[u8]>,
    pub(crate) byte_length: usize,
    pub(crate) physical_alias_of: Option<PathBuf>,
}

pub(crate) struct UnrecognizedEntryClassification {
    pub(crate) outcome: OfflineIntegrityOutcome,
    pub(crate) physical_alias_of: Option<PathBuf>,
}

struct CachedPhysicalFile {
    first_path: PathBuf,
    bytes: Option<Arc<[u8]>>,
    snapshot: Option<fs::Metadata>,
}

impl BoundedAcquisition {
    pub(crate) const fn is_alias(&self) -> bool {
        self.physical_alias_of.is_some()
    }
}

impl BoundedMediaWalk {
    pub(crate) fn exhausted_reason(&self) -> Option<OfflineIndeterminatePhysicalReason> {
        self.last_exhaustion
    }
    pub(crate) fn maximum_entries(&self) -> u64 {
        self.limits.maximum_entries()
    }
    pub(crate) fn entry_bound(&mut self) -> OfflineIntegrityOutcome {
        self.bound(OfflineIndeterminatePhysicalReason::EntryBoundExceeded)
    }
    pub(crate) fn new(
        limits: OfflineIntegrityObservationLimits,
        store_root: PathBuf,
        started: Instant,
    ) -> Self {
        Self {
            limits,
            store_root,
            counters: OfflineIntegrityObservationCounters::default(),
            started,
            completeness: OfflineIntegrityReportCompleteness::Complete,
            seen_files: BTreeMap::new(),
            cached_lengths: BTreeSet::new(),
            seen_directories: BTreeMap::new(),
            last_exhaustion: None,
            refused_paths: BTreeMap::new(),
        }
    }

    pub(crate) fn scan_directory(&mut self, path: &Path, depth: u32) -> io::Result<DirectoryScan> {
        if let Some(scan) = self.seen_directories.get(path) {
            return Ok(scan.clone());
        }
        let scan = self.scan_uncached_directory(path, depth)?;
        self.seen_directories.insert(path.to_owned(), scan.clone());
        Ok(scan)
    }

    fn scan_uncached_directory(&mut self, path: &Path, depth: u32) -> io::Result<DirectoryScan> {
        if let Some(reason) = self.depth_exhaustion(depth) {
            return Ok(incomplete_scan(Vec::new(), reason));
        }
        if let Some(reason) = self.refused_path_reason(path)? {
            return Ok(incomplete_scan(Vec::new(), reason));
        }
        let mut entries = Vec::new();
        for entry in fs::read_dir(path)? {
            if let Some(reason) = self.elapsed_exhaustion() {
                entries.sort();
                return Ok(incomplete_scan(entries, reason));
            }
            if self.counters.entries_visited == self.limits.maximum_entries() {
                self.mark_bound_exhausted();
                entries.sort();
                return Ok(incomplete_scan(
                    entries,
                    OfflineIndeterminatePhysicalReason::EntryBoundExceeded,
                ));
            }
            self.counters.entries_visited += 1;
            entries.push(entry?.path());
        }
        entries.sort();
        Ok(DirectoryScan {
            entries,
            incomplete_reason: None,
        })
    }

    pub(crate) const fn counters_mut(&mut self) -> &mut OfflineIntegrityObservationCounters {
        &mut self.counters
    }

    pub(crate) fn record_outcome(&mut self, outcome: &OfflineIntegrityOutcome) {
        if matches!(outcome, OfflineIntegrityOutcome::Unsupported(_)) {
            self.counters.unsupported_versions += 1;
        }
        if matches!(outcome, OfflineIntegrityOutcome::Indeterminate(_)) {
            self.counters.indeterminate_reads += 1;
            if self.completeness == OfflineIntegrityReportCompleteness::Complete {
                self.completeness = OfflineIntegrityReportCompleteness::Indeterminate;
            }
        }
        if matches!(
            outcome,
            OfflineIntegrityOutcome::Indeterminate(
                OfflineIndeterminatePhysicalReason::EntryBoundExceeded
                    | OfflineIndeterminatePhysicalReason::ByteBoundExceeded
                    | OfflineIndeterminatePhysicalReason::OpenFileBoundExceeded
                    | OfflineIndeterminatePhysicalReason::DepthBoundExceeded
                    | OfflineIndeterminatePhysicalReason::SymlinkBoundExceeded
                    | OfflineIndeterminatePhysicalReason::ElapsedBoundExceeded
            )
        ) {
            if let OfflineIntegrityOutcome::Indeterminate(reason) = outcome {
                self.last_exhaustion = Some(*reason);
            }
            self.mark_bound_exhausted();
        }
    }

    pub(crate) fn classify_unrecognized(
        &mut self,
        path: &Path,
        depth: u32,
    ) -> UnrecognizedEntryClassification {
        if let Some(reason) = self.depth_exhaustion(depth) {
            return self.unrecognized_outcome(OfflineIntegrityOutcome::Indeterminate(reason));
        }
        if let Some(reason) = self.elapsed_exhaustion() {
            return self.unrecognized_outcome(OfflineIntegrityOutcome::Indeterminate(reason));
        }
        match self.refused_path_reason(path) {
            Ok(Some(reason)) => {
                self.unrecognized_outcome(OfflineIntegrityOutcome::Indeterminate(reason))
            }
            Ok(None) => self.classify_unrecognized_entry(path),
            Err(_) => self.unrecognized_outcome(OfflineIntegrityOutcome::Unknown(
                OfflineUnknownPhysicalReason::FilesystemEntryUnavailable,
            )),
        }
    }

    fn unrecognized_outcome(
        &mut self,
        outcome: OfflineIntegrityOutcome,
    ) -> UnrecognizedEntryClassification {
        self.record_outcome(&outcome);
        UnrecognizedEntryClassification {
            outcome,
            physical_alias_of: None,
        }
    }

    pub(crate) fn finish(
        mut self,
    ) -> (
        OfflineIntegrityObservationCounters,
        OfflineIntegrityReportCompleteness,
    ) {
        self.elapsed_exhaustion();
        (self.counters, self.completeness)
    }

    fn depth_exhaustion(&mut self, depth: u32) -> Option<OfflineIndeterminatePhysicalReason> {
        self.counters.maximum_depth_reached = self.counters.maximum_depth_reached.max(depth);
        (depth > self.limits.maximum_depth()).then(|| {
            self.last_exhaustion = Some(OfflineIndeterminatePhysicalReason::DepthBoundExceeded);
            self.mark_bound_exhausted();
            OfflineIndeterminatePhysicalReason::DepthBoundExceeded
        })
    }

    fn elapsed_exhaustion(&mut self) -> Option<OfflineIndeterminatePhysicalReason> {
        (self.started.elapsed().as_millis() as u64 > self.limits.maximum_elapsed_milliseconds())
            .then(|| {
                self.last_exhaustion =
                    Some(OfflineIndeterminatePhysicalReason::ElapsedBoundExceeded);
                self.mark_bound_exhausted();
                OfflineIndeterminatePhysicalReason::ElapsedBoundExceeded
            })
    }

    fn remaining_elapsed_budget(&self) -> Duration {
        Duration::from_millis(self.limits.maximum_elapsed_milliseconds())
            .saturating_sub(self.started.elapsed())
    }

    fn refused_path_reason(
        &mut self,
        path: &Path,
    ) -> io::Result<Option<OfflineIndeterminatePhysicalReason>> {
        if let Some(reason) = self.refused_paths.get(path) {
            return Ok(Some(*reason));
        }
        let metadata = fs::symlink_metadata(path)?;
        let escaped = fs::canonicalize(path)
            .map(|resolved| !resolved.starts_with(&self.store_root))
            .unwrap_or(true);
        if !metadata.file_type().is_symlink() && !escaped {
            return Ok(None);
        }
        self.counters.symlinks_refused += 1;
        if self.counters.symlinks_refused > self.limits.maximum_symlinks() {
            self.last_exhaustion = Some(OfflineIndeterminatePhysicalReason::SymlinkBoundExceeded);
            self.refused_paths.insert(
                path.to_owned(),
                OfflineIndeterminatePhysicalReason::SymlinkBoundExceeded,
            );
            self.mark_bound_exhausted();
            Ok(Some(
                OfflineIndeterminatePhysicalReason::SymlinkBoundExceeded,
            ))
        } else {
            self.refused_paths.insert(
                path.to_owned(),
                OfflineIndeterminatePhysicalReason::SymlinkRefused,
            );
            self.completeness = OfflineIntegrityReportCompleteness::Indeterminate;
            Ok(Some(OfflineIndeterminatePhysicalReason::SymlinkRefused))
        }
    }

    fn bound(&mut self, reason: OfflineIndeterminatePhysicalReason) -> OfflineIntegrityOutcome {
        self.last_exhaustion = Some(reason);
        self.mark_bound_exhausted();
        OfflineIntegrityOutcome::Indeterminate(reason)
    }

    fn mark_bound_exhausted(&mut self) {
        if self.last_exhaustion.is_none() {
            self.last_exhaustion = Some(OfflineIndeterminatePhysicalReason::EntryBoundExceeded);
        }
        if self.completeness != OfflineIntegrityReportCompleteness::BoundExhausted {
            self.counters.exhausted_bounds += 1;
        }
        self.completeness = OfflineIntegrityReportCompleteness::BoundExhausted;
    }

    fn indeterminate_io(&mut self) -> OfflineIntegrityOutcome {
        self.completeness = OfflineIntegrityReportCompleteness::Indeterminate;
        OfflineIntegrityOutcome::Indeterminate(OfflineIndeterminatePhysicalReason::IoFailure)
    }
}

#[cfg(test)]
mod cached_source_tests;
#[cfg(test)]
mod source_change_tests;

fn incomplete_scan(
    entries: Vec<PathBuf>,
    reason: OfflineIndeterminatePhysicalReason,
) -> DirectoryScan {
    DirectoryScan {
        entries,
        incomplete_reason: Some(reason),
    }
}
