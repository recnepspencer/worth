//! Derived spatial coverage for sampled surfaces and scrollbar parts. The
//! sample transaction carries its inverse; replay never scans all overrides.
use super::{UiNativeRetainedDrawList, UiNativeRetainedDrawListDenial as Denial};
use crate::native::presentation::damage_index::{UiNativeDamageIndex, UiNativeDamageQuery};
use std::collections::HashMap;
use worth_ui_host_contract::{UiMountedCanonicalBox, UiMountedPaintCommandIdentity};

pub(super) type CoverageUndo = Vec<(UiMountedPaintCommandIdentity, Option<UiMountedCanonicalBox>)>;

pub(super) struct UiSampledAppearanceCoverage {
    bounds: HashMap<UiMountedPaintCommandIdentity, UiMountedCanonicalBox>,
    index: UiNativeDamageIndex<UiMountedPaintCommandIdentity>,
}

impl UiSampledAppearanceCoverage {
    pub(super) fn new() -> Self {
        Self {
            bounds: HashMap::new(),
            index: UiNativeDamageIndex::new(),
        }
    }

    pub(super) fn snapshot(
        &self,
        identities: impl IntoIterator<Item = UiMountedPaintCommandIdentity>,
    ) -> CoverageUndo {
        identities
            .into_iter()
            .filter(|id| id.is_appearance_sample())
            .map(|id| (id, self.bounds.get(&id).copied()))
            .collect()
    }

    pub(super) fn replace(
        &mut self,
        identity: UiMountedPaintCommandIdentity,
        bounds: Option<UiMountedCanonicalBox>,
    ) -> Result<(), Denial> {
        let previous = self.bounds.get(&identity).copied();
        if previous == bounds {
            return Ok(());
        }
        super::mutation::update_damage(&mut self.index, identity, previous, bounds)?;
        if let Some(bounds) = bounds {
            self.bounds.insert(identity, bounds);
        } else {
            self.bounds.remove(&identity);
        }
        Ok(())
    }

    pub(super) fn restore(&mut self, undo: CoverageUndo) -> Result<(), Denial> {
        // Release successor membership before restoring a full predecessor.
        for (identity, _) in &undo {
            self.replace(*identity, None)?;
        }
        for (identity, bounds) in undo {
            self.replace(identity, bounds)?;
        }
        Ok(())
    }

    pub(super) fn intersecting(
        &self,
        bounds: UiMountedCanonicalBox,
    ) -> Result<UiNativeDamageQuery<UiMountedPaintCommandIdentity>, Denial> {
        Ok(self.index.intersecting(bounds)?)
    }
}

impl UiNativeRetainedDrawList {
    pub(super) fn reconstruct_sampled_appearance_coverage(&mut self) -> Result<(), Denial> {
        let mut coverage = UiSampledAppearanceCoverage::new();
        for (identity, change) in &self.sample_overrides {
            if identity.is_appearance_sample() {
                coverage.replace(
                    *identity,
                    self.sampled_target_bounds(*identity, Some(*change))?,
                )?;
            }
        }
        self.sampled_appearance = coverage;
        Ok(())
    }
}
