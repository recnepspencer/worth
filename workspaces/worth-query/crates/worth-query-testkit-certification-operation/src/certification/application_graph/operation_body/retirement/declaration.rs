//! Exclusive holders survive retirement of their optional current snapshots.
pub use super::super::schema::*;
pub use std::num::NonZeroUsize;
pub use worth_query_decl::facade::{
    worth_query_operation_creates, worth_query_operation_deletes, worth_query_operation_links,
    worth_query_operation_unlinks,
};
pub use worth_query_host::facade::{
    declaration::application_schema::*, worth_query_operation_reads, worth_query_operation_writes,
};

worth_query_operation_reads!(SetRetention => [Snapshot, OwnedRows, CurrentSelection, AlternateSelection]);
worth_query_operation_deletes!(SetRetention => [Snapshot]);
worth_query_operation_unlinks!(SetRetention => [CurrentSelection, AlternateSelection]);

pub struct ReplaceSelection;
impl<S: BodySchemaBinding> ApplicationOperationMarkerIdentity<S> for ReplaceSelection {
    type InputBinding = S::InputBinding;
    const IDENTIFIER: &'static str = "ReplaceSelection";
}
impl ReplaceSelection {
    pub fn reference<S: BodySchemaBinding>() -> ApplicationOperationRef<S, Self, SetRetentionInput>
    {
        ApplicationOperationRef::from_declaration()
    }
}
worth_query_operation_reads!(ReplaceSelection => [Audit, CurrentSelection]);
worth_query_operation_creates!(ReplaceSelection => [Snapshot]);

worth_query_operation_links!(ReplaceSelection => [CurrentSelection]);
worth_query_operation_unlinks!(ReplaceSelection => [CurrentSelection]);

pub fn declare<const M: u64>(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema<M>>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema<M>> {
    if !matches!(M, 120..=125) {
        return schema;
    }
    schema
        .operation(
            ReplaceSelection::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish()
                .with_body(
                    ReplaceSelection::reference()
                        .body()
                        .in_scope(Audit::reference())
                        .unwrap()
                        .create_in_scope(Snapshot::reference(), "snapshot")
                        .unwrap()
                        .initialize_input(
                            SnapshotIdentity::at("SnapshotIdentity"),
                            "retention_days",
                        )
                        .unwrap()
                        .replace_from_scope(CurrentSelection::reference(), "selection")
                        .unwrap()
                        .finish()
                        .unwrap(),
                )
                .unwrap(),
        )
        .operation_read_field(
            ReplaceSelection::reference(),
            AuditIdentityField::at("AuditIdentityField"),
        )
        .operation_projection_work_budget(ReplaceSelection::reference(), 64)
        .application_mutation_binding::<super::binding::ReplaceBinding<M>>()
        .application_query(super::read::definition::<M>())
        .application_query_binding::<super::read::SnapshotQueryBinding<M>>()
}

pub fn body<const M: u64>(
) -> ApplicationOperationBody<DocumentRetentionSchema<M>, SetRetention, SetRetentionInput> {
    let body = SetRetention::reference::<DocumentRetentionSchema<M>>().body();
    let body = if M == 124 {
        body.set_optional_input(
            DocumentDraftNoteField::at("DocumentDraftNoteField"),
            "identity",
        )
        .unwrap()
        .retire_owned_selections_on_input_change(
            DocumentDraftNoteField::at("DocumentDraftNoteField"),
            "identity",
            OwnedRows::reference(),
            CurrentSelection::reference(),
            NonZeroUsize::new(2).unwrap(),
        )
        .unwrap()
    } else {
        body.retire_owned_selections_on_input_change(
            DocumentRetentionField::at("DocumentRetentionField"),
            "retention_days",
            OwnedRows::reference(),
            CurrentSelection::reference(),
            NonZeroUsize::new(2).unwrap(),
        )
        .unwrap()
    };
    let body = if M == 122 {
        body.retire_owned_selections_on_input_change(
            DocumentRetentionField::at("DocumentRetentionField"),
            "retention_days",
            OwnedRows::reference(),
            AlternateSelection::reference(),
            NonZeroUsize::new(2).unwrap(),
        )
        .unwrap()
    } else {
        body
    };
    body.assign_input(
        DocumentRetentionField::at("DocumentRetentionField"),
        "retention_days",
    )
    .unwrap()
    .create_in_scope(Audit::reference(), "audit")
    .unwrap()
    .initialize_input(AuditIdentityField::at("AuditIdentityField"), "identity")
    .unwrap()
    .initialize_input(
        AuditRetentionField::at("AuditRetentionField"),
        "retention_days",
    )
    .unwrap()
    .initialize_from_scope(
        DocumentRetentionField::at("DocumentRetentionField"),
        AuditPriorRetentionField::at("AuditPriorRetentionField"),
    )
    .unwrap()
    .link_from_scope(DocumentMembers::reference(), "audit-owner")
    .unwrap()
    .finish()
    .unwrap()
}

pub use crate::facade::certification::application_graph::operation_body::schema::storage_model::{
    AlternateSelection, CurrentSelection, OwnedRows, Snapshot, SnapshotFacts, SnapshotIdentity,
};
