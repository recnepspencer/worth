//! Payment relations of the bank model.
use super::*;
pub struct PaymentSource;
impl fragment::Relation for PaymentSource {
    const NAME: &'static str = "PaymentSource";
    type To = Account;
}
impl PaymentSource {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, PaymentIntent, Account> {
        relation(PaymentRecords, PaymentIntent, Self)
    }
}
pub struct PaymentDestination;
impl fragment::Relation for PaymentDestination {
    const NAME: &'static str = "PaymentDestination";
    type To = Account;
}
impl PaymentDestination {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, PaymentIntent, Account> {
        relation(PaymentRecords, PaymentIntent, Self)
    }
}
pub struct PaymentBusiness;
impl fragment::Relation for PaymentBusiness {
    const NAME: &'static str = "PaymentBusiness";
    type To = Business;
}
impl PaymentBusiness {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, PaymentIntent, Business> {
        relation(PaymentRecords, PaymentIntent, Self)
    }
}
pub struct PaymentInitiator;
impl fragment::Relation for PaymentInitiator {
    const NAME: &'static str = "PaymentInitiator";
    type To = PaymentIntent;
}
impl PaymentInitiator {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, Principal, PaymentIntent> {
        relation(AccountRecords, Principal, Self)
    }
}
pub struct PaymentApproval;
impl fragment::Relation for PaymentApproval {
    const NAME: &'static str = "PaymentApproval";
    type To = Approval;
}
impl PaymentApproval {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, PaymentIntent, Approval> {
        relation(PaymentRecords, PaymentIntent, Self)
    }
}
pub struct ApprovalPrincipal;
impl fragment::Relation for ApprovalPrincipal {
    const NAME: &'static str = "ApprovalPrincipal";
    type To = Principal;
}
impl ApprovalPrincipal {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, Approval, Principal> {
        relation(PaymentRecords, Approval, Self)
    }
}
pub struct JournalPosting;
impl fragment::Relation for JournalPosting {
    const NAME: &'static str = "JournalPosting";
    type To = Posting;
}
impl JournalPosting {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, JournalEntry, Posting> {
        relation(PaymentRecords, JournalEntry, Self)
    }
}
pub struct JournalReversal;
impl fragment::Relation for JournalReversal {
    const NAME: &'static str = "JournalReversal";
    type To = JournalEntry;
}
impl JournalReversal {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, JournalEntry, JournalEntry> {
        relation(PaymentRecords, JournalEntry, Self)
    }
}
pub struct PostingAccount;
impl fragment::Relation for PostingAccount {
    const NAME: &'static str = "PostingAccount";
    type To = Account;
}
impl PostingAccount {
    pub fn reference() -> ApplicationRelationRef<BankSchema, Self, Posting, Account> {
        relation(PaymentRecords, Posting, Self)
    }
}
