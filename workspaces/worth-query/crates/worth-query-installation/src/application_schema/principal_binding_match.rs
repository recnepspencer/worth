use worth_query_declaration::facade::application_schema::{
    ApplicationPrincipalBindingRef, ApplicationSchemaMember,
};

pub(super) fn principal_binding_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::PrincipalBinding { binding, .. } => Some(binding),
        _ => None,
    }
}

pub(super) fn principal_binding_matches<
    Schema,
    Binding,
    Mapping,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityBinding,
>(
    member: &ApplicationSchemaMember,
    reference: &ApplicationPrincipalBindingRef<
        Schema,
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >,
) -> bool {
    matches!(
        member,
        ApplicationSchemaMember::PrincipalBinding {
            binding,
            mapping_entity,
            identity_aspect,
            identity_field,
            status_aspect,
            status_field,
            target_relation,
            principal_entity,
            principal_identity_aspect,
            principal_identity_field,
            principal_identity_scalar_family,
            principal_identity_value_type,
        } if binding == reference.name()
            && mapping_entity == reference.mapping_entity()
            && identity_aspect == reference.identity_aspect()
            && identity_field == reference.identity_field()
            && status_aspect == reference.status_aspect()
            && status_field == reference.status_field()
            && target_relation == reference.target_relation()
            && principal_entity == reference.principal_entity()
            && principal_identity_aspect == reference.principal_identity_aspect()
            && principal_identity_field == reference.principal_identity_field()
            && *principal_identity_scalar_family == reference.principal_identity_scalar_family()
            && principal_identity_value_type == reference.principal_identity_value_type()
    )
}
