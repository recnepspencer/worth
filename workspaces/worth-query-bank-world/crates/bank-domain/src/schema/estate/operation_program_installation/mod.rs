mod capability_revocation;
mod delegation;
mod disburse_estate;
mod freeze_account;
mod lifecycle;
mod notify_death;
mod open_estate_case;
mod recognize_executor;
mod release_estate;
mod retransmit_death_notice;

use worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder;

use crate::schema::{BankSchema, EstateCaseIdentityField, ViewRestrictedEstateOperation};

pub(super) fn install(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let schema = schema.operation_read_field(
        ViewRestrictedEstateOperation::reference(),
        EstateCaseIdentityField::reference(),
    );
    let schema = lifecycle::install(schema);
    let schema = freeze_account::install(schema);
    let schema = delegation::install(schema);
    let schema = capability_revocation::install(schema);
    let schema = recognize_executor::install(schema);
    let schema = notify_death::install(schema);
    let schema = retransmit_death_notice::install(schema);
    let schema = open_estate_case::install(schema);
    let schema = disburse_estate::install(schema);
    release_estate::install(schema)
}
