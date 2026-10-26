use super::super::WorthQueryPrimaryGraphLayout;
use super::*;
use crate::domain_computation::execution_runtime::WorthQueryExecutionRuntimeInstaller;
use worth_query_declaration::facade::application_schema::*;
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};

struct Schema<const ORBIT: bool>;
struct Node;
struct Next;
impl<const ORBIT: bool> ApplicationEntityMarkerIdentity<Schema<ORBIT>> for Node {
    const IDENTIFIER: &'static str = "Node";
}
impl<const ORBIT: bool> ApplicationSchema for Schema<ORBIT> {
    const OWNER: &'static str = "worth.tests.aggregate";
    const NAME: &'static str = "AggregateLowering";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        let node = ApplicationEntityRef::<Self, Node>::from_schema_identifier("Node");
        let base = ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
        let law = if ORBIT { base.orbit() } else { base };
        ApplicationSchemaDeclarationBuilder::<Self>::for_schema()
            .entity(node)
            .relation(
                ApplicationRelationRef::<Self, Next, Node, Node>::from_schema_identifiers(
                    "Next", "Node", "Node", law,
                ),
                node,
                node,
            )
            .relation(
                ApplicationRelationRef::<Self, Next, Node, Node>::from_schema_identifiers(
                    "Other", "Node", "Node", law,
                ),
                node,
                node,
            )
            .build()
    }
}
fn installed<const ORBIT: bool>() -> (WorthQueryPrimaryGraphLayout, RelationalSchemaRegistry) {
    let declaration = Schema::<ORBIT>::declaration().unwrap();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        Schema::<ORBIT>::OWNER,
        1,
        0,
    ))
    .application_schema(declaration.clone())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("aggregate-proof", "local")
        .admit(package)
        .unwrap();
    let (runtime, _) = WorthQueryExecutionRuntimeInstaller::new()
        .install(WorthQueryInstallationGeneration::initial(), [admitted])
        .unwrap()
        .into_parts();
    let installed = runtime
        .installed_packages()
        .bind_application_schema(declaration)
        .unwrap();
    WorthQueryPrimaryGraphLayout::lower(
        installed.installed_declaration(),
        installed.native_contracts(),
        &RelationalSchemaRegistry::new(),
    )
    .unwrap()
}

#[test]
fn actual_orbit_declaration_installs_private_namespaced_native_owners_only_when_enabled() {
    let (_, ordinary_registry) = installed::<false>();
    let (layout, orbit_registry) = installed::<true>();
    let ordinary = ordinary_registry.authority_snapshot();
    let orbit = orbit_registry.authority_snapshot();
    assert!(ordinary.relation_kinds.iter().all(|kind| ordinary_registry
        .relation_registration(kind.kind_id)
        .unwrap()
        .relation_integrity
        .aggregate_identity
        .is_none()));
    let contracts: Vec<_> = orbit
        .relation_kinds
        .iter()
        .filter_map(|kind| {
            orbit_registry
                .relation_registration(kind.kind_id)
                .unwrap()
                .relation_integrity
                .aggregate_identity
        })
        .collect();
    assert_eq!(contracts.len(), 2);
    assert_ne!(contracts[0].definition, contracts[1].definition);
    assert_ne!(contracts[0].source_relation, contracts[1].source_relation);
    assert_eq!(contracts[0].storage, contracts[1].storage);
    assert_eq!(orbit.entity_kinds.len(), ordinary.entity_kinds.len() + 1);
    assert_eq!(
        orbit.relation_kinds.len(),
        ordinary.relation_kinds.len() + 3
    );
    for contract in contracts {
        assert!(!layout.is_application_entity_kind(contract.storage.aggregate_kind));
        assert!(!layout.is_application_relation_kind(contract.storage.lineage_kind));
        assert!(!layout.is_application_relation_kind(contract.membership_kind));
        assert!(layout.is_application_relation_kind(contract.source_relation));
    }
}
