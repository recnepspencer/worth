//! The bounded edit operation's schema and declared reads and effects.
use super::*;

/// The edit's operation: it resolves the set or the entry it changes.
pub(in super::super) fn declare<Schema: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let operation = EditEntry::reference::<Schema>();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(operation, 4_096)
        .operation_read_entity(operation, Body::reference())
        .operation_read_field(operation, BodyKey::reference())
        .operation_read_entity(operation, EntrySet::reference())
        .operation_read_field(operation, EntrySetKey::reference())
        .operation_read_field(operation, EntrySetWeight::reference())
        .operation_read_relation(operation, EntrySetMember::reference())
        .operation_read_entity(operation, SetEntry::reference())
        .operation_read_field(operation, EntryNumber::reference())
        .operation_read_field(operation, EntryRegion::reference())
        .operation_read_field(operation, EntryValueBits::reference())
        .operation_read_field(operation, EntryWork::reference())
        .operation_read_field(operation, EntryFault::reference())
        .operation_write(operation, EntrySetWeight::reference())
        .operation_write(operation, EntryNumber::reference())
        .operation_write(operation, EntryRegion::reference())
        .operation_write(operation, EntryValueBits::reference())
        .operation_write(operation, EntryWork::reference())
        .operation_write(operation, EntryFault::reference())
        .operation_create(operation, SetEntry::reference())
        .operation_delete(operation, SetEntry::reference())
        .operation_link(operation, EntrySetMember::reference())
        .operation_unlink(operation, EntrySetMember::reference())
        .application_mutation_binding::<EntryEditBinding<Schema>>()
}
