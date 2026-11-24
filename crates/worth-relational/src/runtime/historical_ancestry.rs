//! Bind historical ancestry verification to this owner's canonical history.
use super::RelationalRuntime;
use crate::branch::historical_selection::antecedent::HistoricalSelectionAncestry;
use std::sync::Arc;

impl<'a> From<&'a RelationalRuntime> for HistoricalSelectionAncestry<'a> {
    fn from(runtime: &'a RelationalRuntime) -> Self {
        Self {
            runtime_instance_id: runtime.runtime_instance_id(),
            history: &runtime.history,
        }
    }
}

impl<'a> From<&'a mut RelationalRuntime> for HistoricalSelectionAncestry<'a> {
    fn from(runtime: &'a mut RelationalRuntime) -> Self {
        (&*runtime).into()
    }
}

impl<'a> From<&'a Arc<RelationalRuntime>> for HistoricalSelectionAncestry<'a> {
    fn from(runtime: &'a Arc<RelationalRuntime>) -> Self {
        runtime.as_ref().into()
    }
}
