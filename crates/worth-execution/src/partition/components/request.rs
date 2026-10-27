//! Checked component maintenance under the caller's actual request posture.
use super::{ComponentEdit, ComponentPartitioner};
use crate::{
    authority::{ExecutionMemoryReservation, ExecutionResourceLease, LeaseDenial},
    backend::{KernelContext, MemoryActivityGuard},
    partition::{
        checked_scope::{request_edit, OwnedRetainedCharge},
        PartitionUpdateDenial, PartitionWork,
    },
    ExecutionRequest, MapKernelStop, WorkCeilingDenial,
};

#[cfg(test)]
mod tests;

pub(super) struct ComponentFunding<'scope, 'authority> {
    lease: Option<&'scope ExecutionResourceLease<'authority>>,
    accounted: bool,
}

impl<'scope, 'authority> ComponentFunding<'scope, 'authority> {
    pub(super) fn leased(lease: &'scope ExecutionResourceLease<'authority>) -> Self {
        Self {
            lease: Some(lease),
            accounted: true,
        }
    }

    fn request(
        lease: Option<&'scope ExecutionResourceLease<'authority>>,
        context: &KernelContext<'_, '_>,
    ) -> Self {
        Self {
            lease,
            accounted: context.has_accounting(),
        }
    }

    pub(super) fn checkpoint(
        &self,
        context: &mut KernelContext<'_, '_>,
        units: u64,
    ) -> Result<(), PartitionUpdateDenial> {
        if self.accounted {
            context.checkpoint(units)
        } else {
            context.poll()
        }
        .map_err(PartitionUpdateDenial::Stop)
    }

    pub(super) fn retained(
        &self,
        charge: &mut OwnedRetainedCharge,
        bytes: u64,
    ) -> Result<(), PartitionUpdateDenial> {
        if !self.accounted {
            return if charge.is_bound() {
                Err(PartitionUpdateDenial::Admission(LeaseDenial::refused(
                    LeaseDenial::UnrelatedNestedLease,
                )))
            } else {
                Ok(())
            };
        }
        match self.lease {
            Some(lease) => charge.admit(lease, bytes),
            None => charge.admit_serial(bytes),
        }
    }

    pub(super) fn scratch(
        &self,
        bytes: u64,
    ) -> Result<Option<ExecutionMemoryReservation>, PartitionUpdateDenial> {
        if !self.accounted {
            return Ok(None);
        }
        ExecutionMemoryReservation::reserve_in_scope(self.lease, bytes)
            .map(Some)
            .map_err(PartitionUpdateDenial::Admission)
    }

    pub(super) fn activity(&self, bytes: u64) -> Option<MemoryActivityGuard> {
        self.accounted
            .then(|| crate::backend::enter_retained_memory(bytes))
            .flatten()
    }
}

impl ComponentPartitioner {
    /// Apply one checked edit with the original leased or serial controls.
    /// Accounted backing stays charged until the partitioner drops. Native
    /// unaccounted requests poll controls without inventing accounting and
    /// cannot edit a candidate charged to another request.
    ///
    /// Each edit retains its existing precommit checkpoint. A sequence of
    /// edits is not an atomic batch; publish its candidate only after success.
    pub fn edit_with_request(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        edit: ComponentEdit,
    ) -> Result<PartitionWork, PartitionUpdateDenial> {
        let mut settled = None;
        let handoff = execution
            .in_scope(|lease| {
                execution.run_native_with_checkpoint(|context| {
                    let funding = ComponentFunding::request(lease, context);
                    settled = Some(request_edit(context, |context| match edit {
                        ComponentEdit::UpsertItem(item) => {
                            self.upsert_item_admitted(&funding, context, item)
                        }
                        ComponentEdit::RemoveItem(item) => {
                            self.remove_item_admitted(&funding, context, item)
                        }
                        ComponentEdit::AddEdge(a, b) => {
                            self.add_edge_admitted(&funding, context, a, b)
                        }
                        ComponentEdit::RemoveEdge(a, b) => {
                            self.remove_edge_admitted(&funding, context, a, b)
                        }
                    }));
                })
            })
            .and_then(|inner| inner);
        match (handoff, settled) {
            (Ok(_), Some(result)) => result,
            (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(cause))) => {
                Err(cause)
            }
            (Err(WorkCeilingDenial::Admission(cause)), _) => {
                Err(PartitionUpdateDenial::Admission(cause))
            }
            (Err(WorkCeilingDenial::Stopped(cause)), _) => Err(PartitionUpdateDenial::Stop(cause)),
            (Err(WorkCeilingDenial::Panicked), _) => Err(PartitionUpdateDenial::KernelPanic),
            (Ok(_), None) => unreachable!("a successful component request invokes its edit"),
        }
    }
}
