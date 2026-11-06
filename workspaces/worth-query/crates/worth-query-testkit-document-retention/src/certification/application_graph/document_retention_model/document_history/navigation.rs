//! An independently revocable snapshot-navigation capability on the real document.
use super::super::workflow::declaration::DocumentIdentityField;
use super::super::{
    record_history::capability, retention_entry::DocumentRetentionRead,
    schema::DocumentRetentionSchema,
};
use worth_query_decl::facade::{
    application_history::{ApplicationHistoryBinding, ApplicationHistoryNavigationBinding},
    application_schema::ApplicationSchemaDeclarationBuilder,
    worth_query_capability, worth_query_operation, worth_query_operation_reads,
};

worth_query_capability!(pub NavigationCapability in DocumentRetentionSchema);
worth_query_operation!(pub SelectSnapshot for DocumentRetentionSchema, input capability::InputBinding);
worth_query_operation_reads!(SelectSnapshot => [DocumentIdentityField]);

impl capability::Role for NavigationCapability {
    const ACTION: &'static str = "select-document-snapshot";
}

pub const GRANT: &str = "document-history-navigation-grant";

pub struct Navigation;
impl<H: ApplicationHistoryBinding<DocumentRetentionSchema, Access = DocumentRetentionRead>>
    ApplicationHistoryNavigationBinding<DocumentRetentionSchema, H> for Navigation
{
    const IDENTITY: &'static str = "DocumentSnapshotNavigation";
    type Capability = NavigationCapability;
    type Input = capability::Input;
    type InputBinding = capability::InputBinding;
    type Operation = SelectSnapshot;
    fn input(access: &DocumentRetentionRead) -> Self::Input {
        capability::Input {
            identity: access.identity.clone(),
        }
    }
}

pub struct GovernedPayloadNavigation;
impl
    ApplicationHistoryNavigationBinding<
        DocumentRetentionSchema,
        super::GovernedPayloadRetentionHistory,
    > for GovernedPayloadNavigation
{
    const IDENTITY: &'static str = "GovernedPayloadSnapshotNavigation";
    type Capability = NavigationCapability;
    type Input = capability::Input;
    type InputBinding = capability::InputBinding;
    type Operation = SelectSnapshot;
    fn input(access: &super::super::record_history::query::Input) -> Self::Input {
        capability::Input {
            identity: access.identity.clone(),
        }
    }
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    let operation = SelectSnapshot::reference();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_field(operation, DocumentIdentityField::reference())
        .capability(capability::contract(
            NavigationCapability::reference(),
            operation,
        ))
}
