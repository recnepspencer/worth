use super::*;

pub(super) fn principal_meaning_comparison_work<
    Schema,
    Binding,
    Mapping,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityBinding,
>(
    member: &ApplicationSchemaMember,
    binding: &ApplicationPrincipalBindingRef<
        Schema,
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >,
) -> Option<u64> {
    let ApplicationSchemaMember::PrincipalBinding {
        binding: name,
        mapping_entity,
        identity_aspect,
        identity_field,
        status_aspect,
        status_field,
        target_relation,
        principal_entity,
        principal_identity_aspect,
        principal_identity_field,
        principal_identity_value_type,
        ..
    } = member
    else {
        return Some(1);
    };
    let pairs = [
        (name.as_str(), binding.name()),
        (mapping_entity.as_str(), binding.mapping_entity()),
        (identity_aspect.as_str(), binding.identity_aspect()),
        (identity_field.as_str(), binding.identity_field()),
        (status_aspect.as_str(), binding.status_aspect()),
        (status_field.as_str(), binding.status_field()),
        (target_relation.as_str(), binding.target_relation()),
        (principal_entity.as_str(), binding.principal_entity()),
        (
            principal_identity_aspect.as_str(),
            binding.principal_identity_aspect(),
        ),
        (
            principal_identity_field.as_str(),
            binding.principal_identity_field(),
        ),
        (
            principal_identity_value_type.as_str(),
            binding.principal_identity_value_type(),
        ),
    ];
    pairs.into_iter().try_fold(2_u64, |work, (left, right)| {
        work.checked_add(u64::try_from(left.len().max(right.len())).ok()?)?
            .checked_add(1)
    })
}

pub(super) fn principal_binding_copy_bytes<
    Schema,
    Binding,
    Mapping,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityBinding,
>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    binding: &ApplicationPrincipalBindingRef<
        Schema,
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >,
    value: &super::super::value_binding::WorthQueryInstalledApplicationValueBinding,
) -> Option<u64>
where
    Schema: ApplicationSchema,
{
    let texts = [
        schema.owner(),
        schema.schema_name(),
        binding.name(),
        binding.mapping_entity(),
        binding.identity_aspect(),
        binding.identity_field(),
        binding.status_aspect(),
        binding.status_field(),
        binding.target_relation(),
        binding.principal_entity(),
        binding.principal_identity_aspect(),
        binding.principal_identity_field(),
        binding.principal_identity_value_type(),
        value.locus().entity(),
        value.locus().aspect(),
        value.locus().field(),
    ];
    texts.into_iter().try_fold(64_u64, |sum, text| {
        // AuthoritySeal::from_bytes retains exactly sixty-four hex bytes.
        sum.checked_add(u64::try_from(text.len()).ok()?)
    })
}

pub(super) fn principal_binding_construction_work<
    Schema,
    Binding,
    Mapping,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityBinding,
>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    binding: &ApplicationPrincipalBindingRef<
        Schema,
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >,
    copy_bytes: u64,
) -> Option<u64>
where
    Schema: ApplicationSchema,
{
    let fields = [
        ("package", 5_u64, 32_u64),
        ("schema", 5, 32),
        ("binding", 4, u64::try_from(binding.name().len()).ok()?),
        (
            "mapping-entity",
            4,
            u64::try_from(binding.mapping_entity().len()).ok()?,
        ),
        (
            "identity-aspect",
            4,
            u64::try_from(binding.identity_aspect().len()).ok()?,
        ),
        (
            "identity-field",
            4,
            u64::try_from(binding.identity_field().len()).ok()?,
        ),
        (
            "status-aspect",
            4,
            u64::try_from(binding.status_aspect().len()).ok()?,
        ),
        (
            "status-field",
            4,
            u64::try_from(binding.status_field().len()).ok()?,
        ),
        (
            "target-relation",
            4,
            u64::try_from(binding.target_relation().len()).ok()?,
        ),
        (
            "principal-entity",
            4,
            u64::try_from(binding.principal_entity().len()).ok()?,
        ),
        (
            "principal-identity-aspect",
            4,
            u64::try_from(binding.principal_identity_aspect().len()).ok()?,
        ),
        (
            "principal-identity-field",
            4,
            u64::try_from(binding.principal_identity_field().len()).ok()?,
        ),
        (
            "principal-identity-scalar-family",
            4,
            u64::try_from(
                binding
                    .principal_identity_scalar_family()
                    .canonical_name()
                    .len(),
            )
            .ok()?,
        ),
        (
            "principal-identity-value-type",
            4,
            u64::try_from(binding.principal_identity_value_type().len()).ok()?,
        ),
    ];
    let domain = AuthoritySealDomain::InstalledPrincipalBinding;
    let mut encoded = 8_u64
        .checked_add(u64::try_from("domain".len()).ok()?)?
        .checked_add(8)?
        .checked_add(u64::try_from(domain.label().len()).ok()?)?;
    for (tag, kind, value) in fields {
        encoded = encoded
            .checked_add(24)?
            .checked_add(u64::try_from(tag.len()).ok()?)?
            .checked_add(kind)?
            .checked_add(value)?;
    }
    let blocks = encoded.checked_add(64)?.checked_add(9)?.div_ceil(64);
    let _ = schema;
    copy_bytes
        .checked_add(encoded)?
        .checked_add(128 + 32 + 128)?
        .checked_add(blocks)?
        .checked_add(16)
}
