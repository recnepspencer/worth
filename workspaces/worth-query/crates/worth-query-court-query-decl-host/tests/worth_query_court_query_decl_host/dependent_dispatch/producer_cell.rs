//! A child-owned auxiliary entity, never included in the parent's output witness.
use super::*;
worth_query_entity!(pub ProducerCell for Schema);
worth_query_aspect!(pub CellIdentity for Schema, ProducerCell; identity = AspectIdentity(0x9176A106), revision = AspectContractRevision(1));
worth_query_aspect!(pub CellComputation for Schema, ProducerCell; identity = AspectIdentity(0x9176A107), revision = AspectContractRevision(1));
worth_query_field!(pub CellKey for Schema, ProducerCell, CellIdentity: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub CellValue for Schema, ProducerCell, CellComputation: u64 => U64ApplicationValueBinding, read_write, equality);
pub type CellTarget = WorthQueryInvariantMutationTarget<Schema, ProducerCell>;
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .entity(ProducerCell::reference())
        .aspect(ProducerCell::reference(), CellIdentity::reference())
        .optional_aspect(ProducerCell::reference(), CellComputation::reference())
        .unique_field(ProducerCell::reference(), CellKey::reference())
        .field(ProducerCell::reference(), CellValue::reference())
}
