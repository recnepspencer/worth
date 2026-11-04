//! Sealed test doors over this module's private members.
use super::*;

impl<'authority> QueryRequestExecution<'authority> {
    #[doc(hidden)]
    pub fn wave_for_test(&self) -> WaveRequestExecution<'authority> {
        Self::wave(self)
    }
}

impl<'authority> WaveRequestExecution<'authority> {
    #[doc(hidden)]
    pub fn member_for_test(&self) -> MemberRequestExecution<'authority> {
        Self::member(self)
    }
}

impl<'authority> WaveRequestExecution<'authority> {
    #[doc(hidden)]
    pub fn request_for_test(&self) -> &QueryRequestExecution<'_> {
        Self::request(self)
    }
}

impl MemberRequestExecution<'_> {
    #[doc(hidden)]
    pub fn request_for_test(&self) -> &QueryRequestExecution<'_> {
        Self::request(self)
    }
}
