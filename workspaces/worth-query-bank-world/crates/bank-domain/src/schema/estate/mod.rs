mod authority_relation_installation;
mod capability_composition;
mod capability_contract_installation;
mod capability_contracts;
mod capability_elevation;
mod capability_installation;
mod effect_installation;
mod effects;
mod entities;
mod estate_relation_installation;
mod fields;
mod member_installation;
mod operation_program_installation;
mod policies;
mod policy_installation;
mod relations;
mod request_projection;
mod values;

pub use capability_contracts::*;
pub use effects::*;
pub use entities::*;
pub use fields::*;
pub use policies::*;
pub use relations::*;
pub use values::*;

use worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder;

use crate::schema::BankSchema;

pub(crate) fn install_estate_world(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let schema = member_installation::install(schema);
    let schema = capability_installation::install(schema);
    let schema = estate_relation_installation::install(schema);
    let schema = authority_relation_installation::install(schema);
    let schema = policy_installation::install(schema);
    let schema = capability_contract_installation::install(schema);
    let schema = effect_installation::install(schema);
    operation_program_installation::install(schema)
}
