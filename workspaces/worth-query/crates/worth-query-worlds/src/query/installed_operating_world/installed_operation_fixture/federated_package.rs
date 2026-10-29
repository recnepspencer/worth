pub use worth_query::facade::domain;

pub use super::{read_vertex_definition, FederatedRead, GeometryDomain, ReadFamily};

#[derive(Clone, Copy)]
pub enum FederatedOperationContractDrift {
    PreserveLineage,
    UnderstatesExternalCost,
}

pub fn federated_package<G1: 'static, G2: 'static>(
) -> domain::WorthQueryDomainPackage<GeometryDomain> {
    federated_package_with_drift::<G1, G2>(None)
}

pub fn federated_operation_contract_drift_package<G1: 'static, G2: 'static>(
    drift: FederatedOperationContractDrift,
) -> domain::WorthQueryDomainPackage<GeometryDomain> {
    federated_package_with_drift::<G1, G2>(Some(drift))
}

pub fn federated_package_with_drift<G1: 'static, G2: 'static>(
    drift: Option<FederatedOperationContractDrift>,
) -> domain::WorthQueryDomainPackage<GeometryDomain> {
    let mut semantics = read_vertex_definition(domain::WorthQuerySupportRequirement::Required)
        .semantics()
        .clone();
    semantics.graph_reads = domain::WorthQueryOperationGraphReadContract::DeclaredDomain {
        roles: vec![
            domain::WorthQueryDomainOperationGraphReadRole {
                role: "remote-a".into(),
                participation: domain::WorthQueryOperationGraphParticipation::SeparateAuthority {
                    role: "remote-a".into(),
                },
                access: domain::WorthQueryOperationGraphAccess::Project,
                semantic_reads: Vec::new(),
            },
            domain::WorthQueryDomainOperationGraphReadRole {
                role: "remote-b".into(),
                participation: domain::WorthQueryOperationGraphParticipation::SeparateAuthority {
                    role: "remote-b".into(),
                },
                access: domain::WorthQueryOperationGraphAccess::Observe,
                semantic_reads: Vec::new(),
            },
        ],
    };
    semantics.aftermath = None;
    semantics.cost.execution = domain::WorthQueryOperationCostClass::ExternalBoundary;
    match drift {
        Some(FederatedOperationContractDrift::PreserveLineage) => {
            semantics.lineage = domain::WorthQueryOperationLineageContract::Preserve;
        }
        Some(FederatedOperationContractDrift::UnderstatesExternalCost) => {
            semantics.cost.execution = domain::WorthQueryOperationCostClass::DeclaredWidth;
        }
        None => {}
    }
    let operation = domain::WorthQueryDomainOperationDefinition::<
        GeometryDomain,
        FederatedRead,
        ReadFamily,
    >::new(
        domain::WorthQueryDomainOperationIdentity::new("federated-read", 1),
        semantics,
    );
    domain::WorthQueryDomainPackage::declare(
        GeometryDomain,
        domain::WorthQueryDomainIdentityDeclaration::new(
            domain::WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
            domain::WorthQueryDomainIdentityName::new("geometry").unwrap(),
            domain::WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
    .operation(operation)
    .operation_graph_participation::<FederatedRead, ReadFamily, G1>("remote-a")
    .operation_graph_participation::<FederatedRead, ReadFamily, G2>("remote-b")
}
