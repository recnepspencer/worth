use super::super::WorthQueryPrimaryGraphLayout;
use super::*;
mod components;
use crate::domain_computation::execution_runtime::WorthQueryExecutionRuntimeInstaller;
use worth_query_declaration::facade::application_schema::*;
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};

struct Schema<const ORBIT: bool>;
use worth_query_declaration::facade::application_schema::fragment;
struct Node<const O: bool = false>;
struct Next<const O: bool = false>;
struct Other<const O: bool>;
struct AggregateRecords<const O: bool>;
impl<const O: bool> fragment::Fragment for AggregateRecords<O> {
    const NAME: &'static str = "aggregate_records";
    type Bindings = fragment::NoBindings;
    type Members = (Node<O>,);
}
impl<const O: bool> fragment::Entity for Node<O> {
    const NAME: &'static str = "Node";
    type Aspects = ();
    type Relations = (Next<O>, Other<O>);
}
impl<const O: bool> fragment::Relation for Next<O> {
    const NAME: &'static str = "Next";
    type To = Node<O>;
    const ORBIT: bool = O;
}
impl<const O: bool> fragment::Relation for Other<O> {
    const NAME: &'static str = "Other";
    type To = Node<O>;
    const ORBIT: bool = O;
}
impl<const ORBIT: bool> ApplicationSchema for Schema<ORBIT> {
    const OWNER: &'static str = "worth.tests.aggregate";
    const NAME: &'static str = "AggregateLowering";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        ApplicationSchemaDeclarationBuilder::<Self>::from_fragment(AggregateRecords::<ORBIT>)?
            .build()
    }
}
fn installed<const ORBIT: bool>() -> (WorthQueryPrimaryGraphLayout, RelationalSchemaRegistry) {
    let declaration = Schema::<ORBIT>::declaration().unwrap();
    install_declaration(declaration)
}
fn install_declaration<S: ApplicationSchema>(
    declaration: ApplicationSchemaDeclaration<S>,
) -> (WorthQueryPrimaryGraphLayout, RelationalSchemaRegistry) {
    let package =
        WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(S::OWNER, 1, 0))
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
        installed.value_bindings(),
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
        .as_ref()
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
                .as_ref()
        })
        .collect();
    assert_eq!(contracts.len(), 2);
    assert_ne!(contracts[0].definition, contracts[1].definition);
    assert_ne!(contracts[0].source, contracts[1].source);
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
        assert!(layout.is_application_relation_kind(contract.source.orbit_relation().unwrap()));
    }
}
