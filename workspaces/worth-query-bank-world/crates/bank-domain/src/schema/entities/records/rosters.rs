//! Rosters of the bank model.
use super::*;
pub struct AccountRecords;
impl fragment::Fragment for AccountRecords {
    const NAME: &'static str = "AccountRecords";
    type Bindings = BankRecordsBindings;
    type Members = (
        (
            Institution,
            ExternalPrincipalMapping,
            Principal,
            Customer,
            Business,
            Account,
            AccountAuthorization,
            EmployeeAssignment,
        ),
        (DeploymentRecords,),
    );
}
pub struct PaymentRecords;
impl fragment::Fragment for PaymentRecords {
    const NAME: &'static str = "PaymentRecords";
    type Bindings = BankRecordsBindings;
    type Members = (
        PaymentIntent,
        Approval,
        JournalEntry,
        Posting,
        ApprovalRecords,
    );
}
pub struct EstateRecords;
impl fragment::Fragment for EstateRecords {
    const NAME: &'static str = "EstateRecords";
    type Bindings = BankRecordsBindings;
    type Members = (
        Branch,
        CapabilityGrant,
        DeathNotice,
        EmergencyAccess,
        EstateCase,
        LegalAuthority,
        MandatoryReview,
    );
}
