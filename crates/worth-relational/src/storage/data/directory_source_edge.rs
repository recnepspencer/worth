//! A Native owner's directory edge retains the original retirement payload.
use crate::durability::medium::RelationalCheckpointAcknowledgementDenial as Ack;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

pub(crate) trait DirectoryRetirement: std::fmt::Debug + Send + Sync {
    fn advance(&mut self, request: &ExecutionRequest<'_, '_>) -> Result<bool, Ack>;
}

pub(crate) trait RetirableDirectory: std::fmt::Debug + Send + Sync {
    fn retire_last(self: Arc<Self>) -> Option<Box<dyn DirectoryRetirement>>;
}

#[derive(Debug)]
pub(crate) struct DirectorySourceEdge<D: ?Sized> {
    state: DirectoryEdgeState<D>,
}

#[derive(Debug)]
enum DirectoryEdgeState<D: ?Sized> {
    Serving(Arc<D>),
    Retiring(Box<dyn DirectoryRetirement>),
    Loaned,
    Complete {
        _inert: Option<Box<dyn DirectoryRetirement>>,
    },
}

pub(crate) struct NativeDirectoryRetirementHandoff {
    task: Box<dyn DirectoryRetirement>,
    observation: Option<Result<bool, Ack>>,
}
impl<D: RetirableDirectory + ?Sized> DirectorySourceEdge<D> {
    pub(crate) fn new(original: Arc<D>) -> Self {
        Self {
            state: DirectoryEdgeState::Serving(original),
        }
    }

    pub(crate) fn snapshot(&self) -> Option<Arc<D>> {
        match &self.state {
            DirectoryEdgeState::Serving(original) => Some(original.clone()),
            DirectoryEdgeState::Retiring(_)
            | DirectoryEdgeState::Loaned
            | DirectoryEdgeState::Complete { .. } => None,
        }
    }

    pub(crate) fn into_original(self) -> Arc<D> {
        let DirectoryEdgeState::Serving(original) = self.state else {
            panic!("only a serving displacement yields its directory");
        };
        original
    }

    /// The callback is loaned into the persistent close phase before this
    /// owner's gate is released; the unique retirement allocation is prepaid.
    pub(crate) fn loan(
        &mut self,
        handoff: &mut Option<NativeDirectoryRetirementHandoff>,
    ) -> Result<bool, Ack> {
        match &self.state {
            DirectoryEdgeState::Complete { .. } => return Ok(true),
            DirectoryEdgeState::Loaned => return Err(Ack::Busy),
            DirectoryEdgeState::Serving(_) | DirectoryEdgeState::Retiring(_) => {}
        }
        let task = match std::mem::replace(&mut self.state, DirectoryEdgeState::Loaned) {
            DirectoryEdgeState::Serving(original) => match original.retire_last() {
                Some(task) => task,
                None => {
                    self.state = DirectoryEdgeState::Complete { _inert: None };
                    return Ok(true);
                }
            },
            DirectoryEdgeState::Retiring(task) => task,
            DirectoryEdgeState::Loaned | DirectoryEdgeState::Complete { .. } => {
                unreachable!("only the serving or retiring edge loans custody")
            }
        };
        *handoff = Some(NativeDirectoryRetirementHandoff {
            task,
            observation: None,
        });
        Ok(false)
    }

    pub(crate) fn return_handoff(
        &mut self,
        handoff: &mut Option<NativeDirectoryRetirementHandoff>,
    ) -> Result<bool, Ack> {
        assert!(
            matches!(self.state, DirectoryEdgeState::Loaned),
            "only the exact loaned edge receives its task"
        );
        let NativeDirectoryRetirementHandoff { task, observation } =
            handoff.take().expect("original directory loan");
        let observation =
            observation.expect("callback observation retained before returning to owner");
        self.state = if matches!(observation, Ok(true)) {
            DirectoryEdgeState::Complete { _inert: Some(task) }
        } else {
            DirectoryEdgeState::Retiring(task)
        };
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
impl<D: ?Sized> std::ops::Deref for DirectorySourceEdge<D> {
    type Target = Arc<D>;
    fn deref(&self) -> &Self::Target {
        match &self.state {
            DirectoryEdgeState::Serving(original) => original,
            _ => panic!("only a serving owner reads its directory"),
        }
    }
}
impl<D: RetirableDirectory + ?Sized> Clone for DirectorySourceEdge<D> {
    fn clone(&self) -> Self {
        Self::new(std::ops::Deref::deref(self).clone())
    }
}
// Public checkpoint images expose serving directory DATA. A close loan owns
// the checkpoint exclusively; no public image aliases that retiring edge.
impl<D: RetirableDirectory + PartialEq + ?Sized> PartialEq for DirectorySourceEdge<D> {
    fn eq(&self, other: &Self) -> bool {
        std::ops::Deref::deref(self) == std::ops::Deref::deref(other)
    }
}
impl<D: RetirableDirectory + Eq + ?Sized> Eq for DirectorySourceEdge<D> {}
