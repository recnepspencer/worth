//! The document-retention rule contracts this schema publishes.
//!
//! Two contracts with distinct stable identifiers are installed side by side:
//! `document-retention-v1` and `document-retention-v2`. They are separate
//! identifiers rather than two versions of one identifier because the installed
//! invariant catalog is keyed by (identifier, execution point), so one host
//! cannot install two versions of the same rule contract at the same point.
//! The version suffix is spelled with a dash because the declaration facade
//! rejects a `.` inside an invariant identifier.
//!
//! `document-retention-v3` is authored here but deliberately never installed, so
//! a program declaring it names a contract this host cannot support.

use worth_query_host::facade::declaration::application_schema::{
    ApplicationInvariantCostPosture, ApplicationInvariantDefinition,
    ApplicationInvariantEnforcement, ApplicationInvariantExecutionPoint, ApplicationInvariantGroup,
    ApplicationInvariantMarkerIdentity, ApplicationInvariantOperationalContract,
    ApplicationInvariantScopeTarget, RelationalInvariantWorkBudget,
};

use super::DocumentRetentionSchema;

/// The document's retention must not exceed ten days.
pub struct DocumentRetentionV1;

/// The document's retention must lie between five and twenty days inclusive.
pub struct DocumentRetentionV2;

/// A rule contract no host in this court installs.
pub struct DocumentRetentionV3;

impl ApplicationInvariantMarkerIdentity<DocumentRetentionSchema> for DocumentRetentionV1 {
    const IDENTIFIER: &'static str = "document-retention-v1";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

impl ApplicationInvariantMarkerIdentity<DocumentRetentionSchema> for DocumentRetentionV2 {
    const IDENTIFIER: &'static str = "document-retention-v2";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

impl ApplicationInvariantMarkerIdentity<DocumentRetentionSchema> for DocumentRetentionV3 {
    const IDENTIFIER: &'static str = "document-retention-v3";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

pub(super) fn first_definition(
) -> ApplicationInvariantDefinition<DocumentRetentionSchema, DocumentRetentionV1> {
    document_commit_boundary_definition(DocumentRetentionV1::reference())
}

pub(super) fn second_definition(
) -> ApplicationInvariantDefinition<DocumentRetentionSchema, DocumentRetentionV2> {
    document_commit_boundary_definition(DocumentRetentionV2::reference())
}

fn document_commit_boundary_definition<Invariant>(
    reference: worth_query_host::facade::declaration::application_schema::ApplicationInvariantRef<
        DocumentRetentionSchema,
        Invariant,
    >,
) -> ApplicationInvariantDefinition<DocumentRetentionSchema, Invariant>
where
    Invariant: ApplicationInvariantMarkerIdentity<DocumentRetentionSchema>,
{
    ApplicationInvariantDefinition::new(
        reference,
        ApplicationInvariantExecutionPoint::CommitBoundary,
        RelationalInvariantWorkBudget::new(None),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [ApplicationInvariantGroup::SchemaCompliance],
            [ApplicationInvariantScopeTarget::Entity(
                "Document".to_owned(),
            )],
            [ApplicationInvariantScopeTarget::Entity(
                "Document".to_owned(),
            )],
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
}
