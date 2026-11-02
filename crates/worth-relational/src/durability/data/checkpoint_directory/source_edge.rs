//! A Native owner's directory edge retains the original retirement payload.
use super::{CheckpointDirectoryRetirement, RelationalCheckpointDirectory};
use crate::durability::medium::RelationalCheckpointAcknowledgementDenial as Ack;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

#[derive(Debug)]
pub(crate) struct NativeCheckpointDirectoryEdge {
    original: Option<Arc<RelationalCheckpointDirectory>>,
    retirement: Option<CheckpointDirectoryRetirement>,
    loaned: bool,
    complete: bool,
}
pub(crate) struct NativeDirectoryRetirementHandoff {
    task: CheckpointDirectoryRetirement,
    observation: Option<Result<bool, Ack>>,
}
impl NativeCheckpointDirectoryEdge {
    pub(crate) fn new(original: Arc<RelationalCheckpointDirectory>) -> Self {
        Self {
            original: Some(original),
            retirement: None,
            loaned: false,
            complete: false,
        }
    }
    pub(crate) fn snapshot(&self) -> Option<Arc<RelationalCheckpointDirectory>> {
        self.original.clone()
    }
    pub(crate) fn into_original(mut self) -> Arc<RelationalCheckpointDirectory> {
        self.original
            .take()
            .expect("only a serving displacement yields its directory")
    }
    /// Allocation is funded before this owner's gate is entered. The callback
    /// is loaned into the persistent close phase before that gate is released.
    pub(crate) fn loan(
        &mut self,
        handoff: &mut Option<NativeDirectoryRetirementHandoff>,
    ) -> Result<bool, Ack> {
        if self.complete {
            return Ok(true);
        }
        if self.loaned {
            return Err(Ack::Busy);
        }
        if let Some(original) = self.original.take() {
            match Arc::into_inner(original) {
                Some(original) => {
                    self.retirement = Some(original.into_retirement());
                }
                None => {
                    self.complete = true;
                    return Ok(true);
                }
            }
        }
        *handoff = Some(NativeDirectoryRetirementHandoff {
            task: self
                .retirement
                .take()
                .expect("incomplete edge retains its same original task"),
            observation: None,
        });
        self.loaned = true;
        Ok(false)
    }
    pub(crate) fn return_handoff(
        &mut self,
        handoff: &mut Option<NativeDirectoryRetirementHandoff>,
    ) -> Result<bool, Ack> {
        assert!(self.loaned, "only the exact loaned edge receives its task");
        let NativeDirectoryRetirementHandoff { task, observation } =
            handoff.take().expect("original directory loan");
        self.retirement = Some(task);
        self.loaned = false;
        let observation =
            observation.expect("callback observation retained before returning to owner");
        if matches!(observation, Ok(true)) {
            self.complete = true;
        }
        observation
    }
}
impl NativeDirectoryRetirementHandoff {
    pub(crate) fn observe(&mut self, request: &ExecutionRequest<'_, '_>) {
        if self.observation.is_none() {
            self.observation = Some(self.task.advance(request));
        }
    }
}
impl std::ops::Deref for NativeCheckpointDirectoryEdge {
    type Target = Arc<RelationalCheckpointDirectory>;
    fn deref(&self) -> &Self::Target {
        self.original
            .as_ref()
            .expect("only a serving owner reads its directory")
    }
}
impl Clone for NativeCheckpointDirectoryEdge {
    fn clone(&self) -> Self {
        Self::new(self.deref_original())
    }
}
impl NativeCheckpointDirectoryEdge {
    fn deref_original(&self) -> Arc<RelationalCheckpointDirectory> {
        self.original
            .as_ref()
            .expect("only serving directories are cloned")
            .clone()
    }
}
