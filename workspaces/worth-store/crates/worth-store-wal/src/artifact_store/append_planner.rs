use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use super::prefix_scan::{
    scan_segment_path_with_buffer, scan_segment_suffix_with_buffer, WalPrefixScan,
    WAL_SCAN_BUFFER_BYTES,
};
use super::{WalArtifactStoreDenial, WalFrameAppendPlan};

/// Reuses one validated WAL prefix across ordinary appends.
///
/// Opening performs the reconstructive whole-prefix walk. Later preparations
/// validate only bytes appended since the last observation. The physical
/// backend still compares the observed length under its append lock, so a
/// concurrent writer causes a bounded replan instead of a lost append.
#[derive(Debug)]
pub struct WalAppendPlanner {
    root: PathBuf,
    segment_id: u64,
    generation: u64,
    opening_prefix_bytes_scanned: u64,
    state: Mutex<WalPlannerState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Why a path-bound WAL append planner could not produce the next plan.
pub enum WalAppendPlannerDenial {
    /// Durable artifact inspection or frame construction failed with this cause.
    Artifact(WalArtifactStoreDenial),
    /// A panic poisoned planner state; discard this planner and reopen a fresh one.
    StatePoisoned,
}

impl From<WalArtifactStoreDenial> for WalAppendPlannerDenial {
    fn from(denial: WalArtifactStoreDenial) -> Self {
        Self::Artifact(denial)
    }
}

#[derive(Debug)]
struct WalPlannerState {
    tail: WalPrefixScan,
    scan_buffer: Vec<u8>,
}

impl WalAppendPlanner {
    pub fn open(
        root: impl Into<PathBuf>,
        segment_id: u64,
        generation: u64,
    ) -> Result<Self, WalAppendPlannerDenial> {
        if segment_id == 0 || generation == 0 {
            return Err(WalArtifactStoreDenial::InvalidFrame.into());
        }
        let root = root.into();
        let mut scan_buffer = vec![0; WAL_SCAN_BUFFER_BYTES];
        let tail = scan_segment_path_with_buffer(&root, segment_id, generation, &mut scan_buffer)?;
        Ok(Self {
            root,
            segment_id,
            generation,
            opening_prefix_bytes_scanned: tail.bytes_scanned,
            state: Mutex::new(WalPlannerState { tail, scan_buffer }),
        })
    }

    pub fn prepare_append(
        &self,
        lsn_start: u64,
        lsn_end: u64,
        declared_digest: &str,
        payload: &[u8],
    ) -> Result<WalFrameAppendPlan, WalAppendPlannerDenial> {
        let mut state = lock_state(&self.state)?;
        let prior = state.tail;
        state.tail = scan_segment_suffix_with_buffer(
            &self.root,
            self.segment_id,
            self.generation,
            prior.valid_prefix_bytes,
            prior.last_lsn_end,
            &mut state.scan_buffer,
        )?;
        Ok(super::encode_append(
            self.segment_id,
            self.generation,
            lsn_start,
            lsn_end,
            declared_digest,
            payload,
            state.tail,
        )?)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub const fn segment_id(&self) -> u64 {
        self.segment_id
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn opening_prefix_bytes_scanned(&self) -> u64 {
        self.opening_prefix_bytes_scanned
    }

    pub const fn scan_buffer_bytes(&self) -> u64 {
        WAL_SCAN_BUFFER_BYTES as u64
    }
}

fn lock_state(
    state: &Mutex<WalPlannerState>,
) -> Result<MutexGuard<'_, WalPlannerState>, WalAppendPlannerDenial> {
    state
        .lock()
        .map_err(|_| WalAppendPlannerDenial::StatePoisoned)
}

#[cfg(test)]
mod lock_failure_tests {
    use super::*;

    #[test]
    fn poisoned_planner_state_is_not_reported_as_media_io() {
        let state = Mutex::new(WalPlannerState {
            tail: WalPrefixScan {
                valid_prefix_bytes: 0,
                observed_file_bytes: 0,
                last_lsn_end: None,
                bytes_scanned: 0,
            },
            scan_buffer: Vec::new(),
        });
        let _ = std::panic::catch_unwind(|| {
            let _locked = state.lock().expect("acquire planner state");
            panic!("poison planner state");
        });
        let denial = match lock_state(&state) {
            Ok(_) => panic!("poisoned planner state was accepted"),
            Err(denial) => denial,
        };
        assert_eq!(
            denial,
            WalAppendPlannerDenial::StatePoisoned,
            "MUTANT_PREDICATE:wal-planner-poison-misclassified"
        );
    }
}
