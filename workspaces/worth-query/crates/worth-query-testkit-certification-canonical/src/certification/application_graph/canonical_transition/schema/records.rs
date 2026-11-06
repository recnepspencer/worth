//! One record model owns field lowering and reusable scalar bindings.
use super::*;
#[derive(Clone, Copy)]
pub struct StorageModel;
impl Fragment for StorageModel {
    const NAME: &'static str = "storage_model";
    type Bindings = StorageModelBindings;
    type Members = (ExternalMapping, Principal, Document, Derived);
}
pub struct StorageModelBindings;
impl BindingSet for StorageModelBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<WorthQueryExternalPrincipalIdentity> for StorageModelBindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for StorageModelBindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl Scalar<u64> for StorageModelBindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl Scalar<String> for StorageModelBindings {
    type Binding = StringApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
impl Scalar<NestedValue> for StorageModelBindings {
    type Binding = NestedBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl Scalar<output::DerivedV1> for StorageModelBindings {
    type Binding = output::DerivedBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
#[derive(Clone, Copy)]
pub struct ExternalMapping;
impl Entity for ExternalMapping {
    const NAME: &'static str = "ExternalMapping";
    type Aspects = (ExternalIdentity,);
    type Relations = (MappingTarget,);
}
impl ExternalMapping {
    pub fn reference() -> ApplicationEntityRef<CanonicalSchema, Self> {
        entity(StorageModel, Self)
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
    pub fn reference() -> ApplicationEntityRef<CanonicalSchema, Self> {
        entity(StorageModel, Self)
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
    pub fn reference() -> ApplicationEntityRef<CanonicalSchema, Self> {
        entity(StorageModel, Self)
    }
}
#[derive(Clone, Copy)]
pub struct Derived;
impl Entity for Derived {
    const NAME: &'static str = "Derived";
    type Aspects = (DerivedFacts,);
    type Relations = ();
}
impl Derived {
    pub fn reference() -> ApplicationEntityRef<CanonicalSchema, Self> {
        entity(StorageModel, Self)
    }
}
pub struct ExternalIdentity;
impl Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityValue;
}
impl ExternalIdentity {
    pub fn reference() -> AspectSelection<StorageModel, ExternalMapping, Self> {
        aspect(StorageModel, ExternalMapping, Self)
    }
}
pub struct ExternalIdentityValue {
    pub external_identity_field: WorthQueryExternalPrincipalIdentity,
    pub mapping_status_field: WorthQueryPrincipalMappingStatus,
}
impl Record<ExternalIdentityValue> for StorageModelBindings {
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
    pub fn reference() -> AspectSelection<StorageModel, Principal, Self> {
        aspect(StorageModel, Principal, Self)
    }
}
pub struct PrincipalFactsValue {
    pub principal_identity_field: u64,
}
impl Record<PrincipalFactsValue> for StorageModelBindings {
    fn record() -> impl Shape<PrincipalFactsValue, Self> {
        field("PrincipalIdentityField", |row: &PrincipalFactsValue| {
            row.principal_identity_field
        })
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
    pub fn reference() -> AspectSelection<StorageModel, Document, Self> {
        aspect(StorageModel, Document, Self)
    }
}
pub struct DocumentFactsValue {
    pub document_identity_field: String,
    pub document_value_field: NestedValue,
}
impl Record<DocumentFactsValue> for StorageModelBindings {
    fn record() -> impl Shape<DocumentFactsValue, Self> {
        field("DocumentIdentityField", |row: &DocumentFactsValue| {
            row.document_identity_field.clone()
        })
        .and("DocumentValueField", |row: &DocumentFactsValue| {
            row.document_value_field.clone()
        })
        .build(
            |document_identity_field, document_value_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(DocumentFactsValue {
                    document_identity_field,
                    document_value_field,
                })
            },
        )
    }
}
pub struct DerivedFacts;
impl Aspect for DerivedFacts {
    const NAME: &'static str = "DerivedFacts";
    type Row = DerivedFactsValue;
}
impl DerivedFacts {
    pub fn reference() -> AspectSelection<StorageModel, Derived, Self> {
        aspect(StorageModel, Derived, Self)
    }
}
pub struct DerivedFactsValue {
    pub derived_identity_field: String,
    pub derived_value_field: output::DerivedV1,
}
impl Record<DerivedFactsValue> for StorageModelBindings {
    fn record() -> impl Shape<DerivedFactsValue, Self> {
        field("DerivedIdentityField", |row: &DerivedFactsValue| {
            row.derived_identity_field.clone()
        })
        .and("DerivedValueField", |row: &DerivedFactsValue| {
            row.derived_value_field.clone()
        })
        .build(
            |derived_identity_field, derived_value_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(DerivedFactsValue {
                    derived_identity_field,
                    derived_value_field,
                })
            },
        )
    }
}
pub type ExternalIdentityField = FieldSelection<
    StorageModel,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryExternalPrincipalIdentity,
>;
pub type MappingStatusField = FieldSelection<
    StorageModel,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryPrincipalMappingStatus,
>;
pub type PrincipalIdentityField = FieldSelection<StorageModel, Principal, PrincipalFacts, u64>;
pub type DocumentIdentityField = FieldSelection<StorageModel, Document, DocumentFacts, String>;
pub type DocumentValueField = FieldSelection<StorageModel, Document, DocumentFacts, NestedValue>;
pub type DerivedIdentityField = FieldSelection<StorageModel, Derived, DerivedFacts, String>;
pub type DerivedValueField = FieldSelection<StorageModel, Derived, DerivedFacts, output::DerivedV1>;
pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
}
impl MappingTarget {
    pub fn reference() -> ApplicationRelationRef<CanonicalSchema, Self, ExternalMapping, Principal>
    {
        relation(StorageModel, ExternalMapping, Self)
    }
}
