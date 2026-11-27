//! A quality-management site: operators hold role assignments over documents
//! and the site; who may approve a document is relationship-based policy.
use serde::{Deserialize, Serialize};
use worth_query_decl::facade::{
    application_identity::ApplicationIdentityNamespace,
    application_schema::{fragment::*, *},
    worth_query_identity, worth_query_identity_namespace, worth_query_native_scalar_bindings,
    worth_query_structured_value_binding, WorthQuery, WorthQueryAction, WorthQueryRecord,
    WorthQueryRelation, WorthQueryRow, WorthQuerySchema,
};
use worth_query_host::facade::{
    primary_graph::WorthQueryOperationBodyDenial, worth_query_principal_registry,
};

pub use super::access::*;

pub const QMS: ApplicationIdentityNamespace = worth_query_identity_namespace!("rebac.qms.dx.v1");

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    schema,
    owner = "worth.rebac.qms.dx",
    name = "rebac_qms_dx",
    members(super::program::QmsContribution)
)]
pub struct QmsSchema;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    fragment,
    name = "rebac_qms_dx",
    bindings = Codecs,
    members(Operator, Site, Document, Assignment)
)]
pub struct Records;

worth_query_native_scalar_bindings!(pub Codecs);

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(OperatorFacts),
    relations(Holds),
    reference(QmsSchema, Records)
)]
pub struct Operator;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(SiteFacts),
    relations(Publishes),
    reference(QmsSchema, Records)
)]
pub struct Site;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(DocumentFacts), reference(QmsSchema, Records))]
pub struct Document;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(AssignmentFacts),
    relations(Covers, Administers),
    reference(QmsSchema, Records)
)]
pub struct Assignment;

/// An operator holds an assignment; either may be created later than the other.
#[derive(WorthQueryRelation)]
#[worth_query(
    to = Assignment, from = Operator, reference(QmsSchema, Records),
    cross_context = ApplicationRelationCrossContextPolicy::AllowExplicit,
)]
pub struct Holds;

/// A document-level assignment covers one document.
#[derive(WorthQueryRelation)]
#[worth_query(
    to = Document, from = Assignment, reference(QmsSchema, Records),
    cross_context = ApplicationRelationCrossContextPolicy::AllowExplicit,
)]
pub struct Covers;

/// A site-level assignment administers or reads one site.
#[derive(WorthQueryRelation)]
#[worth_query(
    to = Site, from = Assignment, reference(QmsSchema, Records),
    cross_context = ApplicationRelationCrossContextPolicy::AllowExplicit,
)]
pub struct Administers;

/// A site publishes its documents.
#[derive(WorthQueryRelation)]
#[worth_query(to = Document, from = Site, reference(QmsSchema, Records))]
pub struct Publishes;

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = OperatorFacts, fragment = Records, entity = Operator)]
pub struct OperatorRow {
    #[worth_query(equality, unique, read_only)]
    pub operator_id: u64,
}

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = SiteFacts, fragment = Records, entity = Site)]
pub struct SiteRow {
    #[worth_query(equality, unique, read_only)]
    pub site_id: String,
}

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = DocumentFacts, fragment = Records, entity = Document)]
pub struct DocumentRow {
    #[worth_query(equality, unique, read_only)]
    pub document_id: String,
    pub approved: bool,
}

/// `role` is `approver`, `author`, `admin` or `reader`; only `active` ones count.
#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = AssignmentFacts, fragment = Records, entity = Assignment)]
pub struct AssignmentRow {
    #[worth_query(equality, unique, read_only)]
    pub assignment_id: String,
    #[worth_query(equality)]
    pub role: String,
    #[worth_query(equality)]
    pub active: bool,
}

worth_query_structured_value_binding!(pub BodyDenial for WorthQueryOperationBodyDenial {
    identity: worth_query_identity!(QMS, denial)
});

worth_query_principal_registry! {
    pub mod accounts for QmsSchema {
        context: Qms {
            idempotency: u64 = worth_query_identity!(QMS, idempotency),
            denial: BodyDenial,
        },
        principal: Operator => OperatorId = OperatorFacts::operator_id(),
        directory: Site => SiteId = SiteFacts::site_id(),
        onboarding: ManageOperators,
    }
}
pub use accounts::Qms;

#[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
#[worth_query(schema = QmsSchema, root = Document, namespace = QMS, member)]
pub struct DocumentView {
    #[worth_query(field = DocumentId)]
    pub id: String,
    #[worth_query(field = Approved)]
    pub approved: bool,
}

/// A site's documents by key, two per page, for its active readers only.
#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = QmsSchema, context = Qms, namespace = QMS, member,
    result = DocumentView, root = Document, scope = SiteId,
    root_path(Publishes), order_by(DocumentView::id(), Ascending), limits = 2,
    ability = ReadDocuments::reference(),
)]
pub struct SiteDocuments {
    #[worth_query(scope)]
    pub site: String,
}

/// Approves one document; requires `Approve` on it.
#[derive(Clone, Debug, Serialize, Deserialize, WorthQueryAction)]
#[worth_query(
    schema = QmsSchema, context = Qms, namespace = QMS, member,
    input = ApproveDocumentInput, operation = ApproveDocumentOp,
    binding = ApproveDocumentBinding, delivery = DocumentApproved, body, program,
    reads(Document)
)]
pub struct ApproveDocument {
    #[worth_query(scope = DocumentId)]
    pub document: String,
    pub approved: bool,
}

pub fn approve_document_operation(
) -> ApplicationOperationDefinition<QmsSchema, ApproveDocumentOp, ApproveDocument> {
    ApproveDocumentOp::reference()
        .typed_body()
        .assign_input(DocumentFacts::approved(), ApproveDocument::approved_input())
        .finish_pure_definition()
        .expect("approve body admits")
}
