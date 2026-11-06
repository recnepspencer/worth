//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct IdentityModel;
impl Fragment for IdentityModel {
    const NAME: &'static str = "identity_model";
    type Bindings = IdentityModelBindings;
    type Members = (ExternalMapping, Principal, Account);
}
pub struct IdentityModelBindings;
impl BindingSet for IdentityModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for IdentityModelBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl Scalar<u64> for IdentityModelBindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for IdentityModelBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl Scalar<String> for IdentityModelBindings {
    type Binding = StringApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
pub struct ExternalMapping;
impl Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}
impl ExternalMapping {
    pub fn reference() -> ApplicationEntityRef<HostIdentitySchema, Self> {
        entity(IdentityModel, Self)
    }
}
pub struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalIdentity,);
    type Relations = (
        inspection_capability::Grantor,
        inspection_capability::Grantee,
    );
}
impl Principal {
    pub fn reference() -> ApplicationEntityRef<HostIdentitySchema, Self> {
        entity(IdentityModel, Self)
    }
}
pub struct Account;
impl Entity for Account {
    const NAME: &'static str = "Account";
    type Aspects = (AccountIdentity,);
    type Relations = ();
}
impl Account {
    pub fn reference() -> ApplicationEntityRef<HostIdentitySchema, Self> {
        entity(IdentityModel, Self)
    }
}
pub struct ExternalIdentity;
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityValue;
}
impl ExternalIdentity {
    pub fn reference() -> AspectSelection<IdentityModel, ExternalMapping, Self> {
        aspect(IdentityModel, ExternalMapping, Self)
    }
}
pub struct ExternalIdentityValue {
    pub external_identity_field: WorthQueryExternalPrincipalIdentity,
    pub mapping_status_field: WorthQueryPrincipalMappingStatus,
}
impl Record<ExternalIdentityValue> for IdentityModelBindings {
    fn record() -> impl Shape<ExternalIdentityValue, Self> {
        field("ExternalIdentityField", |row: &ExternalIdentityValue| {
            row.external_identity_field.clone()
        })
        .and("MappingStatusField", |row: &ExternalIdentityValue| {
            row.mapping_status_field
        })
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
impl Aspect for PrincipalIdentity {
    const NAME: &'static str = "PrincipalIdentity";
    type Row = PrincipalIdentityValue;
}
impl PrincipalIdentity {
    pub fn reference() -> AspectSelection<IdentityModel, Principal, Self> {
        aspect(IdentityModel, Principal, Self)
    }
}
pub struct PrincipalIdentityValue {
    pub principal_identity_field: u64,
}
impl Record<PrincipalIdentityValue> for IdentityModelBindings {
    fn record() -> impl Shape<PrincipalIdentityValue, Self> {
        field("PrincipalIdentityField", |row: &PrincipalIdentityValue| {
            row.principal_identity_field
        })
        .build(
            |principal_identity_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalIdentityValue {
                    principal_identity_field,
                })
            },
        )
    }
}
pub struct AccountIdentity;
impl Aspect for AccountIdentity {
    const NAME: &'static str = "AccountIdentity";
    type Row = AccountIdentityValue;
}
impl AccountIdentity {
    pub fn reference() -> AspectSelection<IdentityModel, Account, Self> {
        aspect(IdentityModel, Account, Self)
    }
}
pub struct AccountIdentityValue {
    pub account_number: String,
}
impl Record<AccountIdentityValue> for IdentityModelBindings {
    fn record() -> impl Shape<AccountIdentityValue, Self> {
        field("AccountNumber", |row: &AccountIdentityValue| {
            row.account_number.clone()
        })
        .build(|account_number, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(AccountIdentityValue { account_number })
        })
    }
}
pub type ExternalIdentityField = FieldSelection<
    IdentityModel,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryExternalPrincipalIdentity,
>;
pub type PrincipalIdentityField = FieldSelection<IdentityModel, Principal, PrincipalIdentity, u64>;
pub type MappingStatusField = FieldSelection<
    IdentityModel,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryPrincipalMappingStatus,
>;
pub type AccountNumber = FieldSelection<IdentityModel, Account, AccountIdentity, String>;
pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference(
    ) -> ApplicationRelationRef<HostIdentitySchema, Self, ExternalMapping, Principal> {
        relation(IdentityModel, ExternalMapping, Self)
    }
}
