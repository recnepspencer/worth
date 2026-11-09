//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_decl::facade::application_schema::{fragment::*, *};
pub struct RetentionModel;
impl Fragment for RetentionModel {
    const NAME: &'static str = "retention_model";
    type Bindings = RetentionModelBindings;
    type Members = (ExternalMapping, Principal, Document);
}
pub struct RetentionModelBindings;
impl BindingSet for RetentionModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<declaration::authentication::WorthQueryExternalPrincipalIdentity>
    for RetentionModelBindings
{
    type Binding = declaration::authentication::WorthQueryExternalPrincipalIdentityBinding;

    const WRITABLE: bool = false;
}
impl Scalar<declaration::authentication::WorthQueryPrincipalMappingStatus>
    for RetentionModelBindings
{
    type Binding = declaration::authentication::WorthQueryPrincipalMappingStatusBinding;

    const WRITABLE: bool = true;
}
impl Scalar<u64> for RetentionModelBindings {
    type Binding = declaration::application_schema::U64ApplicationValueBinding;

    const WRITABLE: bool = true;
}
impl Scalar<String> for RetentionModelBindings {
    type Binding = declaration::application_schema::StringApplicationValueBinding;

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
    pub fn reference() -> ApplicationEntityRef<SourceOnlyRetentionSchema, Self> {
        entity(RetentionModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Principal;
impl Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = ();
}
impl Principal {
    pub fn reference() -> ApplicationEntityRef<SourceOnlyRetentionSchema, Self> {
        entity(RetentionModel, Self)
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
    pub fn reference() -> ApplicationEntityRef<SourceOnlyRetentionSchema, Self> {
        entity(RetentionModel, Self)
    }
}
pub struct ExternalIdentity;
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityValue;
}
impl ExternalIdentity {
    pub fn reference() -> AspectSelection<RetentionModel, ExternalMapping, Self> {
        aspect(RetentionModel, ExternalMapping, Self)
    }
}
pub struct ExternalIdentityValue {
    pub external_identity_field: declaration::authentication::WorthQueryExternalPrincipalIdentity,
    pub mapping_status_field: declaration::authentication::WorthQueryPrincipalMappingStatus,
}
impl Record<ExternalIdentityValue> for RetentionModelBindings {
    fn record() -> impl Shape<ExternalIdentityValue, Self> {
        field("ExternalIdentityField", |row: &ExternalIdentityValue| {
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
pub struct PrincipalFacts;
impl Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalFactsValue;
}
impl PrincipalFacts {
    pub fn reference() -> AspectSelection<RetentionModel, Principal, Self> {
        aspect(RetentionModel, Principal, Self)
    }
}
pub struct PrincipalFactsValue {
    pub principal_identity_field: u64,
}
impl Record<PrincipalFactsValue> for RetentionModelBindings {
    fn record() -> impl Shape<PrincipalFactsValue, Self> {
        field("PrincipalIdentityField", |row: &PrincipalFactsValue| {
            row.principal_identity_field
        })
        .equality_queryable(true)
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
    pub fn reference() -> AspectSelection<RetentionModel, Document, Self> {
        aspect(RetentionModel, Document, Self)
    }
}
pub struct DocumentFactsValue {
    pub document_identity_field: String,
    pub document_retention_field: u64,
}
impl Record<DocumentFactsValue> for RetentionModelBindings {
    fn record() -> impl Shape<DocumentFactsValue, Self> {
        field("DocumentIdentityField", |row: &DocumentFactsValue| {
            row.document_identity_field.clone()
        })
        .equality_queryable(true)
        .and("DocumentRetentionField", |row: &DocumentFactsValue| {
            row.document_retention_field
        })
        .equality_queryable(true)
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
pub type ExternalIdentityField = FieldSelection<
    RetentionModel,
    ExternalMapping,
    ExternalIdentity,
    declaration::authentication::WorthQueryExternalPrincipalIdentity,
>;
pub type MappingStatusField = FieldSelection<
    RetentionModel,
    ExternalMapping,
    ExternalIdentity,
    declaration::authentication::WorthQueryPrincipalMappingStatus,
>;
pub type PrincipalIdentityField = FieldSelection<RetentionModel, Principal, PrincipalFacts, u64>;
pub type DocumentIdentityField = FieldSelection<RetentionModel, Document, DocumentFacts, String>;
pub type DocumentRetentionField = FieldSelection<RetentionModel, Document, DocumentFacts, u64>;
pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference(
    ) -> ApplicationRelationRef<SourceOnlyRetentionSchema, Self, ExternalMapping, Principal> {
        relation(RetentionModel, ExternalMapping, Self)
    }
}
