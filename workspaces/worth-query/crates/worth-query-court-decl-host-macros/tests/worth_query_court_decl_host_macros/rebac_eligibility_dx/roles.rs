//! Site administrators grant document roles and revoke any assignment: the
//! relationship edges every policy path reads.
use serde::{Deserialize, Serialize};
use worth_query_decl::facade::application_schema::UnitApplicationValueBinding;
use worth_query_decl::facade::WorthQueryAction;
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryApplicationEntityKey, WorthQueryInvariantMutationTarget,
    WorthQueryOperationBodyDenial,
};

use super::schema::*;

type Denial = WorthQueryOperationBodyDenial;

/// Grants `operator` an active `role` assignment on `document`.
#[derive(Clone, Debug, Serialize, Deserialize, WorthQueryAction)]
#[worth_query(
    schema = QmsSchema, context = Qms, namespace = QMS, member,
    input = AssignDocumentRoleInput, operation = AssignDocumentRoleOp,
    binding = AssignDocumentRoleBinding, delivery = DocumentRoleAssigned,
    result = UnitApplicationValueBinding, decision = AssignDecision, program,
    access(
        read_entities: [Site, Operator, Document];
        read_fields: [SiteFacts::site_id(), OperatorFacts::operator_id(),
            DocumentFacts::document_id()];
        writes: [AssignmentFacts::assignment_id(), AssignmentFacts::role(),
            AssignmentFacts::active()];
        creates: [Assignment];
        links: [Holds, Covers];
    )
)]
pub struct AssignDocumentRole {
    #[worth_query(scope = SiteId)]
    pub site: String,
    pub assignment: String,
    pub operator: u64,
    pub role: String,
    pub document: String,
}

pub struct AssignDecision {
    operator: WorthQueryInvariantMutationTarget<QmsSchema, Operator>,
    document: WorthQueryInvariantMutationTarget<QmsSchema, Document>,
}

pub struct AssignRoleHandler;
impl OperationHandler<QmsSchema, AssignDocumentRoleBinding> for AssignRoleHandler {
    fn decide(
        &self,
        input: &AssignDocumentRole,
        reader: &mut DecisionReader<'_, '_, '_, QmsSchema, AssignDocumentRoleBinding>,
    ) -> HandlerResult<AssignDecision, Denial> {
        (|| {
            let operator = reader.resolve_entity(OperatorFacts::operator_id(), input.operator)?;
            let document =
                reader.resolve_entity(DocumentFacts::document_id(), input.document.clone())?;
            Ok::<_, HandlerExecutionDenial>(AssignDecision {
                operator: reader.mutation_target(&operator)?,
                document: reader.mutation_target(&document)?,
            })
        })()
        .into()
    }

    fn build_candidate(
        &self,
        input: &AssignDocumentRole,
        decision: AssignDecision,
        writer: &mut CandidateWriter<'_, QmsSchema, AssignDocumentRoleBinding>,
    ) -> HandlerResult<(), Denial> {
        (|| {
            let operator = writer.projected_entity(&decision.operator)?;
            let document = writer.projected_entity(&decision.document)?;
            let key = WorthQueryApplicationEntityKey::new(input.assignment.clone())
                .map_err(HandlerExecutionDenial::new)?;
            let assignment = writer.create_entity(Assignment::reference(), key)?;
            let id = &input.assignment;
            writer.initialize_field(&assignment, AssignmentFacts::assignment_id(), id.clone())?;
            writer.initialize_field(&assignment, AssignmentFacts::role(), input.role.clone())?;
            writer.initialize_field(&assignment, AssignmentFacts::active(), true)?;
            writer.link(
                Holds::reference(),
                format!("holds-{id}"),
                &operator,
                &assignment,
            )?;
            writer.link(
                Covers::reference(),
                format!("covers-{id}"),
                &assignment,
                &document,
            )?;
            Ok::<_, HandlerExecutionDenial>(())
        })()
        .into()
    }
}

/// Deactivates one assignment immediately.
#[derive(Clone, Debug, Serialize, Deserialize, WorthQueryAction)]
#[worth_query(
    schema = QmsSchema, context = Qms, namespace = QMS, member,
    input = RevokeAssignmentInput, operation = RevokeAssignmentOp,
    binding = RevokeAssignmentBinding, delivery = AssignmentRevoked,
    result = UnitApplicationValueBinding,
    decision = WorthQueryInvariantMutationTarget<QmsSchema, Assignment>, program,
    access(
        read_entities: [Site, Assignment];
        read_fields: [SiteFacts::site_id(), AssignmentFacts::assignment_id(),
            AssignmentFacts::active()];
        writes: [AssignmentFacts::active()];
    )
)]
pub struct RevokeAssignment {
    #[worth_query(scope = SiteId)]
    pub site: String,
    pub assignment: String,
}

pub struct RevokeAssignmentHandler;
impl OperationHandler<QmsSchema, RevokeAssignmentBinding> for RevokeAssignmentHandler {
    fn decide(
        &self,
        input: &RevokeAssignment,
        reader: &mut DecisionReader<'_, '_, '_, QmsSchema, RevokeAssignmentBinding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<QmsSchema, Assignment>, Denial> {
        (|| {
            let assignment = reader
                .resolve_entity(AssignmentFacts::assignment_id(), input.assignment.clone())?;
            reader.field(&assignment, AssignmentFacts::active())?;
            reader.mutation_target(&assignment)
        })()
        .into()
    }

    fn build_candidate(
        &self,
        _: &RevokeAssignment,
        assignment: WorthQueryInvariantMutationTarget<QmsSchema, Assignment>,
        writer: &mut CandidateWriter<'_, QmsSchema, RevokeAssignmentBinding>,
    ) -> HandlerResult<(), Denial> {
        (|| {
            let assignment = writer.projected_entity(&assignment)?;
            writer.write_field(&assignment, AssignmentFacts::active(), false)?;
            Ok::<_, HandlerExecutionDenial>(())
        })()
        .into()
    }
}

/// Publishes a new document on `site`.
#[derive(Clone, Debug, Serialize, Deserialize, WorthQueryAction)]
#[worth_query(
    schema = QmsSchema, context = Qms, namespace = QMS, member,
    input = PublishDocumentInput, operation = PublishDocumentOp,
    binding = PublishDocumentBinding, delivery = DocumentPublished,
    result = UnitApplicationValueBinding,
    decision = WorthQueryInvariantMutationTarget<QmsSchema, Site>, program,
    access(
        read_entities: [Site];
        read_fields: [SiteFacts::site_id()];
        writes: [DocumentFacts::document_id(), DocumentFacts::approved()];
        creates: [Document];
        links: [Publishes];
    )
)]
pub struct PublishDocument {
    #[worth_query(scope = SiteId)]
    pub site: String,
    pub document: String,
}

pub struct PublishDocumentHandler;
impl OperationHandler<QmsSchema, PublishDocumentBinding> for PublishDocumentHandler {
    fn decide(
        &self,
        input: &PublishDocument,
        reader: &mut DecisionReader<'_, '_, '_, QmsSchema, PublishDocumentBinding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<QmsSchema, Site>, Denial> {
        (|| {
            let site = reader.resolve_entity(SiteFacts::site_id(), input.site.clone())?;
            reader.mutation_target(&site)
        })()
        .into()
    }

    fn build_candidate(
        &self,
        input: &PublishDocument,
        site: WorthQueryInvariantMutationTarget<QmsSchema, Site>,
        writer: &mut CandidateWriter<'_, QmsSchema, PublishDocumentBinding>,
    ) -> HandlerResult<(), Denial> {
        (|| {
            let site = writer.projected_entity(&site)?;
            let key = WorthQueryApplicationEntityKey::new(input.document.clone())
                .map_err(HandlerExecutionDenial::new)?;
            let document = writer.create_entity(Document::reference(), key)?;
            let id = input.document.clone();
            writer.initialize_field(&document, DocumentFacts::document_id(), id.clone())?;
            writer.initialize_field(&document, DocumentFacts::approved(), false)?;
            writer.link(
                Publishes::reference(),
                format!("publishes-{id}"),
                &site,
                &document,
            )?;
            Ok::<_, HandlerExecutionDenial>(())
        })()
        .into()
    }
}
