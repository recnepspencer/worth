//! Shared checkpoint and associated-effect access to one real runtime.
use crate::runtime::RelationalRuntime;

pub struct DurabilityAuthority<'runtime> {
    pub(super) runtime: &'runtime RelationalRuntime,
}

impl<'runtime> DurabilityAuthority<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }
}
