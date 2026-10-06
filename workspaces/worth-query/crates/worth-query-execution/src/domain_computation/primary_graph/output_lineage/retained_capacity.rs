//! Aggregate host custody for retained output correspondence.

use std::sync::{Arc, Mutex};

use super::super::{WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind};

pub(super) struct LineageRetentionLedger {
    state: Arc<Mutex<LineageRetentionState>>,
    /// The invalidation window: how many of an occurrence's newest
    /// generations its readers may still select. Unset until installation.
    history_positions: Option<std::num::NonZeroUsize>,
}

struct LineageRetentionState {
    maximum_bytes: u64,
    retained_bytes: u64,
}

pub(super) struct RetainedLineageCapacity {
    state: Arc<Mutex<LineageRetentionState>>,
    bytes: u64,
}

impl LineageRetentionLedger {
    pub(super) fn new(maximum_bytes: u64) -> Self {
        Self {
            state: Arc::new(Mutex::new(LineageRetentionState {
                maximum_bytes,
                retained_bytes: 0,
            })),
            history_positions: None,
        }
    }

    pub(super) fn install(
        &mut self,
        maximum_bytes: u64,
        history_positions: std::num::NonZeroUsize,
    ) {
        self.history_positions = Some(history_positions);
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(state.retained_bytes <= maximum_bytes);
        state.maximum_bytes = maximum_bytes;
    }

    pub(super) const fn history_positions(&self) -> Option<std::num::NonZeroUsize> {
        self.history_positions
    }

    pub(super) fn reserve(
        &self,
        bytes: u64,
    ) -> Result<RetainedLineageCapacity, WorthQueryOutputDemandDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let required = state.retained_bytes.checked_add(bytes).ok_or_else(denial)?;
        if required > state.maximum_bytes {
            return Err(denial());
        }
        state.retained_bytes = required;
        Ok(RetainedLineageCapacity {
            state: Arc::clone(&self.state),
            bytes,
        })
    }
}

#[cfg(feature = "test-query-execution-observer")]
impl LineageRetentionLedger {
    pub(super) fn retained_bytes(&self) -> u64 {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retained_bytes
    }
}

impl Default for LineageRetentionLedger {
    fn default() -> Self {
        let maximum = super::super::super::execution_runtime::WorthQueryOutputDemandResourceProfile::standard()
            .lineage_retained_bytes();
        Self::new(maximum as u64)
    }
}

impl RetainedLineageCapacity {
    /// Extend this same owner's custody before allocating its next prepared
    /// structure. A denial preserves the existing reservation unchanged.
    pub(super) fn reserve_additional(
        &mut self,
        bytes: u64,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let required = state.retained_bytes.checked_add(bytes).ok_or_else(denial)?;
        let owned = self.bytes.checked_add(bytes).ok_or_else(denial)?;
        if required > state.maximum_bytes {
            return Err(denial());
        }
        state.retained_bytes = required;
        self.bytes = owned;
        Ok(())
    }
}

impl Drop for RetainedLineageCapacity {
    fn drop(&mut self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.retained_bytes = state
            .retained_bytes
            .checked_sub(self.bytes)
            .expect("retained lineage capacity has one owner");
    }
}

fn denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "retained output lineage capacity is exhausted",
    )
}
