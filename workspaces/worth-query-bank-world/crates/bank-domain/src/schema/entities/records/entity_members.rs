//! Entity members of the bank model.
use super::*;
pub struct Branch;
impl fragment::Entity for Branch {
    const NAME: &'static str = "Branch";
    type Aspects = (BranchIdentity,);
    type Relations = (BranchInstitution,);
}
impl Branch {
    pub fn reference() -> ApplicationEntityRef<BankSchema, Self> {
        entity(EstateRecords, Self)
    }
}
pub struct CapabilityGrant;
impl fragment::Entity for CapabilityGrant {
    const NAME: &'static str = "CapabilityGrant";
    type Aspects = (CapabilityGrantRecord,);
    type Relations = (
        CapabilityEstate,
        CapabilityAccount,
        CapabilityInstitution,
        CapabilityBranch,
        CapabilityParent,
    );
}
impl CapabilityGrant {
    pub fn reference() -> ApplicationEntityRef<BankSchema, Self> {
        entity(EstateRecords, Self)
    }
}
pub struct DeathNotice;
impl fragment::Entity for DeathNotice {
    const NAME: &'static str = "DeathNotice";
    type Aspects = (DeathNoticeRecord,);
    type Relations = (DeathNoticeSubject,);
}
impl DeathNotice {
    pub fn reference() -> ApplicationEntityRef<BankSchema, Self> {
        entity(EstateRecords, Self)
    }
}
pub struct EmergencyAccess;
impl fragment::Entity for EmergencyAccess {
    const NAME: &'static str = "EmergencyAccess";
    type Aspects = (EmergencyAccessRecord,);
    type Relations = (EmergencyGrant, EmergencyEstate, EmergencyReview);
}
impl EmergencyAccess {
    pub fn reference() -> ApplicationEntityRef<BankSchema, Self> {
        entity(EstateRecords, Self)
    }
}
pub struct EstateCase;
impl fragment::Entity for EstateCase {
    const NAME: &'static str = "EstateCase";
    type Aspects = (EstateCaseRecord,);
    type Relations = (
        EstateDeathNotice,
        EstateDeceased,
        EstateAccount,
        EstateBranch,
    );
}
impl EstateCase {
    pub fn reference() -> ApplicationEntityRef<BankSchema, Self> {
        entity(EstateRecords, Self)
    }
}
pub struct LegalAuthority;
impl fragment::Entity for LegalAuthority {
    const NAME: &'static str = "LegalAuthority";
    type Aspects = (LegalAuthorityRecord,);
    type Relations = (LegalAuthorityEstate, LegalAuthorityHolder);
}
impl LegalAuthority {
    pub fn reference() -> ApplicationEntityRef<BankSchema, Self> {
        entity(EstateRecords, Self)
    }
}
pub struct MandatoryReview;
impl fragment::Entity for MandatoryReview {
    const NAME: &'static str = "MandatoryReview";
    type Aspects = (MandatoryReviewRecord,);
    type Relations = (ReviewEstate,);
}
impl MandatoryReview {
    pub fn reference() -> ApplicationEntityRef<BankSchema, Self> {
        entity(EstateRecords, Self)
    }
}
pub struct Institution;
impl fragment::Entity for Institution {
    const NAME: &'static str = "Institution";
    type Aspects = (InstitutionIdentity,);
    type Relations = (
        InstitutionEmployee,
        InstitutionAccount,
        InstitutionCashAccount,
    );
}
impl Institution {
    pub fn reference() -> ApplicationEntityRef<BankSchema, Self> {
        entity(AccountRecords, Self)
    }
}
pub struct ExternalPrincipalMapping;
impl fragment::Entity for ExternalPrincipalMapping {
    const NAME: &'static str = "ExternalPrincipalMapping";
    type Aspects = (ExternalPrincipalIdentity,);
    type Relations = (ExternalPrincipal,);
}
impl ExternalPrincipalMapping {
    pub fn reference() -> ApplicationEntityRef<BankSchema, Self> {
        entity(AccountRecords, Self)
    }
}
pub struct Principal;
impl fragment::Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalIdentity,);
    type Relations = (
        (
            PrincipalCustomer,
            PersonalOwner,
            AccountAuthorizedUser,
            PaymentInitiator,
            EstateExecutor,
            EstateBeneficiary,
            EstateJointOwner,
            EstateAuthorizedSigner,
        ),
        (
            CapabilityGrantee,
            CapabilityGrantor,
            EmergencyRequester,
            EmergencyApprover,
            ReviewPrincipal,
            ApprovedBusinessPaymentGrantGrantor,
            ApprovedBusinessPaymentGrantGrantee,
            ProgramDeploymentGrantGrantor,
        ),
        (ProgramDeploymentGrantGrantee,),
    );
}
impl Principal {
    pub fn reference() -> ApplicationEntityRef<BankSchema, Self> {
        entity(AccountRecords, Self)
    }
}
pub struct Customer;
impl fragment::Entity for Customer {
    const NAME: &'static str = "Customer";
    type Aspects = ();
    type Relations = ();
}
impl Customer {
    pub fn reference() -> ApplicationEntityRef<BankSchema, Self> {
        entity(AccountRecords, Self)
    }
}
pub struct Business;
impl fragment::Entity for Business {
    const NAME: &'static str = "Business";
    type Aspects = (BusinessIdentity,);
    type Relations = (BusinessOwner, BusinessAccount);
}
impl Business {
    pub fn reference() -> ApplicationEntityRef<BankSchema, Self> {
        entity(AccountRecords, Self)
    }
}
pub struct Account;
impl fragment::Entity for Account {
    const NAME: &'static str = "Account";
    type Aspects = (Identity, AccountProfile, AccountState);
    type Relations = ();
}
impl Account {
    pub fn reference() -> ApplicationEntityRef<BankSchema, Self> {
        entity(AccountRecords, Self)
    }
}
pub struct AccountAuthorization;
impl fragment::Entity for AccountAuthorization {
    const NAME: &'static str = "AccountAuthorization";
    type Aspects = (AuthorizationScope, AuthorizationIdentity);
    type Relations = (AuthorizationAccount,);
}
impl AccountAuthorization {
    pub fn reference() -> ApplicationEntityRef<BankSchema, Self> {
        entity(AccountRecords, Self)
    }
}
pub struct EmployeeAssignment;
impl fragment::Entity for EmployeeAssignment {
    const NAME: &'static str = "EmployeeAssignment";
    type Aspects = (EmployeeScope,);
    type Relations = (AssignmentPrincipal, EstateAssignment);
}
impl EmployeeAssignment {
    pub fn reference() -> ApplicationEntityRef<BankSchema, Self> {
        entity(AccountRecords, Self)
    }
}
pub struct PaymentIntent;
impl fragment::Entity for PaymentIntent {
    const NAME: &'static str = "PaymentIntent";
    type Aspects = (PaymentIdentity, PaymentState, PaymentValue);
    type Relations = (
        PaymentSource,
        PaymentDestination,
        PaymentBusiness,
        PaymentApproval,
    );
}
impl PaymentIntent {
    pub fn reference() -> ApplicationEntityRef<BankSchema, Self> {
        entity(PaymentRecords, Self)
    }
}
pub struct Approval;
impl fragment::Entity for Approval {
    const NAME: &'static str = "Approval";
    type Aspects = ();
    type Relations = (ApprovalPrincipal,);
}
impl Approval {
    pub fn reference() -> ApplicationEntityRef<BankSchema, Self> {
        entity(PaymentRecords, Self)
    }
}
pub struct JournalEntry;
impl fragment::Entity for JournalEntry {
    const NAME: &'static str = "JournalEntry";
    type Aspects = (JournalIdentity, JournalState);
    type Relations = (JournalPosting, JournalReversal);
}
impl JournalEntry {
    pub fn reference() -> ApplicationEntityRef<BankSchema, Self> {
        entity(PaymentRecords, Self)
    }
}
pub struct Posting;
impl fragment::Entity for Posting {
    const NAME: &'static str = "Posting";
    type Aspects = (PostingValue, PostingIdentity);
    type Relations = (PostingAccount,);
}
impl Posting {
    pub fn reference() -> ApplicationEntityRef<BankSchema, Self> {
        entity(PaymentRecords, Self)
    }
}
