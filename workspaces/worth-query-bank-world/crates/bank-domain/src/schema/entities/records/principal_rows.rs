//! Principal rows of the bank model.
use super::*;
pub struct ExternalPrincipalIdentity;
impl fragment::Aspect for ExternalPrincipalIdentity {
    const NAME: &'static str = "ExternalPrincipalIdentity";
    type Row = ExternalPrincipalIdentityValue;
}
impl ExternalPrincipalIdentity {
    pub fn reference() -> AspectSelection<AccountRecords, ExternalPrincipalMapping, Self> {
        aspect(AccountRecords, ExternalPrincipalMapping, Self)
    }
}
pub struct ExternalPrincipalIdentityValue {
    pub external_identity_key: WorthQueryExternalPrincipalIdentity,
    pub external_mapping_status: WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<ExternalPrincipalIdentityValue> for BankRecordsBindings {
    fn record() -> impl fragment::Shape<ExternalPrincipalIdentityValue, Self> {
        fragment::field(
            "ExternalIdentityKey",
            |row: &ExternalPrincipalIdentityValue| row.external_identity_key.clone(),
        )
        .equality_queryable(true)
        .and(
            "ExternalMappingStatus",
            |row: &ExternalPrincipalIdentityValue| row.external_mapping_status,
        )
        .equality_queryable(true)
        .build(
            |external_identity_key, external_mapping_status, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ExternalPrincipalIdentityValue {
                    external_identity_key,
                    external_mapping_status,
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
impl PrincipalIdentity {
    pub fn reference() -> AspectSelection<AccountRecords, Principal, Self> {
        aspect(AccountRecords, Principal, Self)
    }
}
pub struct PrincipalIdentityValue {
    pub principal_identity_field: BankPrincipalId,
}
impl fragment::Record<PrincipalIdentityValue> for BankRecordsBindings {
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
