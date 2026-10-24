//! Transport declines unexecuted construction; no post-effect custody is discharged.
use super::super::protocol::{
    BankHttpDenial, BankHttpDenialKind as Kind, BankHttpNextAction as Next,
};
use worth_query_host::facade::{
    application_entry::WorthQueryOperationInputCaptureDenial as Capture,
    declaration::application_operation::ApplicationMutationIntent,
    runtime::ExecutionAllocationDenialClass as Allocation,
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
pub(super) fn capture(cause: &Capture) -> BankHttpDenial {
    match cause {
        Capture::Execution(cause) => super::advancement_denial::advancement(*cause),
        Capture::Encoding(_) => BankHttpDenial::new(Kind::MalformedRequest, Next::CorrectRequest),
        Capture::Allocation(cause) => match cause.classify() {
            Allocation::Request(cause) => super::advancement_denial::request(cause),
            Allocation::Physical(cause) => super::advancement_denial::physical_allocation(cause),
        },
    }
}
