//! Joined facts retain the original funded child array of the exact payload.
use super::FactPayload;
use std::fmt::Debug;
use std::ops::Deref;
use worth_execution::ExecutionArray;

pub(super) trait JoinedFactBacking: Debug + Send + Sync {
    fn children(&self) -> &ExecutionArray<FactPayload>;
}

impl JoinedFactBacking for ExecutionArray<FactPayload> {
    fn children(&self) -> &ExecutionArray<FactPayload> {
        self
    }
}

impl Deref for dyn JoinedFactBacking {
    type Target = ExecutionArray<FactPayload>;

    fn deref(&self) -> &Self::Target {
        self.children()
    }
}
