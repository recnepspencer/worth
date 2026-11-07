//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use crate::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct MutationModel;
impl fragment::Fragment for MutationModel {
    const NAME: &'static str = "mutation_model";
    type Bindings = MutationModelBindings;
    type Members = (ExternalMapping, Account);
}
pub struct MutationModelBindings;
impl fragment::BindingSet for MutationModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<WorthQueryExternalPrincipalIdentity> for MutationModelBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;

    const WRITABLE: bool = false;
}
impl fragment::Scalar<WorthQueryPrincipalMappingStatus> for MutationModelBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;

    const WRITABLE: bool = true;
}
impl fragment::Scalar<u64> for MutationModelBindings {
    type Binding = crate::application_schema::U64ApplicationValueBinding;

    const WRITABLE: bool = false;
}
pub struct ExternalMapping;
impl fragment::Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}
pub struct Account;
impl fragment::Entity for Account {
    const NAME: &'static str = "Account";
    type Aspects = (AccountIdentity,);
    type Relations = ();
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
impl fragment::Record<ExternalIdentityValue> for MutationModelBindings {
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
pub struct AccountIdentity;
impl fragment::Aspect for AccountIdentity {
    const NAME: &'static str = "AccountIdentity";
    type Row = AccountIdentityValue;
}
pub struct AccountIdentityValue {
    pub account_id_field: u64,
}
impl fragment::Record<AccountIdentityValue> for MutationModelBindings {
    fn record() -> impl fragment::Shape<AccountIdentityValue, Self> {
        fragment::field("AccountIdField", |row: &AccountIdentityValue| {
            row.account_id_field
        })
        .equality_queryable(true)
        .build(|account_id_field, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(AccountIdentityValue { account_id_field })
        })
    }
}
pub type ExternalIdentityField = FieldSelection<
    MutationModel,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryExternalPrincipalIdentity,
>;
pub type MappingStatusField = FieldSelection<
    MutationModel,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryPrincipalMappingStatus,
>;
pub type AccountIdField = FieldSelection<MutationModel, Account, AccountIdentity, u64>;
pub struct MappingTarget;
impl fragment::Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Account;
}

impl ExternalMapping {
    pub fn reference() -> ApplicationEntityRef<MutationSchema, Self> {
        entity(MutationModel, Self)
    }
}
impl Account {
    pub fn reference() -> ApplicationEntityRef<MutationSchema, Self> {
        entity(MutationModel, Self)
    }
}
impl MappingTarget {
    pub fn reference() -> ApplicationRelationRef<MutationSchema, Self, ExternalMapping, Account> {
        relation(MutationModel, ExternalMapping, Self)
    }
}
