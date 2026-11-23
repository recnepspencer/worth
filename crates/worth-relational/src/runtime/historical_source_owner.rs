//! Preserve the runtime borrow while lending exact owner affinity to Branch.
use super::RelationalRuntime;
use crate::branch::historical_selection::source_presence::HistoricalSourceOwner;

impl<'owner> From<&'owner RelationalRuntime> for HistoricalSourceOwner<'owner> {
    fn from(runtime: &'owner RelationalRuntime) -> Self {
        Self::bound(runtime.runtime_instance_id())
    }
}
