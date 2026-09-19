use worth_relational::facade::identity::KindId;
use worth_relational::facade::runtime::RelationalRuntimeApi;

use super::*;
use crate::application::{
    WorthQueryDeclarationFamilyMarker, WorthQueryDeclarationLegalityContract,
    WorthQueryNeighborhoodCapableGrouping, WorthQueryRelationalTruthAuthority,
    WorthQuerySignalCompatiblePosture,
};
use crate::domain_installation::{
    WorthQueryDomainDeclarationFamilyDefinition, WorthQueryDomainInvariantDefinition,
    WorthQueryDomainInvariantPredicate,
};
use crate::runtime::WorthQueryGraphReadOperationLookup;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct InstalledReadFamily;

impl WorthQueryDeclarationFamilyMarker<InstalledDomain> for InstalledReadFamily {
    type PrimaryAuthority = WorthQueryRelationalTruthAuthority;
    type SignalCompatibility = WorthQuerySignalCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "installed.read"
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }
}

fn full_substrate_package() -> WorthQueryDomainPackage<InstalledDomain> {
    package(InstalledDomain)
        .invariant(WorthQueryDomainInvariantDefinition::new(
            WorthQueryDomainIdentityName::new("outgoing-manager").unwrap(),
            WorthQueryDomainSemanticVersion::new(1, 0),
            WorthQueryDomainInvariantPredicate::requires_outgoing_relations(
                vec![KindId::new(900)],
                vec![KindId::new(901)],
                1,
            ),
            std::num::NonZeroU64::new(4096).unwrap(),
        ))
        .declaration_family(
            WorthQueryDomainDeclarationFamilyDefinition::from_marker::<
                InstalledDomain,
                InstalledReadFamily,
            >(3)
            .unwrap(),
        )
        .permits_contribution(
            WorthQueryDeclarationEntryContributionCategoryFamily::SupportTraceability,
        )
}

#[test]
fn installed_package_compiles_every_semantic_family_before_runtime_publication() {
    let runtime = complete_query_owned_backend_from_parts_builder()
        .domain_package(full_substrate_package())
        .unwrap()
        .build_backend_from_parts()
        .build()
        .unwrap();
    let receipt = runtime
        .domain_installation_receipt(InstalledDomain)
        .expect("installed package must publish one receipt");
    let counters = receipt.construction_counters();
    assert_eq!(counters.package_lowerings(), 1);
    assert_eq!(counters.invariant_index_entries(), 1);
    assert_eq!(counters.graph_read_operation_index_entries(), 1);
    assert_eq!(counters.declaration_family_index_entries(), 1);
    assert_eq!(counters.contribution_policy_index_entries(), 2);
    let handle = runtime.domain(InstalledDomain).unwrap();
    assert_eq!(
        handle
            .authority()
            .declaration_family_version(InstalledReadFamily::semantic_family_key()),
        Some(3)
    );
    assert!(handle
        .authority()
        .contribution_policy()
        .contains(&WorthQueryDeclarationEntryContributionCategoryFamily::SupportTraceability));
    let operation = handle.graph_read_operation(
        &WorthQueryDomainGraphReadOperationDefinition::new(
            WorthQueryDomainIdentityName::new("neighbors").unwrap(),
            1,
        )
        .accepts_relation(RelationName::new("manager").unwrap()),
    );
    let operation_provenance = runtime
        .installed_domain_execution_index()
        .matching_declared_operation(operation.declaration(), Some(operation.authority()))
        .and_then(|registration| registration.admitted().installed_provenance().cloned())
        .expect("package operation must retain installed provenance");
    assert_eq!(
        operation_provenance.domain_owner(),
        "WORTH.tests.installed-domain"
    );
    assert_eq!(operation_provenance.package_version(), "1.0");
    assert_eq!(
        operation_provenance.package_identity(),
        handle.package_identity().as_str()
    );

    let rebuild = runtime.verify_domain_execution_index_rebuild();
    assert!(rebuild.is_equivalent());
    assert_eq!(rebuild.invariant_count(), 1);
    assert_eq!(rebuild.operation_count(), 1);
    assert_eq!(rebuild.declaration_family_count(), 1);
    assert_eq!(rebuild.contribution_policy_count(), 2);
}

#[test]
fn installed_invariants_cannot_be_bypassed_with_an_explicit_relational_runtime() {
    let result = WorthQueryRuntime::builder(test_product_world_resources())
        .domain_package(full_substrate_package())
        .unwrap()
        .relational_runtime(RelationalRuntimeApi::builder().build())
        .build_backend_from_parts()
        .build();
    let error = result
        .err()
        .expect("a package-compiled invariant must conflict with a separately supplied runtime");
    match error {
        crate::runtime::WorthQueryRuntimeError::InvariantRegistration { stage, message } => {
            assert_eq!(stage, "relational_runtime_authority_selection");
            assert!(message.contains("choose one authority path"));
        }
        other => panic!("unexpected installed-substrate error: {other:?}"),
    }
}

#[test]
fn rebuilt_execution_index_reproduces_resolution_denial_and_diagnostic_identity() {
    let mut runtime = complete_query_owned_backend_from_parts_builder()
        .domain_package(full_substrate_package())
        .unwrap()
        .build_backend_from_parts()
        .build()
        .unwrap();
    let handle = runtime.domain(InstalledDomain).unwrap();
    let authority = WorthQueryGraphReadAccessAuthorityContext::runtime_current_compatibility();
    let family = installed_operation_family(&handle);

    let resolution_before_destruction =
        crate::runtime::explain_graph_read_access_shape_for_family_in_authority_with_lookup(
            &family,
            &authority,
            runtime.installed_domain_execution_index(),
        )
        .unwrap();
    let missing = WorthQueryDomainGraphReadOperationDefinition::new(
        WorthQueryDomainIdentityName::new("missing-neighbors").unwrap(),
        1,
    )
    .accepts_relation(RelationName::new("manager").unwrap())
    .requires_support_family(
        WorthQueryDomainIdentityNamespace::new("WORTH.tests.missing-neighbors").unwrap(),
    );
    let owner = handle
        .authority_witness()
        .authority()
        .domain_owner()
        .to_string();
    let mut missing_declaration =
        crate::authoring::WorthQueryGraphReadDomainOperationDeclaration::new(
            missing.name().as_str(),
            missing.version(),
            owner,
        )
        .unwrap()
        .admit_relation_reference("manager")
        .unwrap();
    {
        let support_family = "WORTH.tests.missing-neighbors";
        missing_declaration = missing_declaration
            .requires_support_family(support_family)
            .unwrap();
    }
    let missing_family = operation_family(missing_declaration);
    let denial_before_destruction =
        crate::runtime::explain_graph_read_access_shape_for_family_in_authority_with_lookup(
            &missing_family,
            &authority,
            runtime.installed_domain_execution_index(),
        )
        .unwrap_err();
    let denial_identity_before_destruction = match &denial_before_destruction {
        WorthQueryGraphReadAccessShapeExplanationError::OperationRequiresAccessCapabilityRegistration(
            denial,
        ) => denial.digest_part(),
        other => panic!("unexpected pre-rebuild denial: {other:?}"),
    };

    let rebuild = runtime.destroy_and_rebuild_domain_execution_index();
    assert!(rebuild.is_equivalent());

    let resolution_after_rebuild =
        crate::runtime::explain_graph_read_access_shape_for_family_in_authority_with_lookup(
            &family,
            &authority,
            runtime.installed_domain_execution_index(),
        )
        .unwrap();
    assert_eq!(resolution_before_destruction, resolution_after_rebuild);
    assert_eq!(
        resolution_before_destruction.explain(),
        resolution_after_rebuild.explain()
    );
    assert_eq!(
        resolution_before_destruction.access_shape().digest(),
        resolution_after_rebuild.access_shape().digest()
    );

    let denial_after_rebuild =
        crate::runtime::explain_graph_read_access_shape_for_family_in_authority_with_lookup(
            &missing_family,
            &authority,
            runtime.installed_domain_execution_index(),
        )
        .unwrap_err();
    assert_eq!(denial_before_destruction, denial_after_rebuild);
    let WorthQueryGraphReadAccessShapeExplanationError::OperationRequiresAccessCapabilityRegistration(
        denial_after_rebuild,
    ) = denial_after_rebuild
    else {
        panic!("rebuilt index changed the denial family")
    };
    assert_eq!(
        denial_identity_before_destruction,
        denial_after_rebuild.digest_part()
    );
}
