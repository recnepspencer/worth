mod branch_head_residency;
mod replay_authority;
pub(crate) use worth_relational_branch::internal_port::visibility::pins::pin_authority::replay_pins;

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
