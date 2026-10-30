//! One installed audit surface reuses the real retention operation and grant owner.
pub use super::{retention_entry::DocumentRetentionQueryBinding, schema::*};
pub use worth_query_decl::facade::{
    application_query::*, application_schema::*, worth_query_portable_type,
};

#[path = "record_history/access.rs"]
pub mod access;
#[path = "record_history/capability.rs"]
pub mod capability;
#[path = "record_history/court.rs"]
pub mod court;
#[path = "record_history/grants.rs"]
pub mod grants;
#[path = "record_history/query.rs"]
pub mod query;
#[path = "record_history/reaffirm.rs"]
pub mod reaffirm;
#[path = "record_history/stimuli.rs"]
pub mod stimuli;
pub use grants::seed;

pub struct PublicChanges;
pub struct GovernedChanges;
worth_query_portable_type!(PublicChanges => "worth.query.certification.record-history.public.v1");
worth_query_portable_type!(GovernedChanges => "worth.query.certification.record-history.governed.v1");
impl ApplicationHistoryQueryBinding<DocumentRetentionSchema> for PublicChanges {
    type SubjectRead = DocumentRetentionQueryBinding;
    const IDENTITY: &'static str = "DocumentChanges";
}
impl ApplicationHistoryQueryBinding<DocumentRetentionSchema> for GovernedChanges {
    type SubjectRead = query::Binding;
    const IDENTITY: &'static str = "GovernedDocumentChanges";
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    let mut disclosure = ApplicationQueryDisclosureContract::governed_by(
        "record-history-metadata",
        capability::MetadataCapability::reference(),
    );
    for facet in ApplicationHistoryMetadataFacet::ALL {
        disclosure = disclosure.disclose_history_metadata_by(
            ApplicationHistoryMetadataRef::<GovernedChanges>::new(facet),
            capability::encoded(),
            ApplicationQueryInfluenceContract::permit([facet.influence()]),
        );
    }
    reaffirm::declare(capability::declare(schema))
        .application_query(query::definition())
        .application_query_binding::<query::Binding>()
        .application_history_query(
            ApplicationHistoryQueryDefinition::<DocumentRetentionSchema, PublicChanges>::entity(
                Document::reference(),
                ApplicationQueryDisclosureContract::public(),
            )
            .aspect(DocumentFacts::reference())
            .finish()
            .unwrap(),
        )
        .application_history_query(
            ApplicationHistoryQueryDefinition::<DocumentRetentionSchema, GovernedChanges>::entity(
                Document::reference(),
                disclosure,
            )
            .aspect(DocumentFacts::reference())
            .finish()
            .unwrap(),
        )
}
