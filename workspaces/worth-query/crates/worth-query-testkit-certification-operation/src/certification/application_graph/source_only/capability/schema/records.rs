//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct CapabilityModel;
impl Fragment for CapabilityModel {
    const NAME: &'static str = "capability_model";
    type Bindings = CapabilityModelBindings;
    type Members = (ExternalMapping, Principal, Document, Grant);
}
pub struct CapabilityModelBindings;
impl BindingSet for CapabilityModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<declaration::authentication::WorthQueryExternalPrincipalIdentity>
    for CapabilityModelBindings
{
    type Binding = declaration::authentication::WorthQueryExternalPrincipalIdentityBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl Scalar<declaration::authentication::WorthQueryPrincipalMappingStatus>
    for CapabilityModelBindings
{
    type Binding = declaration::authentication::WorthQueryPrincipalMappingStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl Scalar<u64> for CapabilityModelBindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl Scalar<String> for CapabilityModelBindings {
    type Binding = StringApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
#[derive(Clone, Copy)]
pub struct ExternalMapping;
impl Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}
impl ExternalMapping {
    pub fn reference() -> ApplicationEntityRef<CapabilitySchema, Self> {
        entity(CapabilityModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = (GrantGrantee, GrantGrantor);
}
impl Principal {
    pub fn reference() -> ApplicationEntityRef<CapabilitySchema, Self> {
        entity(CapabilityModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Document;
impl Entity for Document {
    const NAME: &'static str = "Document";
    type Aspects = (DocumentFacts,);
    type Relations = ();
}
impl Document {
    pub fn reference() -> ApplicationEntityRef<CapabilitySchema, Self> {
        entity(CapabilityModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Grant;
impl Entity for Grant {
    const NAME: &'static str = "Grant";
    type Aspects = (GrantFacts,);
    type Relations = (GrantResource, GrantParent);
}
impl Grant {
    pub fn reference() -> ApplicationEntityRef<CapabilitySchema, Self> {
        entity(CapabilityModel, Self)
    }
}
pub struct ExternalIdentity;
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityValue;
}
impl ExternalIdentity {
    pub fn reference() -> AspectSelection<CapabilityModel, ExternalMapping, Self> {
        aspect(CapabilityModel, ExternalMapping, Self)
    }
}
pub struct ExternalIdentityValue {
    pub external_identity_field: declaration::authentication::WorthQueryExternalPrincipalIdentity,
    pub mapping_status_field: declaration::authentication::WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<ExternalIdentityValue> for CapabilityModelBindings {
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
pub struct PrincipalFacts;
impl Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalFactsValue;
}
impl PrincipalFacts {
    pub fn reference() -> AspectSelection<CapabilityModel, Principal, Self> {
        aspect(CapabilityModel, Principal, Self)
    }
}
pub struct PrincipalFactsValue {
    pub principal_identity_field: u64,
}
impl fragment::Record<PrincipalFactsValue> for CapabilityModelBindings {
    fn record() -> impl Shape<PrincipalFactsValue, Self> {
        field("PrincipalIdentityField", |row: &PrincipalFactsValue| {
            row.principal_identity_field
        })
        .read_only()
        .build(
            |principal_identity_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalFactsValue {
                    principal_identity_field,
                })
            },
        )
    }
}
pub struct DocumentFacts;
impl Aspect for DocumentFacts {
    const NAME: &'static str = "DocumentFacts";
    type Row = DocumentFactsValue;
}
impl DocumentFacts {
    pub fn reference() -> AspectSelection<CapabilityModel, Document, Self> {
        aspect(CapabilityModel, Document, Self)
    }
}
pub struct DocumentFactsValue {
    pub document_identity_field: String,
    pub document_retention_field: u64,
}
impl fragment::Record<DocumentFactsValue> for CapabilityModelBindings {
    fn record() -> impl Shape<DocumentFactsValue, Self> {
        field("DocumentIdentityField", |row: &DocumentFactsValue| {
            row.document_identity_field.clone()
        })
        .and("DocumentRetentionField", |row: &DocumentFactsValue| {
            row.document_retention_field
        })
        .build(
            |document_identity_field,
             document_retention_field,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(DocumentFactsValue {
                    document_identity_field,
                    document_retention_field,
                })
            },
        )
    }
}
pub struct GrantFacts;
impl Aspect for GrantFacts {
    const NAME: &'static str = "GrantFacts";
    type Row = GrantFactsValue;
}
impl GrantFacts {
    pub fn reference() -> AspectSelection<CapabilityModel, Grant, Self> {
        aspect(CapabilityModel, Grant, Self)
    }
}
pub struct GrantFactsValue {
    pub resource_disclosure_field: u64,
    pub grant_action: u64,
    pub grant_purpose: u64,
    pub grant_status: u64,
    pub grant_not_before: u64,
    pub grant_not_after: u64,
    pub grant_depth: u64,
    pub grant_identity: String,
    pub grant_scope_key: String,
}
impl fragment::Record<GrantFactsValue> for CapabilityModelBindings {
    fn record() -> impl Shape<GrantFactsValue, Self> {
        field("ResourceDisclosureField", |row: &GrantFactsValue| {
            row.resource_disclosure_field
        })
        .read_only()
        .and("GrantAction", |row: &GrantFactsValue| row.grant_action)
        .read_only()
        .and("GrantPurpose", |row: &GrantFactsValue| row.grant_purpose)
        .read_only()
        .and("GrantStatus", |row: &GrantFactsValue| row.grant_status)
        .and("GrantNotBefore", |row: &GrantFactsValue| {
            row.grant_not_before
        })
        .read_only()
        .and("GrantNotAfter", |row: &GrantFactsValue| row.grant_not_after)
        .read_only()
        .and("GrantDepth", |row: &GrantFactsValue| row.grant_depth)
        .read_only()
        .and("GrantIdentity", |row: &GrantFactsValue| {
            row.grant_identity.clone()
        })
        .and("GrantScopeKey", |row: &GrantFactsValue| {
            row.grant_scope_key.clone()
        })
        .build(
            |resource_disclosure_field,
             grant_action,
             grant_purpose,
             grant_status,
             grant_not_before,
             grant_not_after,
             grant_depth,
             grant_identity,
             grant_scope_key,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(GrantFactsValue {
                    resource_disclosure_field,
                    grant_action,
                    grant_purpose,
                    grant_status,
                    grant_not_before,
                    grant_not_after,
                    grant_depth,
                    grant_identity,
                    grant_scope_key,
                })
            },
        )
    }
}
pub type ExternalIdentityField = FieldSelection<
    CapabilityModel,
    ExternalMapping,
    ExternalIdentity,
    declaration::authentication::WorthQueryExternalPrincipalIdentity,
>;
pub type MappingStatusField = FieldSelection<
    CapabilityModel,
    ExternalMapping,
    ExternalIdentity,
    declaration::authentication::WorthQueryPrincipalMappingStatus,
>;
pub type PrincipalIdentityField = FieldSelection<CapabilityModel, Principal, PrincipalFacts, u64>;
pub type DocumentIdentityField = FieldSelection<CapabilityModel, Document, DocumentFacts, String>;
pub type DocumentRetentionField = FieldSelection<CapabilityModel, Document, DocumentFacts, u64>;
pub type ResourceDisclosureField = FieldSelection<CapabilityModel, Grant, GrantFacts, u64>;
pub type GrantAction = FieldSelection<CapabilityModel, Grant, GrantFacts, u64>;
pub type GrantPurpose = FieldSelection<CapabilityModel, Grant, GrantFacts, u64>;
pub type GrantStatus = FieldSelection<CapabilityModel, Grant, GrantFacts, u64>;
pub type GrantNotBefore = FieldSelection<CapabilityModel, Grant, GrantFacts, u64>;
pub type GrantNotAfter = FieldSelection<CapabilityModel, Grant, GrantFacts, u64>;
pub type GrantDepth = FieldSelection<CapabilityModel, Grant, GrantFacts, u64>;
pub type GrantIdentity = FieldSelection<CapabilityModel, Grant, GrantFacts, String>;
pub type GrantScopeKey = FieldSelection<CapabilityModel, Grant, GrantFacts, String>;
pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference() -> ApplicationRelationRef<CapabilitySchema, Self, ExternalMapping, Principal>
    {
        relation(CapabilityModel, ExternalMapping, Self)
    }
}
pub struct GrantResource;
impl Relation for GrantResource {
    const NAME: &'static str = "GrantResource";
    type To = Document;
}
impl GrantResource {
    pub fn reference() -> ApplicationRelationRef<CapabilitySchema, Self, Grant, Document> {
        relation(CapabilityModel, Grant, Self)
    }
}
pub struct GrantGrantee;
impl Relation for GrantGrantee {
    const NAME: &'static str = "GrantGrantee";
    type To = Grant;
}
impl GrantGrantee {
    pub fn reference() -> ApplicationRelationRef<CapabilitySchema, Self, Principal, Grant> {
        relation(CapabilityModel, Principal, Self)
    }
}
pub struct GrantGrantor;
impl Relation for GrantGrantor {
    const NAME: &'static str = "GrantGrantor";
    type To = Grant;
}
impl GrantGrantor {
    pub fn reference() -> ApplicationRelationRef<CapabilitySchema, Self, Principal, Grant> {
        relation(CapabilityModel, Principal, Self)
    }
}
pub struct GrantParent;
impl Relation for GrantParent {
    const NAME: &'static str = "GrantParent";
    type To = Grant;
}
impl GrantParent {
    pub fn reference() -> ApplicationRelationRef<CapabilitySchema, Self, Grant, Grant> {
        relation(CapabilityModel, Grant, Self)
    }
}
