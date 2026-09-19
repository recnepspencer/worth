//! Exact physical damage and ordered retained selections for staged replay.
use super::command::UiNativeAppearanceCommandKey;
use super::damage::UiNativeAppearanceDamageRect;
use super::retained::{UiNativeAppearanceRetained, UiNativeAppearanceRetainedDenial};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UiNativeAppearanceReplayRegion {
    pub(crate) damage: UiNativeAppearanceDamageRect,
    pub(crate) replay: Box<[UiNativeAppearanceCommandKey]>,
}

impl UiNativeAppearanceRetained {
    /// Drain only inside the caller's reversible presentation transaction.
    /// Its existing text inverse restores pending damage if preparation fails.
    pub(crate) fn prepare_replay(
        &mut self,
    ) -> Result<Box<[UiNativeAppearanceReplayRegion]>, UiNativeAppearanceRetainedDenial> {
        self.take_damage()
            .iter()
            .map(|&damage| {
                let replay = self.replay_for_damage(damage)?;
                Ok(UiNativeAppearanceReplayRegion { damage, replay })
            })
            .collect()
    }
}
