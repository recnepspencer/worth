//! Moved whole from `domain_computation::primary_graph::admitted_capture`: the items below and every impl they own.
use worth_execution::MapKernelContext;
use worth_execution::MapKernelStop;
use worth_execution::ScopeStop;
use worth_execution::WorkCeilingDenial;

pub enum CaptureStop<E> {
    Request(WorkCeilingDenial),
    Kernel(ScopeStop<E>),
}

pub fn checkpoint(context: &mut MapKernelContext<'_, '_>, work: u64) -> Result<(), MapKernelStop> {
    if context.has_accounting() {
        context.checkpoint(work)
    } else {
        context.poll()
    }
}
