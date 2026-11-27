//! Who may act, as relationship paths from an operator through an active
//! assignment: approvers approve unless they also author (a deny path),
//! approvers and authors see the approver roster, site admins manage
//! operators and site readers page the site's documents.
use worth_query_decl::facade::application_schema::fragment::Registration;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{
    worth_query_ability, worth_query_operation_requires, worth_query_policy,
};

use super::roles::{AssignDocumentRoleOp, PublishDocumentOp, RevokeAssignmentOp};
use super::schema::*;

worth_query_ability!(pub Approve scoped_to Document, in QmsSchema);
worth_query_ability!(pub ViewApprovers scoped_to Document, in QmsSchema);
worth_query_ability!(pub ManageOperators scoped_to Site, in QmsSchema);
worth_query_ability!(pub ReadDocuments scoped_to Site, in QmsSchema);
worth_query_policy!(pub ApprovalPolicy in QmsSchema);
worth_query_policy!(pub RosterPolicy in QmsSchema);
worth_query_policy!(pub AdministrationPolicy in QmsSchema);
worth_query_policy!(pub ReadingPolicy in QmsSchema);

worth_query_operation_requires!(ApproveDocumentOp => [Approve]);
worth_query_operation_requires!(AssignDocumentRoleOp => [ManageOperators]);
worth_query_operation_requires!(RevokeAssignmentOp => [ManageOperators]);
worth_query_operation_requires!(PublishDocumentOp => [ManageOperators]);

/// Operator → Holds → an active `role` assignment.
fn holding(role: &str) -> ApplicationAuthorizationPathBuilder<QmsSchema, Assignment> {
    ApplicationAuthorizationPathBuilder::from_principal(Operator::reference())
        .forward(Holds::reference())
        .where_equal(
            AssignmentFacts::role(),
            ApplicationEncodedScalarValue::try_new(role.to_owned()).expect("role encodes"),
        )
        .expect("role is equality-queryable")
        .where_equal(
            AssignmentFacts::active(),
            ApplicationEncodedScalarValue::try_new(true).expect("active encodes"),
        )
        .expect("active is equality-queryable")
}

fn on_document(role: &str) -> ApplicationAuthorizationPathBuilder<QmsSchema, Document> {
    holding(role).forward(Covers::reference())
}

fn on_site(role: &str) -> ApplicationAuthorizationPathBuilder<QmsSchema, Site> {
    holding(role).forward(Administers::reference())
}

pub struct QmsAccess;
impl Registration<QmsSchema> for QmsAccess {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<QmsSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<QmsSchema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(builder
            .ability(Approve::reference())
            .ability(ViewApprovers::reference())
            .ability(ManageOperators::reference())
            .ability(ReadDocuments::reference())
            .policy(ApprovalPolicy::reference())
            .policy(RosterPolicy::reference())
            .policy(AdministrationPolicy::reference())
            .policy(ReadingPolicy::reference())
            .ability_policy(
                Approve::reference(),
                ApprovalPolicy::reference(),
                [
                    on_document("approver").allow(Document::reference()),
                    on_document("author").deny(Document::reference()),
                ],
            )
            .ability_policy(
                ViewApprovers::reference(),
                RosterPolicy::reference(),
                [
                    on_document("approver").allow(Document::reference()),
                    on_document("author").allow(Document::reference()),
                ],
            )
            .ability_policy(
                ManageOperators::reference(),
                AdministrationPolicy::reference(),
                [on_site("admin").allow(Site::reference())],
            )
            .ability_policy(
                ReadDocuments::reference(),
                ReadingPolicy::reference(),
                [on_site("reader").allow(Site::reference())],
            )
            .operation_requires_ability(ApproveDocumentOp::reference(), Approve::reference())
            .operation_requires_ability(
                AssignDocumentRoleOp::reference(),
                ManageOperators::reference(),
            )
            .operation_requires_ability(
                RevokeAssignmentOp::reference(),
                ManageOperators::reference(),
            )
            .operation_requires_ability(
                PublishDocumentOp::reference(),
                ManageOperators::reference(),
            ))
    }
}
