pub use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::*;
pub use crate::domain_computation::primary_graph::tests::fixture::capability::elevated::*;

pub fn install(
    schema: worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationBuilder<
        crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema,
    >,
) -> worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationBuilder<
    crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema,
> {
    crate::domain_computation::primary_graph::tests::fixture::capability::elevated::contract::install(crate::domain_computation::primary_graph::tests::fixture::capability::contract::install(schema))
}

#[path = "contract/mod.rs"]
pub mod contract;
#[path = "declaration/mod.rs"]
pub mod declaration;
#[path = "elevated/mod.rs"]
pub mod elevated;
#[path = "touch_input_encoding/mod.rs"]
pub mod touch_input_encoding;
