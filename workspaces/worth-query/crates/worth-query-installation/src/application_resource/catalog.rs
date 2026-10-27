use super::{
    WorthQueryApplicationResourceInstallationDenial as Denial,
    WorthQueryApplicationResourceInstallationDenialKind as Kind,
};
use crate::{
    application_operation::WorthQueryInstalledAbilityRequirement,
    application_query::binding::WorthQueryCompiledApplicationQueryBinding,
    application_schema::WorthQueryInstalledApplicationSchema,
};
use std::{collections::BTreeMap, sync::Arc};
use worth_query_declaration::facade::{
    application_query::ApplicationQueryAuthorizationRequirement,
    application_resource::{
        ApplicationResourceAction, ApplicationResourceBindingDescriptor,
        ErasedApplicationResourceDefinition,
    },
    application_schema::{
        ApplicationSchema, ApplicationSchemaBindingIdentity, ApplicationSchemaMember,
    },
};

pub(crate) struct CompiledApplicationResource {
    pub(super) schema: ApplicationSchemaBindingIdentity,
    pub(super) definition: ErasedApplicationResourceDefinition,
    pub(super) descriptor: ApplicationResourceBindingDescriptor,
    pub(super) access: Arc<WorthQueryCompiledApplicationQueryBinding>,
    pub(super) requirements:
        BTreeMap<ApplicationResourceAction, WorthQueryInstalledAbilityRequirement>,
    pub(super) access_contracts: BTreeMap<
        super::WorthQueryResourceAccessPurpose,
        Arc<super::WorthQueryInstalledResourceAccessContract>,
    >,
}

#[derive(Default)]
pub(crate) struct WorthQueryInstalledApplicationResourceCatalog {
    resources: BTreeMap<String, Arc<CompiledApplicationResource>>,
}

impl WorthQueryInstalledApplicationResourceCatalog {
    pub(super) fn get(&self, identity: &str) -> Option<Arc<CompiledApplicationResource>> {
        self.resources.get(identity).map(Arc::clone)
    }

    pub(crate) fn compile<Schema: ApplicationSchema>(
        schema: &WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<Self, Denial> {
        let mut resources = BTreeMap::new();
        for member in schema.installed_declaration().members() {
            let ApplicationSchemaMember::ApplicationResource { definition } = member else {
                continue;
            };
            let parts = definition.parts();
            definition
                .validate()
                .map_err(|_| Denial::new(Kind::BindingMeaningChanged, &parts.name))?;
            let mut candidates = schema
                .member_provenance
                .resource_bindings()
                .iter()
                .filter(|binding| binding.identity() == parts.name);
            let descriptor = candidates
                .next()
                .ok_or_else(|| Denial::new(Kind::MissingTypedBinding, &parts.name))?;
            if candidates.next().is_some() || !descriptor.matches_definition(definition) {
                return Err(Denial::new(Kind::BindingMeaningChanged, &parts.name));
            }
            validate_value_profiles(definition)?;
            let access = schema
                .query_catalog
                .get_binding(descriptor.access().identity())
                .filter(|binding| binding.descriptor() == descriptor.access())
                .ok_or_else(|| Denial::new(Kind::AccessBindingMismatch, &parts.name))?;
            let mut requirements = BTreeMap::new();
            for action in ApplicationResourceAction::ALL {
                if let ApplicationQueryAuthorizationRequirement::Ability {
                    ability,
                    scope_entity,
                } = parts.authorization.requirement(action)
                {
                    let requirement = schema
                        .installed_ability_requirement(ability, scope_entity)
                        .ok_or_else(|| {
                            Denial::new(Kind::MissingActionAuthorization, &parts.name)
                        })?;
                    requirements.insert(action, requirement.clone());
                }
            }
            let access_contracts = super::access_compilation::compile_access_contracts(
                schema,
                definition,
                &requirements,
            )?;
            let compiled = CompiledApplicationResource {
                schema: schema.binding_identity(),
                definition: definition.clone(),
                descriptor: descriptor.clone(),
                access,
                requirements,
                access_contracts,
            };
            if resources
                .insert(parts.name.clone(), Arc::new(compiled))
                .is_some()
            {
                return Err(Denial::new(Kind::BindingMeaningChanged, &parts.name));
            }
        }
        if schema
            .member_provenance
            .resource_bindings()
            .iter()
            .any(|binding| !resources.contains_key(binding.identity()))
        {
            return Err(Denial::new(
                Kind::BindingMeaningChanged,
                schema.schema_name(),
            ));
        }
        Ok(Self { resources })
    }
}

fn validate_value_profiles(definition: &ErasedApplicationResourceDefinition) -> Result<(), Denial> {
    let parts = definition.parts();
    for (value, maximum) in [
        (&parts.input, parts.limits.maximum_input_bytes),
        (&parts.result, parts.limits.maximum_frame_bytes),
        (&parts.failure, parts.limits.maximum_frame_bytes),
    ] {
        if !value.has_supported_profile()
            || value
                .parts()
                .native_input_admission
                .is_some_and(|profile| profile.requirements(maximum).is_none())
        {
            return Err(Denial::new(Kind::UnsupportedValueProfile, &parts.name));
        }
    }
    let limits = parts.limits;
    if !limits.is_valid()
        || usize::try_from(limits.maximum_input_bytes).is_err()
        || usize::try_from(limits.maximum_frame_bytes).is_err()
    {
        return Err(Denial::new(Kind::InvalidLimits, &parts.name));
    }
    Ok(())
}
