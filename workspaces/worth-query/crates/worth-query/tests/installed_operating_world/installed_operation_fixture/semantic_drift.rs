use worth_query::facade::{domain, runtime};
use worth_relational::facade::identity::KindId;

use super::executors::ReadVertexExecutor;
use super::{
    configured_runtime_without_executors, read_vertex_definition, GeometryDomain, ReadFamily,
    ReadVertex,
};

pub fn semantic_drift_workspace(
    name: &str,
    mutate: fn(&mut domain::WorthQueryDomainOperationSemanticClosure),
) -> Result<
    runtime::WorthQueryWorkspace,
    worth_query::facade::consumer_kit::WorthQueryTestBackendError,
> {
    let original = read_vertex_definition(domain::WorthQuerySupportRequirement::Required);
    let mut drifted_semantics = original.semantics().clone();
    mutate(&mut drifted_semantics);
    let drifted = domain::WorthQueryDomainOperationDefinition::<
        GeometryDomain,
        ReadVertex,
        ReadFamily,
    >::new(original.identity().clone(), drifted_semantics);
    let package = domain::WorthQueryDomainPackage::declare(
        GeometryDomain,
        domain::WorthQueryDomainIdentityDeclaration::new(
            domain::WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
            domain::WorthQueryDomainIdentityName::new("geometry").unwrap(),
            domain::WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
    .requires_capability(domain::WorthQueryCapabilityFamily::QueryRead)
    .invariant(domain::WorthQueryDomainInvariantDefinition::new(
        domain::WorthQueryDomainIdentityName::new("semantic-invariant").unwrap(),
        domain::WorthQueryDomainSemanticVersion::new(1, 0),
        domain::WorthQueryDomainInvariantPredicate::requires_outgoing_relations(
            vec![KindId::new(0xff00_0011)],
            vec![KindId::new(0xff00_0012)],
            1,
        ),
        std::num::NonZeroU64::new(4096).unwrap(),
    ))
    .operation(original)
    .operation(drifted);
    configured_runtime_without_executors(package)
        .domain_operation_executor(GeometryDomain, ReadVertex, ReadFamily, ReadVertexExecutor)
        .workspace(name)
}
