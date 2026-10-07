use worth_query_declaration::facade::application_schema::ApplicationSchemaMember;

use crate::authority_cryptography::AuthoritySealDomain;

use super::WorthQueryInstalledPrincipalBinding;

impl<Schema, Binding, Mapping, Principal, PrincipalIdentity, PrincipalIdentityBinding>
    WorthQueryInstalledPrincipalBinding<
        Schema,
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >
{
    /// Initialized transcript bytes plus its SHA-256 block/finalization visits.
    /// This mirrors the existing transcript's field order and framing.
    pub(crate) fn authority_validation_work_bound(&self) -> Option<u64> {
        let fields = [
            ("package", 5, 32),
            ("schema", 5, 32),
            ("binding", 4, self.binding.len()),
            ("mapping-entity", 4, self.mapping_entity.len()),
            ("identity-aspect", 4, self.identity_aspect.len()),
            ("identity-field", 4, self.identity_field.len()),
            ("status-aspect", 4, self.status_aspect.len()),
            ("status-field", 4, self.status_field.len()),
            ("target-relation", 4, self.target_relation.len()),
            ("principal-entity", 4, self.principal_entity.len()),
            (
                "principal-identity-aspect",
                4,
                self.principal_identity_aspect.len(),
            ),
            (
                "principal-identity-field",
                4,
                self.principal_identity_field.len(),
            ),
            (
                "principal-identity-scalar-family",
                4,
                self.principal_identity_scalar_family.canonical_name().len(),
            ),
            (
                "principal-identity-value-type",
                4,
                self.principal_identity_value_type.len(),
            ),
        ];
        let domain = AuthoritySealDomain::InstalledPrincipalBinding;
        let domain_bytes = 8_usize
            .checked_add("domain".len())?
            .checked_add(8)?
            .checked_add(domain.label().len())?;
        let encoded = fields
            .into_iter()
            .try_fold(domain_bytes, |sum, (tag, kind, value)| {
                sum.checked_add(24)?
                    .checked_add(tag.len())?
                    .checked_add(kind)?
                    .checked_add(value)
            })?;
        let encoded = u64::try_from(encoded).ok()?;
        let blocks = encoded.checked_add(64)?.checked_add(9)?.div_ceil(64);
        // HMAC inner/outer pads, final digest, and both SHA-256 final blocks.
        encoded.checked_add(128 + 32 + 128)?.checked_add(blocks)
    }

    /// Bound the actual declared member comparison before it is attempted.
    pub(crate) fn meaning_comparison_work_bound(
        &self,
        member: &ApplicationSchemaMember,
    ) -> Option<u64> {
        let ApplicationSchemaMember::PrincipalBinding {
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
            principal_identity_value_type,
            ..
        } = member
        else {
            return Some(1);
        };
        [
            (&self.binding, binding),
            (&self.mapping_entity, mapping_entity),
            (&self.identity_aspect, identity_aspect),
            (&self.identity_field, identity_field),
            (&self.status_aspect, status_aspect),
            (&self.status_field, status_field),
            (&self.target_relation, target_relation),
            (&self.principal_entity, principal_entity),
            (&self.principal_identity_aspect, principal_identity_aspect),
            (&self.principal_identity_field, principal_identity_field),
            (
                &self.principal_identity_value_type,
                principal_identity_value_type,
            ),
        ]
        .into_iter()
        .try_fold(2_u64, |work, (installed, declared)| {
            work.checked_add(u64::try_from(installed.len().max(declared.len())).ok()?)?
                .checked_add(1)
        })
    }
}
