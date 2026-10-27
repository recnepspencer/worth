//! One installed audit surface reuses the real retention operation and grant owner.
use super::{retention_entry::DocumentRetentionQueryBinding, schema::*};
use worth_query_decl::facade::{
    application_query::*, application_schema::*, worth_query_portable_type,
};

#[cfg(test)]
#[path = "record_history/access.rs"]
mod access;
#[path = "record_history/capability.rs"]
mod capability;
#[cfg(test)]
#[path = "record_history/court.rs"]
mod court;
#[path = "record_history/grants.rs"]
mod grants;
#[path = "record_history/query.rs"]
mod query;
#[path = "record_history/reaffirm.rs"]
pub(super) mod reaffirm;
#[cfg(test)]
#[path = "record_history/stimuli.rs"]
mod stimuli;
pub(super) use grants::seed;

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

pub(super) fn declare(
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
