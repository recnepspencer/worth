//! No borrowed initiating scope escapes failed ordinary construction.
use bank_domain::schema::BankSchema;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryOperationConstructionDenial, WorthQueryOperationDescription,
        WorthQueryOperationInputCaptureRefusal,
    },
    declaration::application_operation::ApplicationMutationIntent,
};
#[must_use]
pub enum BankOperationConstructionRefusal<I: ApplicationMutationIntent<BankSchema>> {
    Capture(WorthQueryOperationInputCaptureRefusal<BankSchema, I>),
    Declaration {
        description: WorthQueryOperationDescription<BankSchema, I>,
        denial: WorthQueryOperationConstructionDenial,
    },
}
impl<I: ApplicationMutationIntent<BankSchema>> std::fmt::Debug
    for BankOperationConstructionRefusal<I>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Capture(owner) => f.debug_tuple("Capture").field(owner.denial()).finish(),
            Self::Declaration { denial, .. } => f.debug_tuple("Declaration").field(denial).finish(),
        }
    }
}
