use worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder;

use crate::schema::BankSchema;

pub(super) fn install(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    install_elevation_relations(install_capability_relations(schema))
}

fn install_capability_relations(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
}

fn install_elevation_relations(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    schema
}
