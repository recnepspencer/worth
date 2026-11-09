//! Window and bookmark purposes have independently revocable real grants.
use super::super::{
    record_history::capability, retention_entry::DocumentRetentionRead,
    schema::DocumentRetentionSchema, workflow::declaration::DocumentIdentityField,
};
use worth_query_decl::facade::{
    application_history::*, application_schema::ApplicationSchemaDeclarationBuilder,
    worth_query_capability, worth_query_operation,
};

worth_query_capability!(pub WindowCapability in DocumentRetentionSchema);
worth_query_capability!(pub BookmarkCapability in DocumentRetentionSchema);
worth_query_operation!(pub ChangeWindow for DocumentRetentionSchema, input capability::InputBinding);
worth_query_operation!(pub ChangeBookmark for DocumentRetentionSchema, input capability::InputBinding);

impl capability::Role for WindowCapability {
    const ACTION: &'static str = "change-document-history-window";
}
impl capability::Role for BookmarkCapability {
    const ACTION: &'static str = "change-document-history-bookmark";
}
pub const WINDOW_GRANT: &str = "document-history-window-grant";
pub const BOOKMARK_GRANT: &str = "document-history-bookmark-grant";

pub struct Window;
impl<H: ApplicationHistoryBinding<DocumentRetentionSchema, Access = DocumentRetentionRead>>
    ApplicationHistoryRetentionBinding<DocumentRetentionSchema, H> for Window
{
    const IDENTITY: &'static str = "DocumentHistoryWindow";
    type Capability = WindowCapability;
    type Input = capability::Input;
    type InputBinding = capability::InputBinding;
    type Operation = ChangeWindow;
    fn input(access: &DocumentRetentionRead) -> Self::Input {
        capability::Input {
            identity: access.identity.clone(),
        }
    }
}
impl<H: ApplicationHistoryBinding<DocumentRetentionSchema, Access = DocumentRetentionRead>>
    ApplicationHistoryWindowBinding<DocumentRetentionSchema, H> for Window
{
}

pub struct Bookmark;
impl<H: ApplicationHistoryBinding<DocumentRetentionSchema, Access = DocumentRetentionRead>>
    ApplicationHistoryRetentionBinding<DocumentRetentionSchema, H> for Bookmark
{
    const IDENTITY: &'static str = "DocumentHistoryBookmark";
    type Capability = BookmarkCapability;
    type Input = capability::Input;
    type InputBinding = capability::InputBinding;
    type Operation = ChangeBookmark;
    fn input(access: &DocumentRetentionRead) -> Self::Input {
        capability::Input {
            identity: access.identity.clone(),
        }
    }
}
impl<H: ApplicationHistoryBinding<DocumentRetentionSchema, Access = DocumentRetentionRead>>
    ApplicationHistoryBookmarkBinding<DocumentRetentionSchema, H> for Bookmark
{
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    let window = ChangeWindow::reference();
    let bookmark = ChangeBookmark::reference();
    schema
        .operation(
            window
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_field(window, DocumentIdentityField::at("DocumentIdentityField"))
        .capability(capability::contract(WindowCapability::reference(), window))
        .operation(
            bookmark
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_field(bookmark, DocumentIdentityField::at("DocumentIdentityField"))
        .capability(capability::contract(
            BookmarkCapability::reference(),
            bookmark,
        ))
}
