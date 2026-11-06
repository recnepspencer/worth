//! Estate relations of the bank model.
use super::*;
pub struct BranchInstitution;
impl fragment::Relation for BranchInstitution {
    const NAME: &'static str = "BranchInstitution";
    type To = Institution;
}
impl BranchInstitution {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, Branch, Institution> {
        relation(EstateRecords, Branch, Self)
    }
}
pub struct DeathNoticeSubject;
impl fragment::Relation for DeathNoticeSubject {
    const NAME: &'static str = "DeathNoticeSubject";
    type To = Principal;
}
impl DeathNoticeSubject {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, DeathNotice, Principal> {
        relation(EstateRecords, DeathNotice, Self)
    }
}
pub struct EstateDeathNotice;
impl fragment::Relation for EstateDeathNotice {
    const NAME: &'static str = "EstateDeathNotice";
    type To = DeathNotice;
}
impl EstateDeathNotice {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, EstateCase, DeathNotice> {
        relation(EstateRecords, EstateCase, Self)
    }
}
pub struct EstateDeceased;
impl fragment::Relation for EstateDeceased {
    const NAME: &'static str = "EstateDeceased";
    type To = Principal;
}
impl EstateDeceased {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, EstateCase, Principal> {
        relation(EstateRecords, EstateCase, Self)
    }
}
pub struct EstateAccount;
impl fragment::Relation for EstateAccount {
    const NAME: &'static str = "EstateAccount";
    type To = Account;
}
impl EstateAccount {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, EstateCase, Account> {
        relation(EstateRecords, EstateCase, Self)
    }
}
pub struct EstateBranch;
impl fragment::Relation for EstateBranch {
    const NAME: &'static str = "EstateBranch";
    type To = Branch;
}
impl EstateBranch {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, EstateCase, Branch> {
        relation(EstateRecords, EstateCase, Self)
    }
}
pub struct EstateExecutor;
impl fragment::Relation for EstateExecutor {
    const NAME: &'static str = "EstateExecutor";
    type To = EstateCase;
}
impl EstateExecutor {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, Principal, EstateCase> {
        relation(AccountRecords, Principal, Self)
    }
}
pub struct EstateBeneficiary;
impl fragment::Relation for EstateBeneficiary {
    const NAME: &'static str = "EstateBeneficiary";
    type To = EstateCase;
}
impl EstateBeneficiary {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, Principal, EstateCase> {
        relation(AccountRecords, Principal, Self)
    }
}
pub struct EstateJointOwner;
impl fragment::Relation for EstateJointOwner {
    const NAME: &'static str = "EstateJointOwner";
    type To = Account;
}
impl EstateJointOwner {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, Principal, Account> {
        relation(AccountRecords, Principal, Self)
    }
}
pub struct EstateAuthorizedSigner;
impl fragment::Relation for EstateAuthorizedSigner {
    const NAME: &'static str = "EstateAuthorizedSigner";
    type To = Account;
}
impl EstateAuthorizedSigner {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, Principal, Account> {
        relation(AccountRecords, Principal, Self)
    }
}
pub struct EstateAssignment;
impl fragment::Relation for EstateAssignment {
    const NAME: &'static str = "EstateAssignment";
    type To = EstateCase;
}
impl EstateAssignment {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, EmployeeAssignment, EstateCase> {
        relation(AccountRecords, EmployeeAssignment, Self)
    }
}
pub struct LegalAuthorityEstate;
impl fragment::Relation for LegalAuthorityEstate {
    const NAME: &'static str = "LegalAuthorityEstate";
    type To = EstateCase;
}
impl LegalAuthorityEstate {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, LegalAuthority, EstateCase> {
        relation(EstateRecords, LegalAuthority, Self)
    }
}
pub struct LegalAuthorityHolder;
impl fragment::Relation for LegalAuthorityHolder {
    const NAME: &'static str = "LegalAuthorityHolder";
    type To = Principal;
}
impl LegalAuthorityHolder {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, LegalAuthority, Principal> {
        relation(EstateRecords, LegalAuthority, Self)
    }
}
pub struct CapabilityGrantee;
impl fragment::Relation for CapabilityGrantee {
    const NAME: &'static str = "CapabilityGrantee";
    type To = CapabilityGrant;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        ApplicationRelationCrossContextPolicy::AllowExplicit;
}
impl CapabilityGrantee {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, Principal, CapabilityGrant> {
        relation(AccountRecords, Principal, Self)
    }
}
pub struct CapabilityGrantor;
impl fragment::Relation for CapabilityGrantor {
    const NAME: &'static str = "CapabilityGrantor";
    type To = CapabilityGrant;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        ApplicationRelationCrossContextPolicy::AllowExplicit;
}
impl CapabilityGrantor {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, Principal, CapabilityGrant> {
        relation(AccountRecords, Principal, Self)
    }
}
pub struct CapabilityEstate;
impl fragment::Relation for CapabilityEstate {
    const NAME: &'static str = "CapabilityEstate";
    type To = EstateCase;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        ApplicationRelationCrossContextPolicy::AllowExplicit;
}
impl CapabilityEstate {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, CapabilityGrant, EstateCase> {
        relation(EstateRecords, CapabilityGrant, Self)
    }
}
pub struct CapabilityAccount;
impl fragment::Relation for CapabilityAccount {
    const NAME: &'static str = "CapabilityAccount";
    type To = Account;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        ApplicationRelationCrossContextPolicy::AllowExplicit;
}
impl CapabilityAccount {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, CapabilityGrant, Account> {
        relation(EstateRecords, CapabilityGrant, Self)
    }
}
pub struct CapabilityInstitution;
impl fragment::Relation for CapabilityInstitution {
    const NAME: &'static str = "CapabilityInstitution";
    type To = Institution;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        ApplicationRelationCrossContextPolicy::AllowExplicit;
}
impl CapabilityInstitution {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, CapabilityGrant, Institution> {
        relation(EstateRecords, CapabilityGrant, Self)
    }
}
pub struct CapabilityBranch;
impl fragment::Relation for CapabilityBranch {
    const NAME: &'static str = "CapabilityBranch";
    type To = Branch;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        ApplicationRelationCrossContextPolicy::AllowExplicit;
}
impl CapabilityBranch {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, CapabilityGrant, Branch> {
        relation(EstateRecords, CapabilityGrant, Self)
    }
}
pub struct CapabilityParent;
impl fragment::Relation for CapabilityParent {
    const NAME: &'static str = "CapabilityParent";
    type To = CapabilityGrant;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        ApplicationRelationCrossContextPolicy::AllowExplicit;
}
impl CapabilityParent {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, CapabilityGrant, CapabilityGrant>
    {
        relation(EstateRecords, CapabilityGrant, Self)
    }
}
pub struct EmergencyRequester;
impl fragment::Relation for EmergencyRequester {
    const NAME: &'static str = "EmergencyRequester";
    type To = EmergencyAccess;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        ApplicationRelationCrossContextPolicy::AllowExplicit;
}
impl EmergencyRequester {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, Principal, EmergencyAccess> {
        relation(AccountRecords, Principal, Self)
    }
}
pub struct EmergencyApprover;
impl fragment::Relation for EmergencyApprover {
    const NAME: &'static str = "EmergencyApprover";
    type To = EmergencyAccess;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        ApplicationRelationCrossContextPolicy::AllowExplicit;
}
impl EmergencyApprover {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, Principal, EmergencyAccess> {
        relation(AccountRecords, Principal, Self)
    }
}
pub struct EmergencyGrant;
impl fragment::Relation for EmergencyGrant {
    const NAME: &'static str = "EmergencyGrant";
    type To = CapabilityGrant;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        ApplicationRelationCrossContextPolicy::AllowExplicit;
}
impl EmergencyGrant {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, EmergencyAccess, CapabilityGrant>
    {
        relation(EstateRecords, EmergencyAccess, Self)
    }
}
pub struct EmergencyEstate;
impl fragment::Relation for EmergencyEstate {
    const NAME: &'static str = "EmergencyEstate";
    type To = EstateCase;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        ApplicationRelationCrossContextPolicy::AllowExplicit;
}
impl EmergencyEstate {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, EmergencyAccess, EstateCase> {
        relation(EstateRecords, EmergencyAccess, Self)
    }
}
pub struct EmergencyReview;
impl fragment::Relation for EmergencyReview {
    const NAME: &'static str = "EmergencyReview";
    type To = MandatoryReview;
}
impl EmergencyReview {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, EmergencyAccess, MandatoryReview>
    {
        relation(EstateRecords, EmergencyAccess, Self)
    }
}
pub struct ReviewPrincipal;
impl fragment::Relation for ReviewPrincipal {
    const NAME: &'static str = "ReviewPrincipal";
    type To = MandatoryReview;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        ApplicationRelationCrossContextPolicy::AllowExplicit;
}
impl ReviewPrincipal {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, Principal, MandatoryReview> {
        relation(AccountRecords, Principal, Self)
    }
}
pub struct ReviewEstate;
impl fragment::Relation for ReviewEstate {
    const NAME: &'static str = "ReviewEstate";
    type To = EstateCase;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        ApplicationRelationCrossContextPolicy::AllowExplicit;
}
impl ReviewEstate {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, MandatoryReview, EstateCase> {
        relation(EstateRecords, MandatoryReview, Self)
    }
}
