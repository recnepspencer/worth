//! Only physically closed native source evidence can leave its local worker.
use bank_domain::schema::BankSchema;
use worth_query_host::facade::{
    application_entry::WorthQueryClosedOperation,
    declaration::application_operation::{ApplicationMutationBinding, ApplicationMutationIntent},
};
pub(super) type SourceResult<I> =
    <<I as ApplicationMutationIntent<BankSchema>>::Binding as ApplicationMutationBinding<
        BankSchema,
    >>::Result;
pub(super) enum ClosedSourceEvidence<R> {
    Completion(bank_server::BankClosedSourceCompletion<R>),
    WithoutCompletion(WorthQueryClosedOperation),
}
impl<R> ClosedSourceEvidence<R> {
    pub(super) fn closed(&self) -> &WorthQueryClosedOperation {
        match self {
            Self::Completion(owner) => owner.closed(),
            Self::WithoutCompletion(owner) => owner,
        }
    }
    pub(super) fn closed_mut(&mut self) -> &mut WorthQueryClosedOperation {
        match self {
            Self::Completion(owner) => owner.closed_mut(),
            Self::WithoutCompletion(owner) => owner,
        }
    }
}
pub(super) enum ClosedCapabilityEvidence {
    Completion(bank_server::BankClosedCapabilityCommand),
    WithoutCompletion(WorthQueryClosedOperation),
}
impl ClosedCapabilityEvidence {
    pub(super) fn closed(&self) -> &WorthQueryClosedOperation {
        match self {
            Self::Completion(owner) => owner.closed(),
            Self::WithoutCompletion(owner) => owner,
        }
    }
    pub(super) fn closed_mut(&mut self) -> &mut WorthQueryClosedOperation {
        match self {
            Self::Completion(owner) => owner.closed_mut(),
            Self::WithoutCompletion(owner) => owner,
        }
    }
}
