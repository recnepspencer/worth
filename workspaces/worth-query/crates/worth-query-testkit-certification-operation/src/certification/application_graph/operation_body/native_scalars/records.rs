//! An optional aspect whose fields are foundational native carriers bound
//! through the stock native scalar bindings.
use super::super::schema::storage_model::{BodyModel, BodyModelBindings, Document};
use worth_foundational::facade::{CanonicalDate, CanonicalTime, InlineBytes};
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};

pub struct DocumentScheduleFacts;
impl Aspect for DocumentScheduleFacts {
    const NAME: &'static str = "DocumentScheduleFacts";
    type Row = DocumentScheduleFactsValue;
}
impl DocumentScheduleFacts {
    pub fn reference() -> AspectSelection<BodyModel, Document, Self> {
        aspect(BodyModel, Document, Self)
    }
}
pub struct DocumentScheduleFactsValue {
    pub due: Option<CanonicalDate>,
    pub opens: Option<CanonicalTime>,
    pub token: Option<[u8; 16]>,
    pub payload: Option<InlineBytes>,
}
impl fragment::Record<DocumentScheduleFactsValue> for BodyModelBindings {
    fn record() -> impl Shape<DocumentScheduleFactsValue, Self> {
        field(
            "DocumentDueDateField",
            |row: &DocumentScheduleFactsValue| row.due,
        )
        .and(
            "DocumentOpensAtField",
            |row: &DocumentScheduleFactsValue| row.opens,
        )
        .and("DocumentTokenField", |row: &DocumentScheduleFactsValue| {
            row.token
        })
        .and(
            "DocumentPayloadField",
            |row: &DocumentScheduleFactsValue| row.payload.clone(),
        )
        .build(
            |due, opens, token, payload, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(DocumentScheduleFactsValue {
                    due,
                    opens,
                    token,
                    payload,
                })
            },
        )
    }
}

pub type DocumentDueDateField =
    FieldSelection<BodyModel, Document, DocumentScheduleFacts, CanonicalDate>;
pub type DocumentOpensAtField =
    FieldSelection<BodyModel, Document, DocumentScheduleFacts, CanonicalTime>;
pub type DocumentTokenField = FieldSelection<BodyModel, Document, DocumentScheduleFacts, [u8; 16]>;
pub type DocumentPayloadField =
    FieldSelection<BodyModel, Document, DocumentScheduleFacts, InlineBytes>;

impl fragment::Scalar<CanonicalDate> for BodyModelBindings {
    type Binding = CanonicalDateApplicationValueBinding;
}
impl fragment::Scalar<CanonicalTime> for BodyModelBindings {
    type Binding = CanonicalTimeApplicationValueBinding;
}
impl fragment::Scalar<[u8; 16]> for BodyModelBindings {
    type Binding = UuidApplicationValueBinding;
}
impl fragment::Scalar<InlineBytes> for BodyModelBindings {
    type Binding = InlineBytesApplicationValueBinding;
}
