//! Derived declarations, including explicit wire-identity overrides, lower
//! to the same canonical schema as the same members written by hand.
use super::schema::{Board, PrincipalBinding, Schema};
use worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder;

mod action;
mod projection;
mod record;

fn board() -> ApplicationSchemaDeclarationBuilder<Schema> {
    ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(Board)
        .expect("the board fragment lowers")
        .principal_binding(PrincipalBinding::reference())
}
