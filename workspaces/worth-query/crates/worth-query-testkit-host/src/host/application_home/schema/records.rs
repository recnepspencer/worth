//! The shared Home account and principal record contracts for A and B.
use worth_query_decl::facade::{
    application_schema::{fragment::*, *},
    authentication::{
        WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
        WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
    },
};
pub struct HomeRecords;
impl Fragment for HomeRecords {
    const NAME: &'static str = "home_records";
    type Bindings = HomeBindings;
    type Members = (ExternalMapping, Principal, Account);
}
pub struct HomeBindings;
impl BindingSet for HomeBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for HomeBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for HomeBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}
impl Scalar<String> for HomeBindings {
    type Binding = StringApplicationValueBinding;
    const WRITABLE: bool = false;
}
impl Scalar<u64> for HomeBindings {
    type Binding = U64ApplicationValueBinding;
}
pub struct ExternalMapping;
impl Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}
pub struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalIdentity,);
    type Relations = ();
}
pub struct Account;
impl Entity for Account {
    const NAME: &'static str = "Account";
    type Aspects = (AccountIdentity,);
    type Relations = (UnusedAccountPeer,);
}
pub struct ExternalIdentity;
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityRow;
}
pub struct ExternalIdentityRow {
    pub identity: WorthQueryExternalPrincipalIdentity,
    pub status: WorthQueryPrincipalMappingStatus,
}
impl Record<ExternalIdentityRow> for HomeBindings {
    fn record() -> impl Shape<ExternalIdentityRow, Self> {
        field("ExternalIdentityField", |row: &ExternalIdentityRow| {
            row.identity.clone()
        })
        .equality_queryable(true)
        .and("MappingStatusField", |row: &ExternalIdentityRow| row.status)
        .equality_queryable(true)
        .build(|identity, status, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(ExternalIdentityRow { identity, status })
        })
    }
}
pub struct PrincipalIdentity;
impl Aspect for PrincipalIdentity {
    const NAME: &'static str = "PrincipalIdentity";
    type Row = PrincipalIdentityRow;
}
pub struct PrincipalIdentityRow {
    pub identity: u64,
}
impl Record<PrincipalIdentityRow> for HomeBindings {
    fn record() -> impl Shape<PrincipalIdentityRow, Self> {
        field("PrincipalIdentityField", |row: &PrincipalIdentityRow| {
            row.identity
        })
        .read_only()
        .equality_queryable(true)
        .build(|identity, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(PrincipalIdentityRow { identity })
        })
    }
}
pub struct AccountIdentity;
impl Aspect for AccountIdentity {
    const NAME: &'static str = "AccountIdentity";
    type Row = AccountIdentityRow;
}
pub struct AccountIdentityRow {
    pub number: String,
    pub balance: u64,
}
impl Record<AccountIdentityRow> for HomeBindings {
    fn record() -> impl Shape<AccountIdentityRow, Self> {
        field("AccountNumber", |row: &AccountIdentityRow| {
            row.number.clone()
        })
        .equality_queryable(true)
        .and("Balance", |row: &AccountIdentityRow| row.balance)
        .equality_queryable(true)
        .build(|number, balance, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(AccountIdentityRow { number, balance })
        })
    }
}
pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
// Deliberately never seeded: its physical meaning must survive unused.
pub struct UnusedAccountPeer;
impl Relation for UnusedAccountPeer {
    const NAME: &'static str = "UnusedAccountPeer";
    type To = Account;
    const SELF_EDGES: bool = false;
}
