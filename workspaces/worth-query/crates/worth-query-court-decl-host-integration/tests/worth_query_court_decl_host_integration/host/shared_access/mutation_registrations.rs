//! Schema registrations for the shared-access court's durable operations and mutation bindings.

use worth_query_decl::facade::application_schema::{
    fragment::Registration, ApplicationSchemaDeclarationBuilder, ApplicationSchemaDeclarationDenial,
};

use super::mutations::{
    GrantProjectAccess, GrantProjectAccessBinding, RevokeProjectAccess, RevokeProjectAccessBinding,
    WriteProjectVersion, WriteProjectVersionBinding,
};
use super::query::ProjectUpdatedEffect;
use super::schema::{
    GrantActionField, GrantActiveField, GrantDelegationLimitField, GrantDisclosureField,
    GrantIdField, GrantModeField, GrantNotAfterField, GrantNotBeforeField, GrantPurposeField,
    GrantResource, GrantWorkflowField, Grantee, Grantor, PrincipalIdField, ProjectGrant,
    ProjectKeyField, ProjectVersionField, SharedProjectSchema,
};

impl Registration<SharedProjectSchema> for WriteProjectVersion {
    fn register(
        s: ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(s.operation(
            Self::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(Self::reference(), 64)
        .operation_read_field(Self::reference(), ProjectKeyField::at("ProjectKey"))
        .operation_read_field(Self::reference(), ProjectVersionField::at("ProjectVersion"))
        .operation_write(Self::reference(), ProjectVersionField::at("ProjectVersion"))
        .operation_emit(Self::reference(), ProjectUpdatedEffect::reference()))
    }
}

impl Registration<SharedProjectSchema> for WriteProjectVersionBinding {
    fn register(
        s: ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(s.application_mutation_binding::<Self>())
    }
}

impl Registration<SharedProjectSchema> for GrantProjectAccess {
    fn register(
        s: ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(s.operation(
            Self::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(Self::reference(), 64)
        .operation_read_field(Self::reference(), ProjectKeyField::at("ProjectKey"))
        .operation_read_field(Self::reference(), PrincipalIdField::at("PrincipalIdField"))
        .operation_create(Self::reference(), ProjectGrant::reference())
        .operation_write(Self::reference(), GrantIdField::at("GrantId"))
        .operation_write(Self::reference(), GrantActionField::at("GrantAction"))
        .operation_write(Self::reference(), GrantPurposeField::at("GrantPurpose"))
        .operation_write(
            Self::reference(),
            GrantDisclosureField::at("GrantDisclosure"),
        )
        .operation_write(Self::reference(), GrantModeField::at("GrantMode"))
        .operation_write(Self::reference(), GrantActiveField::at("GrantActive"))
        .operation_write(Self::reference(), GrantWorkflowField::at("GrantWorkflow"))
        .operation_write(Self::reference(), GrantNotBeforeField::at("GrantNotBefore"))
        .operation_write(Self::reference(), GrantNotAfterField::at("GrantNotAfter"))
        .operation_write(
            Self::reference(),
            GrantDelegationLimitField::at("GrantDelegationLimit"),
        )
        .operation_link(Self::reference(), Grantor::reference())
        .operation_link(Self::reference(), Grantee::reference())
        .operation_link(Self::reference(), GrantResource::reference()))
    }
}

impl Registration<SharedProjectSchema> for GrantProjectAccessBinding {
    fn register(
        s: ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(s.application_mutation_binding::<Self>())
    }
}

impl Registration<SharedProjectSchema> for RevokeProjectAccess {
    fn register(
        s: ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(s.operation(
            Self::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(Self::reference(), 64)
        .operation_read_field(Self::reference(), GrantIdField::at("GrantId"))
        .operation_read_field(Self::reference(), GrantActiveField::at("GrantActive"))
        .operation_write(Self::reference(), GrantActiveField::at("GrantActive")))
    }
}

impl Registration<SharedProjectSchema> for RevokeProjectAccessBinding {
    fn register(
        s: ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(s.application_mutation_binding::<Self>())
    }
}
