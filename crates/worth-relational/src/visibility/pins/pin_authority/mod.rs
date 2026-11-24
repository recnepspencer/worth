mod branch_head_residency;
mod replay_authority;
pub(super) mod replay_pins;

use crate::runtime::RelationalRuntime;

pub(crate) struct VisibilityPinAuthority<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

impl<'runtime> VisibilityPinAuthority<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }
}

impl RelationalRuntime {
    pub(crate) fn visibility_pins(&self) -> VisibilityPinAuthority<'_> {
        VisibilityPinAuthority::new(self)
    }
}
