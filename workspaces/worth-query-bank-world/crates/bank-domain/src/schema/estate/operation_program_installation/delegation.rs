use worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder;

use super::super::*;
use crate::schema::{AccountIdentity, BankSchema, InstitutionIdentityField};

pub(super) fn install(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let operation = DelegateEstateCapabilityOperation::reference();
    schema
        .operation_decision_fact_budget(operation, 8)
        .operation_projection_work_budget(operation, 160)
        .operation_read_field(operation, AccountIdentity::reference())
        .operation_read_field(operation, InstitutionIdentityField::reference())
        .operation_read_field(operation, BranchIdentityField::reference())
        .operation_read_field(operation, CapabilityGrantIdentityField::reference())
        .operation_read_relation(operation, EstateBranch::reference())
        .operation_read_relation(operation, BranchInstitution::reference())
        .operation_read_relation(operation, EstateAccount::reference())
}
