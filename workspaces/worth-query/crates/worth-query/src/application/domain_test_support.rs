use super::{
    WorthQueryDomainEntryMarker, WorthQueryDomainOperatingContext,
    WorthQueryInstalledDomainDeclarationContext,
};
use crate::consumer_kit::{in_memory_test_runtime, WorthQueryTestBackendSchema};
use crate::domain_installation::{
    WorthQueryDomainDeclarationFamilyDefinition, WorthQueryDomainIdentityDeclaration,
    WorthQueryDomainIdentityName, WorthQueryDomainIdentityNamespace, WorthQueryDomainPackage,
    WorthQueryDomainSemanticVersion,
};

/// Builds test declaration authority through the same package/install/lookup path
/// available to external consumers.
pub(crate) fn installed_declaration_context<D, C>(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    marker: D,
    context: C,
    declaration_families: impl IntoIterator<Item = WorthQueryDomainDeclarationFamilyDefinition>,
) -> WorthQueryInstalledDomainDeclarationContext<D, C>
where
    D: WorthQueryDomainEntryMarker + 'static,
    C: WorthQueryDomainOperatingContext<D>,
{
    let (_, context) =
        installed_declaration_workspace(request_execution, marker, context, declaration_families);
    context
}

pub(crate) fn installed_declaration_context_with_contributions<D, C>(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    marker: D,
    context: C,
    declaration_families: impl IntoIterator<Item = WorthQueryDomainDeclarationFamilyDefinition>,
    contribution_categories: impl IntoIterator<
        Item = super::WorthQueryDeclarationEntryContributionCategoryFamily,
    >,
) -> WorthQueryInstalledDomainDeclarationContext<D, C>
where
    D: WorthQueryDomainEntryMarker + 'static,
    C: WorthQueryDomainOperatingContext<D>,
{
    let (_, context) = installed_declaration_workspace_with_contributions(
        request_execution,
        marker,
        context,
        declaration_families,
        contribution_categories,
    );
    context
}

pub(crate) fn installed_declaration_workspace<D, C>(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    marker: D,
    context: C,
    declaration_families: impl IntoIterator<Item = WorthQueryDomainDeclarationFamilyDefinition>,
) -> (
    crate::runtime::WorthQueryWorkspace,
    WorthQueryInstalledDomainDeclarationContext<D, C>,
)
where
    D: WorthQueryDomainEntryMarker + 'static,
    C: WorthQueryDomainOperatingContext<D>,
{
    installed_declaration_workspace_with_contributions(
        request_execution,
        marker,
        context,
        declaration_families,
        [],
    )
}

pub(crate) fn installed_declaration_workspace_with_contributions<D, C>(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    marker: D,
    context: C,
    declaration_families: impl IntoIterator<Item = WorthQueryDomainDeclarationFamilyDefinition>,
    contribution_categories: impl IntoIterator<
        Item = super::WorthQueryDeclarationEntryContributionCategoryFamily,
    >,
) -> (
    crate::runtime::WorthQueryWorkspace,
    WorthQueryInstalledDomainDeclarationContext<D, C>,
)
where
    D: WorthQueryDomainEntryMarker + 'static,
    C: WorthQueryDomainOperatingContext<D>,
{
    let mut package = domain_package(marker);
    for family in context.required_capability_families() {
        package = package.requires_capability(*family);
    }
    for section in context.required_config_sections() {
        package = package.requires_configuration(*section);
    }
    for requirement in context.required_operating_requirements() {
        package = package.requires_operating_posture(*requirement);
    }
    let mut families = declaration_families.into_iter().collect::<Vec<_>>();
    families.sort_by(|left, right| left.family_key().cmp(right.family_key()));
    families.dedup_by(|left, right| left.family_key() == right.family_key());
    package = package.declaration_families(families);
    for category in contribution_categories {
        package = package.permits_contribution(category);
    }

    let schema = WorthQueryTestBackendSchema::single_collection("TestEntity")
        .aspect_contract(test_identity_contract())
        .expect("test identity contract should admit")
        .aspect("identity.id", "identity.id")
        .expect("test declaration schema should be valid");
    let workspace = in_memory_test_runtime()
        .with_schema(schema)
        .domain_package(package)
        .workspace(
            request_execution,
            format!("{}.workspace", marker.domain_key()),
        )
        .expect("test domain package should install");
    let context = workspace
        .domain(marker)
        .expect("installed test domain should be available")
        .declarations_in(&workspace, context)
        .expect("installed declaration context should admit");
    (workspace, context)
}

pub(crate) fn domain_package<D>(marker: D) -> WorthQueryDomainPackage<D>
where
    D: WorthQueryDomainEntryMarker,
{
    let required_capabilities = marker.required_capability_families().to_vec();
    let (namespace, name) = marker
        .domain_key()
        .rsplit_once('.')
        .expect("test domain keys must contain a namespace and name");
    let identity = WorthQueryDomainIdentityDeclaration::new(
        WorthQueryDomainIdentityNamespace::new(namespace)
            .expect("test domain namespace must be valid"),
        WorthQueryDomainIdentityName::new(name).expect("test domain name must be valid"),
        WorthQueryDomainSemanticVersion::new(1, 0),
    );
    let mut package = WorthQueryDomainPackage::declare(marker, identity);
    for capability in required_capabilities {
        package = package.requires_capability(capability);
    }
    package
}

fn test_identity_contract() -> worth_foundational::facade::AspectContract {
    use worth_foundational::facade::{
        AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
        AspectKey, FieldDeclaration, FieldKey, FieldRequirement, ScalarAspectType,
        StructAspectShape,
    };

    let id = FieldDeclaration::new(
        FieldKey::new("id").expect("static identity field must admit"),
        ScalarAspectType::String,
        FieldRequirement::Required,
        AbsenceLaw::Required,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .expect("identity field law must be coherent");
    AspectContract::struct_aspect(
        AspectKey::new("identity").expect("static identity aspect must admit"),
        AspectIdentity(0x5751_1902),
        AspectContractRevision(1),
        StructAspectShape::new([id]).expect("identity fields must be unique"),
    )
}

pub(crate) fn family<D, F>() -> WorthQueryDomainDeclarationFamilyDefinition
where
    D: WorthQueryDomainEntryMarker,
    F: super::WorthQueryDeclarationFamilyMarker<D>,
{
    WorthQueryDomainDeclarationFamilyDefinition::from_marker::<D, F>(1)
        .expect("test declaration family identity must be valid")
}
