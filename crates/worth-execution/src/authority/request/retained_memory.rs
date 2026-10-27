//! Original-request admission of retention, shared by real backing owners.
use super::{ExecutionRequest, RequestBacking};
use crate::{ExecutionMemoryReservation, MapKernelStop, WorkCeilingDenial};

impl ExecutionRequest<'_, '_> {
    /// Admit known retained backing in the effective original context before
    /// allocation. A native-only context has no accounting ticket. The returned
    /// sealed reservation must live with that backing until its final drop.
    pub fn reserve_retained_memory(
        self,
        bytes: u64,
    ) -> Result<Option<ExecutionMemoryReservation>, WorkCeilingDenial> {
        let mut held = None;
        let outcome = self.run_native_with_checkpoint(|context| {
            let admission = (|| {
                context.poll().map_err(WorkCeilingDenial::Stopped)?;
                if !context.has_accounting() {
                    Ok(None)
                } else {
                    let reservation = match self.form {
                        RequestBacking::Leased(lease) => {
                            let active = lease.current_descendant();
                            ExecutionMemoryReservation::reserve_in_scope(
                                Some(active.as_ref().unwrap_or(lease)),
                                bytes,
                            )
                        }
                        RequestBacking::Serial(_) => {
                            ExecutionMemoryReservation::reserve_in_scope(None, bytes)
                        }
                    };
                    reservation.map(Some).map_err(WorkCeilingDenial::Admission)
                }
            })();
            held = Some(admission);
        });
        match (outcome, held) {
            (Ok(_), Some(held)) => held,
            // A real classified refusal can mark the enclosing scope stopped.
            // Only that generic handoff may preserve the earlier exact cause.
            (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(cause))) => {
                Err(cause)
            }
            (Err(cause), _) => Err(cause),
            (Ok(_), None) => unreachable!("completed original-request callback"),
        }
    }
}
