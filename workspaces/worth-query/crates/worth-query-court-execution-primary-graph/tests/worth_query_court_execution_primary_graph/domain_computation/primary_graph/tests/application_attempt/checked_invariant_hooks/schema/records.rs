//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct HookSchemaRecords;
impl fragment::Fragment for HookSchemaRecords {
    const NAME: &'static str = "hook_schema_records";
    type Bindings = HookSchemaRecordsBindings;
    type Members = (ExternalMapping, Principal, Account);
}
pub struct HookSchemaRecordsBindings;
impl fragment::BindingSet for HookSchemaRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<WorthQueryExternalPrincipalIdentity> for HookSchemaRecordsBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<u64> for HookSchemaRecordsBindings {
    type Binding = U64ApplicationValueBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<WorthQueryPrincipalMappingStatus> for HookSchemaRecordsBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<String> for HookSchemaRecordsBindings {
    type Binding = StringApplicationValueBinding;

    const WRITABLE: bool = true;
}
pub struct ExternalMapping;
impl fragment::Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}

pub struct Principal;
impl fragment::Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalIdentity,);
    type Relations = (AccountOwner,);
}
impl Principal {
    pub fn reference() -> ApplicationEntityRef<HookSchema, Self> {
        entity(HookSchemaRecords, Self)
    }
}
pub struct Account;
impl fragment::Entity for Account {
    const NAME: &'static str = "Account";
    type Aspects = (AccountPolicy,);
    type Relations = ();
}
impl Account {
    pub fn reference() -> ApplicationEntityRef<HookSchema, Self> {
        entity(HookSchemaRecords, Self)
    }
}
pub struct ExternalIdentity;
impl fragment::Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityValue;
}

pub struct ExternalIdentityValue {
    pub external_identity_field: WorthQueryExternalPrincipalIdentity,
    pub mapping_status_field: WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<ExternalIdentityValue> for HookSchemaRecordsBindings {
    fn record() -> impl fragment::Shape<ExternalIdentityValue, Self> {
        fragment::field("ExternalIdentityField", |row: &ExternalIdentityValue| {
            row.external_identity_field.clone()
        })
        .equality_queryable(true)
        .and("MappingStatusField", |row: &ExternalIdentityValue| {
            row.mapping_status_field
        })
        .equality_queryable(true)
        .build(
            |external_identity_field, mapping_status_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ExternalIdentityValue {
                    external_identity_field,
                    mapping_status_field,
                })
            },
        )
    }
}
pub struct PrincipalIdentity;
impl fragment::Aspect for PrincipalIdentity {
    const NAME: &'static str = "PrincipalIdentity";
    type Row = PrincipalIdentityValue;
}

pub struct PrincipalIdentityValue {
    pub principal_identity_field: u64,
}
impl fragment::Record<PrincipalIdentityValue> for HookSchemaRecordsBindings {
    fn record() -> impl fragment::Shape<PrincipalIdentityValue, Self> {
        fragment::field("PrincipalIdentityField", |row: &PrincipalIdentityValue| {
            row.principal_identity_field
        })
        .equality_queryable(true)
        .build(
            |principal_identity_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalIdentityValue {
                    principal_identity_field,
                })
            },
        )
    }
}
pub struct AccountPolicy;
impl fragment::Aspect for AccountPolicy {
    const NAME: &'static str = "AccountPolicy";
    type Row = AccountPolicyValue;
}

pub struct AccountPolicyValue {
    pub account_identity: String,
    pub account_status: String,
}
impl fragment::Record<AccountPolicyValue> for HookSchemaRecordsBindings {
    fn record() -> impl fragment::Shape<AccountPolicyValue, Self> {
        fragment::field("AccountIdentity", |row: &AccountPolicyValue| {
            row.account_identity.clone()
        })
        .equality_queryable(true)
        .read_only()
        .and("AccountStatus", |row: &AccountPolicyValue| {
            row.account_status.clone()
        })
        .equality_queryable(true)
        .build(
            |account_identity, account_status, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(AccountPolicyValue {
                    account_identity,
                    account_status,
                })
            },
        )
    }
}
pub type ExternalIdentityField = FieldSelection<
    HookSchemaRecords,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryExternalPrincipalIdentity,
>;
pub type PrincipalIdentityField =
    FieldSelection<HookSchemaRecords, Principal, PrincipalIdentity, u64>;
pub type MappingStatusField = FieldSelection<
    HookSchemaRecords,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryPrincipalMappingStatus,
>;
pub type AccountIdentity = FieldSelection<HookSchemaRecords, Account, AccountPolicy, String>;
pub type AccountStatus = FieldSelection<HookSchemaRecords, Account, AccountPolicy, String>;
pub struct MappingTarget;
impl fragment::Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference() -> ApplicationRelationRef<HookSchema, Self, ExternalMapping, Principal> {
        relation(HookSchemaRecords, ExternalMapping, Self)
    }
}
pub struct AccountOwner;
impl fragment::Relation for AccountOwner {
    const NAME: &'static str = "AccountOwner";
    type To = Account;
}
impl AccountOwner {
    pub fn reference() -> ApplicationRelationRef<HookSchema, Self, Principal, Account> {
        relation(HookSchemaRecords, Principal, Self)
    }
}
