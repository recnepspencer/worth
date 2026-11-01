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

#[path = "document_history/control_roles.rs"]
pub mod control_roles;
#[path = "document_history/matching_read.rs"]
pub mod matching_read;
#[path = "document_history/navigation.rs"]
pub mod navigation;
#[path = "document_history/retention_policy.rs"]
pub mod retention_policy;

pub struct RetentionHistory;
worth_query_portable_type!(RetentionHistory => "worth.query.document-retention.history.v1");

impl ApplicationHistoryBinding<DocumentRetentionSchema> for RetentionHistory {
    type Access = DocumentRetentionRead;
    const IDENTITY: &'static str = "DocumentRetentionHistory";
    const RETENTION_RESOURCES: worth_query_decl::facade::application_history::ApplicationHistoryRetentionResources = worth_query_decl::facade::application_history::ApplicationHistoryRetentionResources {
        maximum_bookmarks: std::num::NonZeroUsize::new(2).unwrap(),
        ..worth_query_decl::facade::application_history::ApplicationHistoryRetentionResources::DEFAULT
    };
}

/// Distinct installed counts expose result admission independently of membership.
pub struct ResultBoundedRetentionHistory;
worth_query_portable_type!(ResultBoundedRetentionHistory => "worth.query.document-retention.result-bounded-history.v1");
impl ApplicationHistoryBinding<DocumentRetentionSchema> for ResultBoundedRetentionHistory {
    type Access = DocumentRetentionRead;
    const IDENTITY: &'static str = "ResultBoundedDocumentRetentionHistory";
    const GROUP_RESOURCES:
        worth_query_decl::facade::application_history::ApplicationHistoryGroupResources =
        worth_query_decl::facade::application_history::ApplicationHistoryGroupResources {
            maximum_members: std::num::NonZeroUsize::new(4).unwrap(),
            maximum_retained_results: std::num::NonZeroUsize::MIN,
        };
}

/// A real authored label payload exercises record allocation without replacing
/// READ, action authorization, the business operation or native funding.
pub struct DetailedRetentionHistory;
worth_query_portable_type!(DetailedRetentionHistory => "worth.query.document-retention.detailed-history.v1");
impl ApplicationHistoryBinding<DocumentRetentionSchema> for DetailedRetentionHistory {
    type Access = DocumentRetentionRead;
    const IDENTITY: &'static str = "DetailedDocumentRetentionHistory";
}

/// The real navigation recipe with one unfinished attempt per attached document.
pub struct NavigationBoundedRetentionHistory;
worth_query_portable_type!(NavigationBoundedRetentionHistory => "worth.query.document-retention.navigation-bounded-history.v1");
impl ApplicationHistoryBinding<DocumentRetentionSchema> for NavigationBoundedRetentionHistory {
    type Access = DocumentRetentionRead;
    const IDENTITY: &'static str = "NavigationBoundedDocumentRetentionHistory";
    const NAVIGATION_RESOURCES:
        worth_query_decl::facade::application_history::ApplicationHistoryNavigationResources =
        worth_query_decl::facade::application_history::ApplicationHistoryNavigationResources {
            maximum_unfinished_attempts: std::num::NonZeroUsize::MIN,
        };
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

/// Payload and metadata require their independently authored capability purposes.
pub struct GovernedPayloadRetentionHistory;
worth_query_portable_type!(GovernedPayloadRetentionHistory => "worth.query.document-retention.governed-payload-history.v1");
impl ApplicationHistoryBinding<DocumentRetentionSchema> for GovernedPayloadRetentionHistory {
    type Access = super::record_history::query::Input;
    const IDENTITY: &'static str = "GovernedPayloadDocumentRetentionHistory";
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
    control_roles::declare(retention_policy::declare(navigation::declare(schema)))
        .application_history(
            ApplicationHistoryDefinition::<DocumentRetentionSchema, NavigationBoundedRetentionHistory>::for_binding(ApplicationQueryDisclosureContract::public())
                .model_domain(ApplicationHistoryModelDomain::ScopeEntityRecord)
                .navigation::<navigation::Navigation>()
                .operation::<SetRetentionBinding>("Set retention"),
        )
        .application_history(
            ApplicationHistoryDefinition::<DocumentRetentionSchema, ResultBoundedRetentionHistory>::for_binding(ApplicationQueryDisclosureContract::public())
                .model_domain(ApplicationHistoryModelDomain::ScopeEntityRecord)
                .operation::<SetRetentionBinding>("Set retention"),
        ).application_query(matching_read::definition())
        .application_query_binding::<matching_read::Binding>()
        .application_history(
            ApplicationHistoryDefinition::<DocumentRetentionSchema, matching_read::History>::for_binding(metadata_disclosure_for::<matching_read::History>())
                .operation::<SetRetentionBinding>("Set retention"),
        ).application_history(
        ApplicationHistoryDefinition::<DocumentRetentionSchema, RetentionHistory>::for_binding(ApplicationQueryDisclosureContract::public())
            .model_domain(ApplicationHistoryModelDomain::ScopeEntityRecord)
            .navigation::<navigation::Navigation>()
            .window_policy::<retention_policy::Window>()
            .bookmark_policy::<retention_policy::Bookmark>()
            .operation::<SetRetentionBinding>("Set retention"),
    ).application_history(
        ApplicationHistoryDefinition::<DocumentRetentionSchema, DetailedRetentionHistory>::for_binding(ApplicationQueryDisclosureContract::public())
            .operation::<SetRetentionBinding>(detailed_edit_label()),
    ).application_history(
        ApplicationHistoryDefinition::<DocumentRetentionSchema, GovernedRetentionHistory>::for_binding(metadata_disclosure())
            .model_domain(ApplicationHistoryModelDomain::ScopeEntityRecord)
            .navigation::<navigation::Navigation>()
            .operation::<SetRetentionBinding>("Set retention"),
    ).application_history(
        ApplicationHistoryDefinition::<DocumentRetentionSchema, GovernedPayloadRetentionHistory>::for_binding(metadata_disclosure_for::<GovernedPayloadRetentionHistory>())
            .model_domain(ApplicationHistoryModelDomain::ScopeEntityRecord)
            .navigation::<navigation::GovernedPayloadNavigation>()
            .operation::<SetRetentionBinding>("Set retention"),
    )
}
