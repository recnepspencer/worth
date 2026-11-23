mod pinning;
mod publication;
mod retention;
mod retention_step;
pub use retention_step::RelationalRetentionPassStop;

use crate::runtime::RelationalRuntime;

pub struct StorageAuthority<'runtime> {
    runtime: &'runtime RelationalRuntime,
}

impl RelationalRuntime {
    pub(crate) fn storage_authority(&self) -> StorageAuthority<'_> {
        StorageAuthority::new(self)
    }
}

impl<'runtime> StorageAuthority<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }
}
