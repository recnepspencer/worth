//! Exclusive holders survive retirement of their optional current snapshots.
use super::super::schema::*;
use std::num::NonZeroUsize;
use worth_query_decl::facade::{
    worth_query_operation_creates, worth_query_operation_deletes, worth_query_operation_links,
    worth_query_operation_unlinks,
};
use worth_query_host::facade::{
    declaration::application_schema::*, worth_query_aspect, worth_query_entity, worth_query_field,
    worth_query_operation_reads, worth_query_operation_writes,
};

worth_query_entity!(pub Snapshot for Schema: BodySchemaBinding);
worth_query_aspect!(pub SnapshotFacts for Schema: BodySchemaBinding, Snapshot; identity = AspectIdentity(0x9175B007), revision = AspectContractRevision(1),);
worth_query_field!(pub SnapshotIdentity for Schema: BodySchemaBinding, Snapshot, SnapshotFacts: u64 => U64ApplicationValueBinding, read_only, equality);
pub struct OwnedRows;
pub struct CurrentSelection;
pub struct AlternateSelection;
impl OwnedRows {
    pub fn reference<S: BodySchemaBinding>() -> ApplicationRelationRef<S, Self, Document, Audit> {
        let base = ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
        ApplicationRelationRef::from_schema_identifiers(
            "OwnedRows",
            "Document",
            "Audit",
            ApplicationRelationIntegrity::new(
                base.endpoints,
                ApplicationRelationCardinality::new(None, None, None, Some(1), None, Some(1)),
                base.deletion,
            ),
        )
    }
}
macro_rules! selection_relation {
    ($name:ident) => {
        impl $name {
            pub fn reference<S: BodySchemaBinding>(
            ) -> ApplicationRelationRef<S, Self, Audit, Snapshot> {
                let base = ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
                ApplicationRelationRef::from_schema_identifiers(
                    stringify!($name),
                    "Audit",
                    "Snapshot",
                    ApplicationRelationIntegrity::new(
                        base.endpoints,
                        ApplicationRelationCardinality::new(
                            None,
                            Some(1),
                            None,
                            Some(1),
                            None,
                            Some(1),
                        ),
                        base.deletion,
                    ),
                )
            }
        }
    };
}
selection_relation!(CurrentSelection);
selection_relation!(AlternateSelection);
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
worth_query_operation_reads!(ReplaceSelection => [Audit, AuditIdentityField, CurrentSelection]);
worth_query_operation_creates!(ReplaceSelection => [Snapshot]);
worth_query_operation_writes!(ReplaceSelection => [SnapshotIdentity]);
worth_query_operation_links!(ReplaceSelection => [CurrentSelection]);
worth_query_operation_unlinks!(ReplaceSelection => [CurrentSelection]);

pub fn declare<const M: u64>(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema<M>>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema<M>> {
    if !matches!(M, 120..=125) {
        return schema;
    }
    schema
        .entity(Snapshot::reference())
        .aspect(Snapshot::reference(), SnapshotFacts::reference())
        .field(Snapshot::reference(), SnapshotIdentity::reference())
        .relation(
            OwnedRows::reference(),
            Document::reference(),
            Audit::reference(),
        )
        .relation(
            CurrentSelection::reference(),
            Audit::reference(),
            Snapshot::reference(),
        )
        .relation(
            AlternateSelection::reference(),
            Audit::reference(),
            Snapshot::reference(),
        )
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
                        .initialize_input(SnapshotIdentity::reference(), "retention_days")
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
            AuditIdentityField::reference(),
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
        body.set_optional_input(DocumentDraftNoteField::reference(), "identity")
            .unwrap()
            .retire_owned_selections_on_input_change(
                DocumentDraftNoteField::reference(),
                "identity",
                OwnedRows::reference(),
                CurrentSelection::reference(),
                NonZeroUsize::new(2).unwrap(),
            )
            .unwrap()
    } else {
        body.retire_owned_selections_on_input_change(
            DocumentRetentionField::reference(),
            "retention_days",
            OwnedRows::reference(),
            CurrentSelection::reference(),
            NonZeroUsize::new(2).unwrap(),
        )
        .unwrap()
    };
    let body = if M == 122 {
        body.retire_owned_selections_on_input_change(
            DocumentRetentionField::reference(),
            "retention_days",
            OwnedRows::reference(),
            AlternateSelection::reference(),
            NonZeroUsize::new(2).unwrap(),
        )
        .unwrap()
    } else {
        body
    };
    body.assign_input(DocumentRetentionField::reference(), "retention_days")
        .unwrap()
        .create_in_scope(Audit::reference(), "audit")
        .unwrap()
        .initialize_input(AuditIdentityField::reference(), "identity")
        .unwrap()
        .initialize_input(AuditRetentionField::reference(), "retention_days")
        .unwrap()
        .initialize_from_scope(
            DocumentRetentionField::reference(),
            AuditPriorRetentionField::reference(),
        )
        .unwrap()
        .link_from_scope(DocumentMembers::reference(), "audit-owner")
        .unwrap()
        .finish()
        .unwrap()
}
