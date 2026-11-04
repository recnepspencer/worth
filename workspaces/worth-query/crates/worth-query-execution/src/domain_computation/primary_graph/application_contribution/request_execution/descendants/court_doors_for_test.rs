//! Sealed test doors over this module's private members.
use super::*;

impl<'authority> QueryRequestExecution<'authority> {
    #[doc(hidden)]
    pub fn wave_for_test(&self) -> WaveRequestExecution<'authority> {
        Self::wave(self)
    }
}
