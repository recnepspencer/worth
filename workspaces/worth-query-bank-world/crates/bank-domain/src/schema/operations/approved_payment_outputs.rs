//! The payment and journal authored by one admitted approval.
use crate::schema::{BankSchema, JournalEntry, PaymentIntent};
use worth_query_decl::facade::application_operation::{
    ApplicationMutationOutputContract, ApplicationMutationOutputRoleDescriptor,
    WorthQueryApplicationDeclaredOutputRole, WorthQueryApplicationOutputRole,
    WorthQueryCreateOutput, WorthQueryExactlyOneOutput, WorthQueryPreserveOutput,
};

pub struct ApprovedPaymentOutputs;
pub struct ApprovedPaymentJournal;
pub struct ApprovedPaymentOutput;

impl WorthQueryApplicationOutputRole for ApprovedPaymentJournal {
    type Schema = BankSchema;
    type Contract = ApprovedPaymentOutputs;
    type Entity = JournalEntry;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "approved-payment-journal";
}
impl WorthQueryApplicationOutputRole for ApprovedPaymentOutput {
    type Schema = BankSchema;
    type Contract = ApprovedPaymentOutputs;
    type Entity = PaymentIntent;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "payment";
}
impl ApplicationMutationOutputContract<BankSchema> for ApprovedPaymentOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
        ApprovedPaymentOutput::DESCRIPTOR,
        ApprovedPaymentJournal::DESCRIPTOR,
    ];
}
