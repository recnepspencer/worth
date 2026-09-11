use std::marker::PhantomData;

use worth_foundational::facade::ScalarAspectType;
use worth_query_declaration::facade::application_schema::{
    ApplicationIdentityScalarValueBinding, ApplicationSchema, ApplicationSchemaBindingIdentity,
    ApplicationSchemaMember, ApplicationValueDecodeDenial,
};

use crate::application_schema::{
    WorthQueryInstalledApplicationSchema, WorthQueryInstalledApplicationValueBinding,
};
use crate::authority_cryptography::{
    AuthoritySeal, AuthoritySealDomain, AuthorityTranscript, PackageAuthorityKey,
};
use crate::installed_index::WorthQueryInstalledPackageAuthority;

/// Opaque installation authority for one schema-declared principal binding.
///
/// The type is intentionally neither cloneable nor serializable. Its
/// constructor consumes meaning retained only by an installed schema handle.
pub struct WorthQueryInstalledPrincipalBinding<
    Schema,
    Binding,
    Mapping,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityBinding,
> {
    binding_identity: ApplicationSchemaBindingIdentity,
    owner: String,
    schema_name: String,
    binding: String,
    mapping_entity: String,
    identity_aspect: String,
    identity_field: String,
    status_aspect: String,
    status_field: String,
    target_relation: String,
    principal_entity: String,
    principal_identity_aspect: String,
    principal_identity_field: String,
    principal_identity_scalar_family: ScalarAspectType,
    principal_identity_value_type: String,
    principal_identity_binding: WorthQueryInstalledApplicationValueBinding,
    authority_identity: AuthoritySeal,
    _marker: PhantomData<
        fn() -> (
            Schema,
            Binding,
            Mapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        ),
    >,
}

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
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_installed_schema(
        schema: &WorthQueryInstalledApplicationSchema<Schema>,
        binding: &str,
        mapping_entity: &str,
        identity_aspect: &str,
        identity_field: &str,
        status_aspect: &str,
        status_field: &str,
        target_relation: &str,
        principal_entity: &str,
        principal_identity_aspect: &str,
        principal_identity_field: &str,
        principal_identity_scalar_family: ScalarAspectType,
        principal_identity_value_type: &str,
        principal_identity_binding: WorthQueryInstalledApplicationValueBinding,
    ) -> Self
    where
        Schema: ApplicationSchema,
    {
        let binding_identity = schema.binding_identity();
        let authority_identity = authority_identity(
            &schema.package_authority.authority_key,
            &binding_identity,
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
        );
        Self {
            binding_identity,
            owner: schema.owner().to_string(),
            schema_name: schema.schema_name().to_string(),
            binding: binding.to_string(),
            mapping_entity: mapping_entity.to_string(),
            identity_aspect: identity_aspect.to_string(),
            identity_field: identity_field.to_string(),
            status_aspect: status_aspect.to_string(),
            status_field: status_field.to_string(),
            target_relation: target_relation.to_string(),
            principal_entity: principal_entity.to_string(),
            principal_identity_aspect: principal_identity_aspect.to_string(),
            principal_identity_field: principal_identity_field.to_string(),
            principal_identity_scalar_family,
            principal_identity_value_type: principal_identity_value_type.to_string(),
            principal_identity_binding,
            authority_identity,
            _marker: PhantomData,
        }
    }

    pub fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding_identity
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }

    pub fn binding(&self) -> &str {
        &self.binding
    }

    pub fn mapping_entity(&self) -> &str {
        &self.mapping_entity
    }

    pub fn identity_aspect(&self) -> &str {
        &self.identity_aspect
    }

    pub fn identity_field(&self) -> &str {
        &self.identity_field
    }

    pub fn status_aspect(&self) -> &str {
        &self.status_aspect
    }

    pub fn status_field(&self) -> &str {
        &self.status_field
    }

    pub fn target_relation(&self) -> &str {
        &self.target_relation
    }

    pub fn principal_entity(&self) -> &str {
        &self.principal_entity
    }

    pub fn principal_identity_aspect(&self) -> &str {
        &self.principal_identity_aspect
    }

    pub fn principal_identity_field(&self) -> &str {
        &self.principal_identity_field
    }

    pub const fn principal_identity_scalar_family(&self) -> ScalarAspectType {
        self.principal_identity_scalar_family
    }

    pub fn principal_identity_value_type(&self) -> &str {
        &self.principal_identity_value_type
    }

    pub fn principal_identity_binding(&self) -> &WorthQueryInstalledApplicationValueBinding {
        &self.principal_identity_binding
    }

    pub fn decode_principal_identity(
        &self,
        value: &worth_foundational::facade::AspectValue,
    ) -> Result<PrincipalIdentity, ApplicationValueDecodeDenial>
    where
        PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
        PrincipalIdentity: 'static,
    {
        self.principal_identity_binding.decode(value)
    }

    pub fn authority_identity(&self) -> &str {
        self.authority_identity.as_str()
    }

    pub(crate) fn meaning_matches(&self, member: &ApplicationSchemaMember) -> bool {
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
            } if binding == &self.binding
                && mapping_entity == &self.mapping_entity
                && identity_aspect == &self.identity_aspect
                && identity_field == &self.identity_field
                && status_aspect == &self.status_aspect
                && status_field == &self.status_field
                && target_relation == &self.target_relation
                && principal_entity == &self.principal_entity
                && principal_identity_aspect == &self.principal_identity_aspect
                && principal_identity_field == &self.principal_identity_field
                && principal_identity_scalar_family == &self.principal_identity_scalar_family
                && principal_identity_value_type == &self.principal_identity_value_type
        )
    }

    pub(crate) fn authority_matches(&self, package: &WorthQueryInstalledPackageAuthority) -> bool {
        authority_transcript(
            &package.authority_key,
            &self.binding_identity,
            PrincipalBindingAuthorityMeaning {
                binding: &self.binding,
                mapping_entity: &self.mapping_entity,
                identity_aspect: &self.identity_aspect,
                identity_field: &self.identity_field,
                status_aspect: &self.status_aspect,
                status_field: &self.status_field,
                target_relation: &self.target_relation,
                principal_entity: &self.principal_entity,
                principal_identity_aspect: &self.principal_identity_aspect,
                principal_identity_field: &self.principal_identity_field,
                principal_identity_scalar_family: self.principal_identity_scalar_family,
                principal_identity_value_type: &self.principal_identity_value_type,
            },
        )
        .verifies(&self.authority_identity)
    }
}

impl<Schema, Binding, Mapping, Principal, PrincipalIdentity, PrincipalIdentityBinding>
    std::fmt::Debug
    for WorthQueryInstalledPrincipalBinding<
        Schema,
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryInstalledPrincipalBinding")
            .field("binding", &self.binding)
            .field("binding_identity", &self.binding_identity)
            .field("authority_identity", &self.authority_identity)
            .finish_non_exhaustive()
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn authority_identity(
    key: &PackageAuthorityKey,
    identity: &ApplicationSchemaBindingIdentity,
    binding: &str,
    mapping_entity: &str,
    identity_aspect: &str,
    identity_field: &str,
    status_aspect: &str,
    status_field: &str,
    target_relation: &str,
    principal_entity: &str,
    principal_identity_aspect: &str,
    principal_identity_field: &str,
    principal_identity_scalar_family: ScalarAspectType,
    principal_identity_value_type: &str,
) -> AuthoritySeal {
    authority_transcript(
        key,
        identity,
        PrincipalBindingAuthorityMeaning {
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
        },
    )
    .finish()
}

struct PrincipalBindingAuthorityMeaning<'a> {
    binding: &'a str,
    mapping_entity: &'a str,
    identity_aspect: &'a str,
    identity_field: &'a str,
    status_aspect: &'a str,
    status_field: &'a str,
    target_relation: &'a str,
    principal_entity: &'a str,
    principal_identity_aspect: &'a str,
    principal_identity_field: &'a str,
    principal_identity_scalar_family: ScalarAspectType,
    principal_identity_value_type: &'a str,
}

fn authority_transcript(
    key: &PackageAuthorityKey,
    identity: &ApplicationSchemaBindingIdentity,
    meaning: PrincipalBindingAuthorityMeaning<'_>,
) -> AuthorityTranscript {
    let mut transcript =
        AuthorityTranscript::new(key, AuthoritySealDomain::InstalledPrincipalBinding);
    transcript.bytes("package", identity.package_identity().bytes());
    transcript.bytes("schema", identity.schema_identity().bytes());
    transcript.text("binding", meaning.binding);
    transcript.text("mapping-entity", meaning.mapping_entity);
    transcript.text("identity-aspect", meaning.identity_aspect);
    transcript.text("identity-field", meaning.identity_field);
    transcript.text("status-aspect", meaning.status_aspect);
    transcript.text("status-field", meaning.status_field);
    transcript.text("target-relation", meaning.target_relation);
    transcript.text("principal-entity", meaning.principal_entity);
    transcript.text(
        "principal-identity-aspect",
        meaning.principal_identity_aspect,
    );
    transcript.text("principal-identity-field", meaning.principal_identity_field);
    transcript.text(
        "principal-identity-scalar-family",
        meaning.principal_identity_scalar_family.canonical_name(),
    );
    transcript.text(
        "principal-identity-value-type",
        meaning.principal_identity_value_type,
    );
    transcript
}
