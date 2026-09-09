use worth_ui_dsl::UiBackdropDeclaration;

use super::dependency_index::UiOverlayChangeSet;
use super::owner_bridge::UiOverlayCompositionOwnerBridge;
use super::planner::UiPreparedOverlayComposition;

pub(crate) use super::owner_bridge::{UiOverlayOwnerBridgeDenial, UiOverlayOwnerSources};

/// Gate 1's normal-library owner entry. It owns only the derived composition
/// coordinator; application/session and host lifecycles remain outside it.
pub(crate) struct UiOverlayCompositionOwnerLifecycle {
    bridge: UiOverlayCompositionOwnerBridge,
    last_counters: super::planner::UiOverlayPlanCounters,
}

impl UiOverlayCompositionOwnerLifecycle {
    /// Admit declarations and retain the first derived snapshot from one
    /// coherent set of current runtime-owner exports.
    pub(crate) fn admit_from_owners(
        declarations: impl IntoIterator<Item = UiBackdropDeclaration>,
        declaration_revision: u64,
        sources: UiOverlayOwnerSources<'_>,
    ) -> Result<Self, UiOverlayOwnerBridgeDenial> {
        let mut lifecycle = Self {
            bridge: UiOverlayCompositionOwnerBridge::admit(declarations, declaration_revision)?,
            last_counters: super::planner::UiOverlayPlanCounters::default(),
        };
        let initial = lifecycle.bridge.prepare_initial(sources)?;
        lifecycle.last_counters = initial.counters();
        lifecycle.bridge.retain_prepared(initial)?;
        Ok(lifecycle)
    }

    pub(crate) fn prepare_successor_from_owners(
        &self,
        sources: UiOverlayOwnerSources<'_>,
        changes: &UiOverlayChangeSet,
    ) -> Result<UiPreparedOverlayComposition, UiOverlayOwnerBridgeDenial> {
        self.bridge.prepare_successor(sources, changes)
    }

    pub(crate) fn reconstruct_from_owners(
        &self,
        sources: UiOverlayOwnerSources<'_>,
    ) -> Result<UiPreparedOverlayComposition, UiOverlayOwnerBridgeDenial> {
        self.bridge.reconstruct(sources)
    }

    pub(crate) fn retain_prepared(
        &mut self,
        prepared: UiPreparedOverlayComposition,
    ) -> Result<(), UiOverlayOwnerBridgeDenial> {
        self.last_counters = prepared.counters();
        self.bridge.retain_prepared(prepared)
    }

    pub(crate) fn current(&self) -> Option<&super::snapshot::UiOverlayStackSnapshot> {
        self.bridge.current()
    }

    pub(crate) const fn last_counters(&self) -> super::planner::UiOverlayPlanCounters {
        self.last_counters
    }
}
