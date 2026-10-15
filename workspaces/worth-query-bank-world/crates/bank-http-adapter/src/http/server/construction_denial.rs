//! Transport declines unexecuted construction; no post-effect custody is discharged.
use super::super::protocol::{
    BankHttpDenial, BankHttpDenialKind as Kind, BankHttpNextAction as Next,
};
use worth_query_host::facade::{
    application_entry::WorthQueryOperationInputCaptureDenial as Capture,
    declaration::application_operation::ApplicationMutationIntent,
    primary_graph::WorthQueryProviderSessionControlStopKind as Control,
    runtime::{
        ExecutionAllocationDenialClass as Allocation,
        ExecutionPhysicalAllocationDenialKind as Physical, LeaseDenial, MapKernelStop,
        MemoryLimitLevel, WorkCeilingDenial as Request,
    },
};
pub(super) fn construction<I: ApplicationMutationIntent<bank_domain::schema::BankSchema>>(
    refusal: bank_server::BankOperationConstructionRefusal<I>,
) -> BankHttpDenial {
    match &refusal {
        bank_server::BankOperationConstructionRefusal::Capture(owner) => capture(owner.denial()),
        bank_server::BankOperationConstructionRefusal::Declaration { .. } => {
            super::mutation_refusal::denial(Kind::InternalDenied)
        }
    }
    // Dropping this pre-effect owner declines the command; it neither releases
    // a native failed run nor claims that a workflow slot or business effect ran.
}
fn capture(cause: &Capture) -> BankHttpDenial {
    match cause {
        Capture::Execution(cause) => super::advancement_denial::advancement(*cause),
        Capture::Encoding(_) => BankHttpDenial::new(Kind::MalformedRequest, Next::CorrectRequest),
        Capture::Allocation(cause) => {
            let (kind, next) = match cause.classify() {
                Allocation::Request(Request::Stopped(MapKernelStop::Cancelled)) => {
                    return super::mutation_application::pending_execution_stop(Err(
                        Control::Cancelled,
                    ));
                }
                Allocation::Request(Request::Stopped(MapKernelStop::DeadlineElapsed)) => {
                    return super::mutation_application::pending_execution_stop(Err(
                        Control::TimedOut,
                    ));
                }
                Allocation::Request(Request::Admission(LeaseDenial::MemoryExhausted(limit))) => {
                    match limit.level {
                        MemoryLimitLevel::Process => (Kind::Unavailable, Next::Retry),
                        MemoryLimitLevel::Policy { .. } | MemoryLimitLevel::Declared => {
                            (Kind::ResourceExhausted, Next::NarrowRequest)
                        }
                    }
                }
                Allocation::Physical(Physical::Allocator) => (Kind::Unavailable, Next::Retry),
                Allocation::Request(_)
                | Allocation::Physical(
                    Physical::Layout
                    | Physical::CapacityMismatch
                    | Physical::WriteBeyondReserved
                    | Physical::IncompleteSeal,
                ) => (Kind::InternalDenied, Next::ContactOperator),
            };
            BankHttpDenial::new(kind, next)
        }
    }
}
