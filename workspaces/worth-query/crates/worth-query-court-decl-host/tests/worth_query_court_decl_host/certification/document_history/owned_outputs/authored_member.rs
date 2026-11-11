//! A declared authored member retirement supplies Native's real cascade anchor.
use super::*;
pub mod correction;
pub mod restore;
pub mod retire;
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    restore::declare(schema).application_mutation_binding::<retire::Binding>()
}
