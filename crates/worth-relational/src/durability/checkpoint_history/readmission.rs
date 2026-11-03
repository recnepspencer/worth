//! A selected directory pins the installed physical read, never a later head.
use crate::durability::medium::{
    RelationalDurableColdSource, RelationalDurableMemberLocator,
    RelationalDurableOwnedReadRefusal as Refusal,
};
use std::{alloc::Layout, sync::Arc};
use worth_execution::{ExecutionImmutableBytes, ExecutionMemoryReservation, ExecutionRequest};
#[derive(Debug)]
struct Backing {
    source: Arc<dyn RelationalDurableColdSource>,
    original: crate::durability::medium::RelationalDurableMemberCustody,
    _memory: Option<ExecutionMemoryReservation>,
}
#[derive(Debug, Clone)]
pub(crate) struct SelectedCheckpointRead(Arc<Backing>);
impl SelectedCheckpointRead {
    pub(crate) fn preflight_retirement(
        &self,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<(), crate::durability::medium::RelationalDurableMemberRetirementRefusal> {
        self.0.original.preflight_retirement(source)
    }
    pub(crate) fn capture(
        source: Arc<dyn RelationalDurableColdSource>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        let bytes = Layout::new::<(usize, usize)>()
            .extend(Layout::new::<Backing>())
            .map_err(|_| Refusal::Bounds)?
            .0
            .pad_to_align()
            .size();
        let memory = request.reserve_retained_memory(bytes as u64)?;
        let original =
            crate::durability::medium::RelationalDurableMemberCustody::capture_selected_read(
                &source, request,
            )?;
        Ok(Self(Arc::new(Backing {
            source,
            original,
            _memory: memory,
        })))
    }
    pub(crate) fn read_member(
        &self,
        source: &Arc<dyn RelationalDurableColdSource>,
        request: &ExecutionRequest<'_, '_>,
        member: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, Refusal> {
        let bytes = self
            .0
            .original
            .read_selected_member(source, request, member, limit)?;
        Ok(bytes)
    }
    pub(crate) fn into_retirement(self) -> SelectedCheckpointReadRetirement {
        SelectedCheckpointReadRetirement {
            shared: Some(self.0),
            original: None,
            retired: None,
            data: None,
            complete: false,
        }
    }
}

#[derive(Debug)]
struct RetiredReadData {
    source: Option<Arc<dyn RelationalDurableColdSource>>,
    _memory: Option<ExecutionMemoryReservation>,
}
#[derive(Debug)]
pub(crate) struct SelectedCheckpointReadRetirement {
    shared: Option<Arc<Backing>>,
    original: Option<crate::durability::medium::RelationalDurableMemberRetirement>,
    retired: Option<crate::durability::medium::RetiredRelationalDurableMemberCustody>,
    data: Option<RetiredReadData>,
    complete: bool,
}
impl SelectedCheckpointReadRetirement {
    pub(crate) fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        source: &dyn RelationalDurableColdSource,
    ) -> crate::durability::medium::RelationalDurableMemberRetirementObservation {
        use crate::durability::medium::RelationalDurableMemberRetirementObservation as O;
        if self.complete {
            return O::Complete;
        }
        if let Err(cause) = request.consult() {
            return O::Stopped(cause);
        }
        if let Some(shared) = self.shared.take() {
            if let Some(Backing {
                source,
                original,
                _memory,
            }) = Arc::into_inner(shared)
            {
                self.original = Some(original.into_retirement());
                self.data = Some(RetiredReadData {
                    source: Some(source),
                    _memory,
                });
                return O::Advanced;
            }
            self.complete = true;
            return O::Complete;
        }
        let original = self
            .original
            .as_mut()
            .expect("owned selected read retains its actual original token");
        let observation = original.advance(request, source);
        if matches!(observation, O::Complete) {
            self.retired = Some(
                self.original
                    .take()
                    .expect("original retirement is owned")
                    .into_retired()
                    .unwrap_or_else(|_| {
                        unreachable!("complete retirement retains original inert data")
                    }),
            );
            self.complete = true;
            // The separately borrowed authentic source keeps this release nonfinal.
            self.data
                .as_mut()
                .expect("unique original read data")
                .source
                .take();
        }
        observation
    }
}
