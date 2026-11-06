mod declarations;
mod lowering;
mod merge_policy_declarations;
mod relation_integrity;
mod relation_integrity_lowering;
mod revision;

use crate::execution::RelationalExecutionDenialCause as CompilationDenial;
use crate::schema::SchemaCompilationControl;
use smallvec::SmallVec;

use crate::schema::data::{
    AspectContractPlanCatalog, KindAspectContractDeclarations, LoweredAspectContractBinding,
    LoweredAspectContractPlan, RelationIntegrityPlanCatalog, RelationKindRegistration,
    RelationalSchemaRegistry, SchemaRegistryError,
};

use declarations::{canonicalize_kind_aspect_declarations, RegistrationDomain};
use lowering::lower_binding;
use relation_integrity::canonicalize_relation_integrity_declarations;
use relation_integrity_lowering::lower_relation_integrity_plan;
use revision::derive_plan_revision;

pub(crate) fn canonicalize_entity_registration(
    mut registration: crate::schema::data::EntityKindRegistration,
) -> Result<crate::schema::data::EntityKindRegistration, SchemaRegistryError> {
    registration.aspect_contract_declarations = canonicalize_kind_aspect_declarations(
        registration.kind_id,
        registration.aspect_contract_declarations,
        RegistrationDomain::Entity,
    )?;
    Ok(registration)
}

pub(crate) fn canonicalize_relation_registration(
    mut registration: RelationKindRegistration,
) -> Result<RelationKindRegistration, SchemaRegistryError> {
    registration.aspect_contract_declarations = canonicalize_kind_aspect_declarations(
        registration.kind_id,
        registration.aspect_contract_declarations,
        RegistrationDomain::Relation,
    )?;
    registration.relation_integrity = canonicalize_relation_integrity_declarations(
        registration.kind_id,
        registration.relation_integrity,
    )?;
    Ok(registration)
}

pub(crate) fn lower_aspect_plans(registry: &RelationalSchemaRegistry) -> AspectContractPlanCatalog {
    lower_aspect_plans_for_compilation(registry, SchemaCompilationControl::Configuration)
        .expect("configuration plan lowering has no request refusal")
}
pub(crate) fn lower_aspect_plans_for_compilation(
    registry: &RelationalSchemaRegistry,
    control: SchemaCompilationControl<'_, '_>,
) -> Result<AspectContractPlanCatalog, CompilationDenial> {
    let mut entity_plans = std::collections::BTreeMap::new();
    for (kind, registration) in &registry.entity_kinds {
        control.checkpoint()?;
        entity_plans.insert(
            *kind,
            lower_kind_plan(*kind, &registration.aspect_contract_declarations, control)?,
        );
    }
    let mut relation_plans = std::collections::BTreeMap::new();
    for (kind, registration) in &registry.relation_kinds {
        control.checkpoint()?;
        relation_plans.insert(
            *kind,
            lower_kind_plan(*kind, &registration.aspect_contract_declarations, control)?,
        );
    }
    Ok(AspectContractPlanCatalog {
        entity_plans,
        relation_plans,
    })
}

pub(crate) fn lower_relation_integrity_plans_for_compilation(
    registry: &RelationalSchemaRegistry,
    control: SchemaCompilationControl<'_, '_>,
) -> Result<RelationIntegrityPlanCatalog, CompilationDenial> {
    let mut relation_plans = std::collections::BTreeMap::new();
    for (kind, registration) in &registry.relation_kinds {
        control.checkpoint()?;
        relation_plans.insert(
            *kind,
            lower_relation_integrity_plan(
                *kind,
                &registration.relation_integrity,
                registration.cascade_delete_policy,
                control,
            )?,
        );
    }
    Ok(RelationIntegrityPlanCatalog { relation_plans })
}

fn lower_kind_plan(
    kind_id: crate::identity::data::KindId,
    declarations: &KindAspectContractDeclarations,
    control: SchemaCompilationControl<'_, '_>,
) -> Result<LoweredAspectContractPlan, CompilationDenial> {
    let mut executable_bindings =
        SmallVec::<[LoweredAspectContractBinding; 8]>::with_capacity(declarations.aspects.len());
    for aspect in &declarations.aspects {
        control.checkpoint()?;
        executable_bindings.push(lower_binding(kind_id, declarations.plan_revision, aspect));
    }
    Ok(LoweredAspectContractPlan {
        kind_id,
        plan_revision: declarations.plan_revision,
        executable_bindings,
    })
}

fn derive_kind_plan_revision(
    kind_id: crate::identity::data::KindId,
    aspects: &[crate::schema::data::DeclaredAspectContractBinding],
    identity_declarations: &[crate::merge::data::IdentityBasisDeclaration],
    merge_policy_declarations: &[crate::merge::data::AspectMergePolicyDeclaration],
) -> Result<crate::schema::data::AspectContractPlanRevision, SchemaRegistryError> {
    derive_plan_revision(
        kind_id,
        aspects,
        identity_declarations,
        merge_policy_declarations,
    )
}
