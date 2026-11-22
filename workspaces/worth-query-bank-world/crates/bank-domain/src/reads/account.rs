use crate::model::{
    AccountAuthorizationId, AccountId, AccountJournalRevision, AccountName, BankPrincipalId,
    BusinessId, CustomerRole, InstitutionId, SignedMoney, USD,
};
use crate::schema::{AccountKind, AccountStatus};

/// One account a principal may see: the row of every account-discovery query.
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    worth_query_decl::facade::WorthQueryProjection,
)]
#[worth_query(schema = crate::schema::BankSchema, root = crate::schema::Account)]
pub struct VisibleAccount {
    #[worth_query(field = crate::schema::AccountIdentity, slot = "AccountIdentitySlot")]
    account: AccountId,
}

impl VisibleAccount {
    pub const fn new(account: AccountId) -> Self {
        Self { account }
    }

    pub const fn id(self) -> AccountId {
        self.account
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountSummary {
    id: AccountId,
    display_name: AccountName,
    kind: AccountKind,
    status: AccountStatus,
    accounting_revision: AccountJournalRevision,
    current_balance: SignedMoney<USD>,
    available_balance: SignedMoney<USD>,
}

impl AccountSummary {
    pub const fn from_projection(
        id: AccountId,
        display_name: AccountName,
        kind: AccountKind,
        status: AccountStatus,
        accounting_revision: AccountJournalRevision,
        current_balance: SignedMoney<USD>,
        available_balance: SignedMoney<USD>,
    ) -> Self {
        Self {
            id,
            display_name,
            kind,
            status,
            accounting_revision,
            current_balance,
            available_balance,
        }
    }

    pub const fn id(&self) -> AccountId {
        self.id
    }

    pub const fn display_name(&self) -> &AccountName {
        &self.display_name
    }

    pub const fn kind(&self) -> AccountKind {
        self.kind
    }

    pub const fn status(&self) -> AccountStatus {
        self.status
    }

    pub const fn accounting_revision(&self) -> AccountJournalRevision {
        self.accounting_revision
    }

    pub const fn current_balance(&self) -> SignedMoney<USD> {
        self.current_balance
    }

    pub const fn available_balance(&self) -> SignedMoney<USD> {
        self.available_balance
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountDetail {
    summary: AccountSummary,
    institution: InstitutionId,
    personal_owner: Option<BankPrincipalId>,
    business_owner: Option<BusinessId>,
}

impl AccountDetail {
    pub const fn from_projection(
        summary: AccountSummary,
        institution: InstitutionId,
        personal_owner: Option<BankPrincipalId>,
        business_owner: Option<BusinessId>,
    ) -> Self {
        Self {
            summary,
            institution,
            personal_owner,
            business_owner,
        }
    }

    pub const fn summary(&self) -> &AccountSummary {
        &self.summary
    }

    pub const fn institution(&self) -> InstitutionId {
        self.institution
    }

    pub const fn personal_owner(&self) -> Option<BankPrincipalId> {
        self.personal_owner
    }

    pub const fn business_owner(&self) -> Option<BusinessId> {
        self.business_owner
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthorizedAccountUser {
    authorization: AccountAuthorizationId,
    principal: BankPrincipalId,
    role: CustomerRole,
}

impl AuthorizedAccountUser {
    pub const fn from_projection(
        authorization: AccountAuthorizationId,
        principal: BankPrincipalId,
        role: CustomerRole,
    ) -> Self {
        Self {
            authorization,
            principal,
            role,
        }
    }

    pub const fn authorization(self) -> AccountAuthorizationId {
        self.authorization
    }

    pub const fn principal(self) -> BankPrincipalId {
        self.principal
    }

    pub const fn role(self) -> CustomerRole {
        self.role
    }
}
