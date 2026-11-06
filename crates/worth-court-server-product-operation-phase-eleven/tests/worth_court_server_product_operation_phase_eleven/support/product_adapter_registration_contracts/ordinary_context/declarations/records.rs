//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_host::facade::declaration::application_schema::{
    fragment::{self, *},
    *,
};
pub struct SessionRecords;
impl fragment::Fragment for SessionRecords {
    const NAME: &'static str = "session_records";
    type Bindings = SessionRecordsBindings;
    type Members = (SessionExternalMapping, SessionPrincipal, SessionBasisRecord);
}
pub struct SessionRecordsBindings;
impl fragment::BindingSet for SessionRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl
    fragment::Scalar<
        worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity,
    > for SessionRecordsBindings
{
    type Binding = worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentityBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl
    fragment::Scalar<
        worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus,
    > for SessionRecordsBindings
{
    type Binding = worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<u64> for SessionRecordsBindings {
    type Binding =
        worth_query_host::facade::declaration::application_schema::U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl fragment::Scalar<String> for SessionRecordsBindings {
    type Binding =
        worth_query_host::facade::declaration::application_schema::StringApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
pub struct SessionExternalMapping;
impl fragment::Entity for SessionExternalMapping {
    const NAME: &'static str = "SessionExternalMapping";
    type Aspects = (SessionExternalIdentity,);
    type Relations = (SessionMappingTarget,);
}
impl SessionExternalMapping {
    pub fn reference() -> ApplicationEntityRef<SessionBasisSchema, Self> {
        entity(SessionRecords, Self)
    }
}
pub struct SessionPrincipal;
impl fragment::Entity for SessionPrincipal {
    const NAME: &'static str = "SessionPrincipal";
    type Aspects = (SessionPrincipalFacts,);
    type Relations = ();
}
impl SessionPrincipal {
    pub fn reference() -> ApplicationEntityRef<SessionBasisSchema, Self> {
        entity(SessionRecords, Self)
    }
}
pub struct SessionBasisRecord;
impl fragment::Entity for SessionBasisRecord {
    const NAME: &'static str = "SessionBasisRecord";
    type Aspects = (SessionBasisFacts,);
    type Relations = ();
}
impl SessionBasisRecord {
    pub fn reference() -> ApplicationEntityRef<SessionBasisSchema, Self> {
        entity(SessionRecords, Self)
    }
}
pub struct SessionExternalIdentity;
impl fragment::Aspect for SessionExternalIdentity {
    const NAME: &'static str = "SessionExternalIdentity";
    type Row = SessionExternalIdentityValue;
}
impl SessionExternalIdentity {
    pub fn reference() -> AspectSelection<SessionRecords, SessionExternalMapping, Self> {
        aspect(SessionRecords, SessionExternalMapping, Self)
    }
}
pub struct SessionExternalIdentityValue {
    pub session_external_identity_field:
        worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity,
    pub session_mapping_status_field:
        worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<SessionExternalIdentityValue> for SessionRecordsBindings {
    fn record() -> impl fragment::Shape<SessionExternalIdentityValue, Self> {
        fragment::field(
            "SessionExternalIdentityField",
            |row: &SessionExternalIdentityValue| row.session_external_identity_field.clone(),
        )
        .and(
            "SessionMappingStatusField",
            |row: &SessionExternalIdentityValue| row.session_mapping_status_field,
        )
        .build(
            |session_external_identity_field,
             session_mapping_status_field,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(SessionExternalIdentityValue {
                    session_external_identity_field,
                    session_mapping_status_field,
                })
            },
        )
    }
}
pub struct SessionPrincipalFacts;
impl fragment::Aspect for SessionPrincipalFacts {
    const NAME: &'static str = "SessionPrincipalFacts";
    type Row = SessionPrincipalFactsValue;
}
impl SessionPrincipalFacts {
    pub fn reference() -> AspectSelection<SessionRecords, SessionPrincipal, Self> {
        aspect(SessionRecords, SessionPrincipal, Self)
    }
}
pub struct SessionPrincipalFactsValue {
    pub session_principal_identity_field: u64,
}
impl fragment::Record<SessionPrincipalFactsValue> for SessionRecordsBindings {
    fn record() -> impl fragment::Shape<SessionPrincipalFactsValue, Self> {
        fragment::field(
            "SessionPrincipalIdentityField",
            |row: &SessionPrincipalFactsValue| row.session_principal_identity_field,
        )
        .build(
            |session_principal_identity_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(SessionPrincipalFactsValue {
                    session_principal_identity_field,
                })
            },
        )
    }
}
pub struct SessionBasisFacts;
impl fragment::Aspect for SessionBasisFacts {
    const NAME: &'static str = "SessionBasisFacts";
    type Row = SessionBasisFactsValue;
}
impl SessionBasisFacts {
    pub fn reference() -> AspectSelection<SessionRecords, SessionBasisRecord, Self> {
        aspect(SessionRecords, SessionBasisRecord, Self)
    }
}
pub struct SessionBasisFactsValue {
    pub session_basis_value: String,
}
impl fragment::Record<SessionBasisFactsValue> for SessionRecordsBindings {
    fn record() -> impl fragment::Shape<SessionBasisFactsValue, Self> {
        fragment::field("SessionBasisValue", |row: &SessionBasisFactsValue| {
            row.session_basis_value.clone()
        })
        .build(|session_basis_value, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(SessionBasisFactsValue {
                session_basis_value,
            })
        })
    }
}
pub type SessionExternalIdentityField = FieldSelection<
    SessionRecords,
    SessionExternalMapping,
    SessionExternalIdentity,
    worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity,
>;
pub type SessionMappingStatusField = FieldSelection<
    SessionRecords,
    SessionExternalMapping,
    SessionExternalIdentity,
    worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus,
>;
pub type SessionPrincipalIdentityField =
    FieldSelection<SessionRecords, SessionPrincipal, SessionPrincipalFacts, u64>;
pub type SessionBasisValue =
    FieldSelection<SessionRecords, SessionBasisRecord, SessionBasisFacts, String>;
pub struct SessionMappingTarget;
impl fragment::Relation for SessionMappingTarget {
    const NAME: &'static str = "SessionMappingTarget";
    type To = SessionPrincipal;
}
impl SessionMappingTarget {
    pub fn reference(
    ) -> ApplicationRelationRef<SessionBasisSchema, Self, SessionExternalMapping, SessionPrincipal>
    {
        relation(SessionRecords, SessionExternalMapping, Self)
    }
}
