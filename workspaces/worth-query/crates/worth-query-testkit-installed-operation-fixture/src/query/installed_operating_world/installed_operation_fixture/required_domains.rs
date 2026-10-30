pub use worth_query::facade::domain;

pub use super::{
    configured_runtime_without_executors, read_vertex_definition, GeometryDomain, ReadFamily,
    ReadVertex, ReadVertexExecutor,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuxiliaryDomain;

impl domain::WorthQueryDomainEntryMarker for AuxiliaryDomain {
    fn domain_key(&self) -> &'static str {
        "WORTH.tests.auxiliary"
    }

    fn display_name(&self) -> &'static str {
        "Auxiliary"
    }

    fn required_capability_families(&self) -> &'static [domain::WorthQueryCapabilityFamily] {
        &[]
    }
}

pub fn required_domain_runtime(
    install_required_domain: bool,
) -> worth_query::facade::consumer_kit::WorthQueryInMemoryTestRuntimeBuilder {
    let base = read_vertex_definition(domain::WorthQuerySupportRequirement::Required);
    let mut semantics = base.semantics().clone();
    semantics
        .required_domains
        .push(domain::WorthQueryOperationRequiredDomainRole::new("auxiliary").unwrap());
    let definition = domain::WorthQueryDomainOperationDefinition::<
        GeometryDomain,
        ReadVertex,
        ReadFamily,
    >::new(base.identity().clone(), semantics);
    let geometry = domain::WorthQueryDomainPackage::declare(
        GeometryDomain,
        domain_identity::<GeometryDomain>("geometry"),
    )
    .operation(definition)
    .operation_required_domain::<ReadVertex, ReadFamily, AuxiliaryDomain>("auxiliary");
    let runtime = configured_runtime_without_executors(geometry).domain_operation_executor(
        GeometryDomain,
        ReadVertex,
        ReadFamily,
        ReadVertexExecutor,
    );
    if install_required_domain {
        runtime.domain_package(domain::WorthQueryDomainPackage::declare(
            AuxiliaryDomain,
            domain_identity::<AuxiliaryDomain>("auxiliary"),
        ))
    } else {
        runtime
    }
}

pub fn domain_identity<D>(name: &str) -> domain::WorthQueryDomainIdentityDeclaration<D> {
    domain::WorthQueryDomainIdentityDeclaration::new(
        domain::WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
        domain::WorthQueryDomainIdentityName::new(name).unwrap(),
        domain::WorthQueryDomainSemanticVersion::new(1, 0),
    )
}
