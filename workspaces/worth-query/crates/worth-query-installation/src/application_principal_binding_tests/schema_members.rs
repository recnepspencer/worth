//! Root membership is lowered once before optional inspection authoring.
use super::*;
use worth_query_declaration::facade::application_schema::{
    fragment, ApplicationSchemaDeclarationBuilder,
};
pub(crate) fn declare() -> ApplicationSchemaDeclarationBuilder<IdentitySchema> {
    fragment::declaration_builder(IdentitySchema).unwrap()
}
