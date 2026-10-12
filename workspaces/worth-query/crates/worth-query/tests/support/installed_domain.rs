use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
    AspectKey, FieldDeclaration, FieldKey, FieldRequirement, ScalarAspectType, StructAspectShape,
};
use worth_query::facade::consumer_kit::{in_memory_test_runtime, WorthQueryTestBackendSchema};
use worth_query::facade::{domain, runtime};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PublicInstalledDomain;

impl domain::WorthQueryDomainEntryMarker for PublicInstalledDomain {
    fn domain_key(&self) -> &'static str {
        "WORTH.tests.public-installed-domain"
    }

    fn display_name(&self) -> &'static str {
        "PublicInstalledDomain"
    }

    fn required_capability_families(&self) -> &'static [domain::WorthQueryCapabilityFamily] {
        &[]
    }
}

pub fn workspace(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    name: &str,
) -> runtime::WorthQueryWorkspace {
    let package = domain::WorthQueryDomainPackage::declare(
        PublicInstalledDomain,
        domain::WorthQueryDomainIdentityDeclaration::new(
            domain::WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
            domain::WorthQueryDomainIdentityName::new("public-installed-domain").unwrap(),
            domain::WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
    .requires_capability(domain::WorthQueryCapabilityFamily::QueryRead)
    .requires_capability(domain::WorthQueryCapabilityFamily::WorkflowOrchestration)
    .requires_configuration(domain::WorthQueryConfigSectionFamily::Query);
    let schema = WorthQueryTestBackendSchema::single_collection("Task")
        .aspect_contract(public_identity_contract())
        .unwrap()
        .aspect("identity.id", "identity.id")
        .unwrap();
    in_memory_test_runtime()
        .with_schema(schema)
        .domain_package(package)
        .workspace(request_execution, name)
        .unwrap()
}

fn public_identity_contract() -> AspectContract {
    let field = FieldDeclaration::new(
        FieldKey::new("id").unwrap(),
        ScalarAspectType::String,
        FieldRequirement::Required,
        AbsenceLaw::Required,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .unwrap();
    AspectContract::struct_aspect(
        AspectKey::new("identity").unwrap(),
        AspectIdentity(0x5751_1901),
        AspectContractRevision(1),
        StructAspectShape::new([field]).unwrap(),
    )
}
