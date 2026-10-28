//! One actual typed schema assembly reused by ordinary and optional metadata courts.
use super::*;
use worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationBuilder;

pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<IdentitySchema>,
) -> ApplicationSchemaDeclarationBuilder<IdentitySchema> {
    resource::with_authorization(schema)
        .entity(ExternalMapping::reference())
        .entity(Principal::reference())
        .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
        .aspect(Principal::reference(), PrincipalIdentity::reference())
        .field(
            ExternalMapping::reference(),
            ExternalIdentityField::reference(),
        )
        .field(
            ExternalMapping::reference(),
            MappingStatusField::reference(),
        )
        .field(Principal::reference(), PrincipalIdentityField::reference())
        .relation(
            MappingTarget::reference(),
            ExternalMapping::reference(),
            Principal::reference(),
        )
        .principal_binding(IdentityBinding::reference())
        .application_query(principal_query_definition())
        .application_query_binding::<PrincipalQueryBinding>()
        .application_query_binding::<result_only::ResultOnlyQueryBinding>()
        .application_resource(resource::definition())
}
