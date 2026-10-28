//! Original-request lane acquisition over the sole occurrence registry.
use super::*;
use worth_execution::{ExecutionOrderedMapDenial, ExecutionRequest, WorkCeilingDenial};

#[derive(Debug)]
pub(crate) enum BranchCoordinationDenial {
    Busy,
    SizeOverflow,
    WorkCounterOverflow,
    Request(WorkCeilingDenial),
    Storage(ExecutionOrderedMapDenial),
}

impl WorthQueryApplicationBranchCommitCoordinator {
    pub(super) fn lane_for_request(
        &self,
        occurrence: ProductBranchIncarnation,
        request: ExecutionRequest<'_, '_>,
    ) -> Result<Arc<WorthQueryApplicationBranchCommitLane>, BranchCoordinationDenial> {
        use BranchCoordinationDenial as Denial;
        request.consult().map_err(Denial::Request)?;
        let mut lanes = match self.lanes.try_lock() {
            Ok(guard) => guard,
            Err(TryLockError::Poisoned(error)) => error.into_inner(),
            Err(TryLockError::WouldBlock) => return Err(Denial::Busy),
        };
        let work = u64::try_from(lanes.len())
            .ok()
            .and_then(|n| n.checked_add(1))
            .and_then(|n| n.checked_mul(std::mem::size_of::<ProductBranchIncarnation>() as u64))
            .ok_or(Denial::WorkCounterOverflow)?;
        request
            .run_native_with_checkpoint(|context| {
                if context.has_accounting() {
                    context.checkpoint(work)
                } else {
                    context.poll()
                }
            })
            .map_err(Denial::Request)?
            .0
            .map_err(|stop| Denial::Request(WorkCeilingDenial::Stopped(stop)))?;
        if let Some(lane) = lanes.get(&occurrence).and_then(|slot| slot.lane.upgrade()) {
            request.consult().map_err(Denial::Request)?;
            return Ok(lane);
        }
        let quote = arc_bytes::<LaneFunding>()?
            .checked_add(arc_bytes::<WorthQueryApplicationBranchCommitLane>()?)
            .ok_or(Denial::SizeOverflow)?;
        let funding = Arc::new(LaneFunding {
            _memory: request
                .reserve_retained_memory(quote)
                .map_err(Denial::Request)?,
        });
        let lane = Arc::new(WorthQueryApplicationBranchCommitLane {
            contention: Arc::clone(&self.contention),
            occurrence,
            transition: Mutex::new(()),
            _funding: Some(Arc::clone(&funding)),
        });
        lanes
            .insert_cloned_native_for_request(
                &occurrence,
                &LaneSlot {
                    lane: Arc::downgrade(&lane),
                    _funding: Some(funding),
                },
                request,
            )
            .map_err(Denial::Storage)?;
        // Once inserted the roster owns the admitted node and weak backing.
        // A late stop returns no lane, while that honest registry custody
        // remains until replacement or occurrence retirement.
        request.consult().map_err(Denial::Request)?;
        Ok(lane)
    }
}

fn arc_bytes<T>() -> Result<u64, BranchCoordinationDenial> {
    std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .ok()
        .and_then(|(layout, _)| u64::try_from(layout.pad_to_align().size()).ok())
        .ok_or(BranchCoordinationDenial::SizeOverflow)
}
