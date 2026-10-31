//! The existing ordinary document read and edit form one declared model history.
use super::{
    retention_entry::{DocumentRetentionRead, SetRetentionBinding},
    schema::DocumentRetentionSchema,
};
use worth_query_decl::facade::{
    application_history::{
        ApplicationHistoryBinding, ApplicationHistoryDefinition, ApplicationHistoryModelDomain,
    },
    application_query::{
        ApplicationHistoryMetadataFacet, ApplicationHistoryMetadataRef,
        ApplicationQueryDisclosureContract, ApplicationQueryInfluenceContract,
    },
    application_schema::ApplicationSchemaDeclarationBuilder,
    worth_query_portable_type,
};

#[path = "document_history/matching_read.rs"]
pub mod matching_read;

pub struct RetentionHistory;
worth_query_portable_type!(RetentionHistory => "worth.query.document-retention.history.v1");

impl ApplicationHistoryBinding<DocumentRetentionSchema> for RetentionHistory {
    type Access = DocumentRetentionRead;
    const IDENTITY: &'static str = "DocumentRetentionHistory";
}

/// A real authored label payload exercises record allocation without replacing
/// READ, action authorization, the business operation or native funding.
pub struct DetailedRetentionHistory;
worth_query_portable_type!(DetailedRetentionHistory => "worth.query.document-retention.detailed-history.v1");
impl ApplicationHistoryBinding<DocumentRetentionSchema> for DetailedRetentionHistory {
    type Access = DocumentRetentionRead;
    const IDENTITY: &'static str = "DetailedDocumentRetentionHistory";
}

pub fn detailed_edit_label() -> String {
    "Set retention ".repeat(4096)
}

pub struct GovernedRetentionHistory;
worth_query_portable_type!(GovernedRetentionHistory => "worth.query.document-retention.governed-history.v1");
impl ApplicationHistoryBinding<DocumentRetentionSchema> for GovernedRetentionHistory {
    type Access = DocumentRetentionRead;
    const IDENTITY: &'static str = "GovernedDocumentRetentionHistory";
}

pub fn metadata_disclosure() -> ApplicationQueryDisclosureContract {
    metadata_disclosure_for::<GovernedRetentionHistory>()
}

fn metadata_disclosure_for<
    H: worth_query_decl::facade::portable_identity::WorthQueryPortableType,
>() -> ApplicationQueryDisclosureContract {
    use super::record_history::capability;
    let mut disclosure = ApplicationQueryDisclosureContract::governed_by(
        "document-history-metadata",
        capability::MetadataCapability::reference(),
    );
    for facet in ApplicationHistoryMetadataFacet::ALL {
        disclosure = disclosure.disclose_history_metadata_by(
            ApplicationHistoryMetadataRef::<H>::new(facet),
            capability::encoded(),
            ApplicationQueryInfluenceContract::permit([facet.influence()]),
        );
    }
    disclosure
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    schema.application_query(matching_read::definition())
        .application_query_binding::<matching_read::Binding>()
        .application_history(
            ApplicationHistoryDefinition::<DocumentRetentionSchema, matching_read::History>::for_binding(metadata_disclosure_for::<matching_read::History>())
                .operation::<SetRetentionBinding>("Set retention"),
        ).application_history(
        ApplicationHistoryDefinition::<DocumentRetentionSchema, RetentionHistory>::for_binding(ApplicationQueryDisclosureContract::public())
            .model_domain(ApplicationHistoryModelDomain::ScopeEntityRecord)
            .operation::<SetRetentionBinding>("Set retention"),
    ).application_history(
        ApplicationHistoryDefinition::<DocumentRetentionSchema, DetailedRetentionHistory>::for_binding(ApplicationQueryDisclosureContract::public())
            .operation::<SetRetentionBinding>(detailed_edit_label()),
    ).application_history(
        ApplicationHistoryDefinition::<DocumentRetentionSchema, GovernedRetentionHistory>::for_binding(metadata_disclosure())
            .model_domain(ApplicationHistoryModelDomain::ScopeEntityRecord)
            .operation::<SetRetentionBinding>("Set retention"),
    )
}
