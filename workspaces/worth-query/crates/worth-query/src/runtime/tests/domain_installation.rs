use super::support::*;
use crate::application::{
    WorthQueryCapabilityFamily, WorthQueryConfigSectionFamily,
    WorthQueryDeclarationEntryContributionCategoryFamily, WorthQueryDomainEntryMarker,
};
use crate::authoring::{
    AspectFieldSelector, DetailQueryBuilder, DetailResultShapeBuilder, RelationName,
    WorthQueryGraphReadDomainOperationDeclaration,
};
use crate::domain_capabilities::{
    admit_eligible_domain_capability_contribution,
    evaluate_requested_domain_capability_contribution, materialize_canonical_admission_artifact,
    prepare_admitted_domain_capability_contribution_for_materialization,
    WorthQueryAdmissionContributionAuthoring, WorthQueryDomainCapabilityProgressionDenialKind,
};
use crate::domain_installation::{
    WorthQueryDomainGraphReadOperationDefinition, WorthQueryDomainHandleDenialKind,
    WorthQueryDomainIdentityDeclaration, WorthQueryDomainIdentityName,
    WorthQueryDomainIdentityNamespace, WorthQueryDomainInstallationDenialKind,
    WorthQueryDomainPackage, WorthQueryDomainRebindDenialKind, WorthQueryDomainRebindNextAction,
    WorthQueryDomainSemanticVersion, WorthQueryInstalledGraphReadOperation,
};
use crate::runtime::{
    WorthQueryIntentDeclaration, WorthQueryIntentInput, WorthQueryReadBuilder,
    WorthQueryReadFamily, WorthQueryRuntime, WorthQueryRuntimeBuilder,
};
use crate::schema_view::{QuerySchemaView, ScalarAspectType, SchemaFieldView, SchemaRelationView};

mod authority;
mod journey;
mod live_lifecycle;
mod lookup_scaling;
mod operation_path_equivalence;
mod rebind;
mod substrates;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct InstalledDomain;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OtherInstalledDomain;

macro_rules! installed_domain_marker {
    ($marker:ty, $display:literal) => {
        impl WorthQueryDomainEntryMarker for $marker {
            fn domain_key(&self) -> &'static str {
                "WORTH.tests.installed-domain"
            }

            fn display_name(&self) -> &'static str {
                $display
            }

            fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
                &[]
            }
        }
    };
}

installed_domain_marker!(InstalledDomain, "InstalledDomain");
installed_domain_marker!(OtherInstalledDomain, "OtherInstalledDomain");

fn identity<D>() -> WorthQueryDomainIdentityDeclaration<D> {
    identity_version(0)
}

fn identity_version<D>(minor: u32) -> WorthQueryDomainIdentityDeclaration<D> {
    WorthQueryDomainIdentityDeclaration::new(
        WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
        WorthQueryDomainIdentityName::new("installed-domain").unwrap(),
        WorthQueryDomainSemanticVersion::new(1, minor),
    )
}

fn package<D: WorthQueryDomainEntryMarker>(marker: D) -> WorthQueryDomainPackage<D> {
    WorthQueryDomainPackage::declare(marker, identity())
        .requires_capability(WorthQueryCapabilityFamily::QueryRead)
        .requires_configuration(WorthQueryConfigSectionFamily::Query)
        .graph_read_operation(
            WorthQueryDomainGraphReadOperationDefinition::new(
                WorthQueryDomainIdentityName::new("neighbors").unwrap(),
                1,
            )
            .accepts_relation(RelationName::new("manager").unwrap()),
        )
        .permits_contribution(WorthQueryDeclarationEntryContributionCategoryFamily::Admission)
}

fn installed_runtime() -> WorthQueryRuntime {
    complete_backend_from_parts_builder()
        .domain_package(package(InstalledDomain))
        .unwrap()
        .build_backend_from_parts()
        .build()
        .unwrap()
}

fn changed_package_runtime() -> WorthQueryRuntime {
    let changed = WorthQueryDomainPackage::declare(InstalledDomain, identity_version(1))
        .requires_capability(WorthQueryCapabilityFamily::QueryRead)
        .requires_configuration(WorthQueryConfigSectionFamily::Query)
        .permits_contribution(WorthQueryDeclarationEntryContributionCategoryFamily::Admission);
    complete_backend_from_parts_builder()
        .domain_package(changed)
        .unwrap()
        .build_backend_from_parts()
        .build()
        .unwrap()
}

#[test]
fn equivalent_packages_mint_semantically_equal_but_runtime_affine_handles() {
    let left = installed_runtime();
    let right = installed_runtime();
    let left_handle = left.domain(InstalledDomain).unwrap();
    let right_handle = right.domain(InstalledDomain).unwrap();

    assert_eq!(
        left_handle.package_identity(),
        right_handle.package_identity()
    );
    assert_ne!(
        left_handle.installation_identity(),
        right_handle.installation_identity()
    );
    assert_eq!(
        right
            .validate_installed_domain_handle(&left_handle)
            .unwrap_err()
            .kind(),
        WorthQueryDomainHandleDenialKind::ForeignRuntime
    );
    left.validate_installed_domain_handle(&left_handle).unwrap();
}

#[test]
fn duplicate_semantic_package_denies_before_a_runtime_can_be_published() {
    let builder = complete_backend_from_parts_builder()
        .domain_package(package(InstalledDomain))
        .unwrap();
    let denial = builder
        .domain_package(package(OtherInstalledDomain))
        .err()
        .expect("equivalent package identity must deny atomically");
    assert_eq!(
        denial
            .installation_denial()
            .expect("equivalent package conflict is an installation denial")
            .kind(),
        WorthQueryDomainInstallationDenialKind::DuplicatePackageIdentity
    );
}

#[test]
fn runtime_installation_reports_package_validation_before_admission() {
    let invalid = WorthQueryDomainPackage::declare(InstalledDomain, identity())
        .graph_read_operation(
            WorthQueryDomainGraphReadOperationDefinition::new(
                WorthQueryDomainIdentityName::new("neighbors").unwrap(),
                1,
            )
            .accepts_relation(RelationName::new("manager").unwrap()),
        )
        .graph_read_operation(
            WorthQueryDomainGraphReadOperationDefinition::new(
                WorthQueryDomainIdentityName::new("neighbors").unwrap(),
                1,
            )
            .accepts_relation(RelationName::new("mentor").unwrap()),
        );

    let error = WorthQueryRuntimeBuilder::new(test_product_world_resources())
        .domain_package(invalid)
        .err()
        .expect("conflicting package meaning must not reach admission");
    assert_eq!(
        error.validation_denial().unwrap().kind(),
        crate::domain_installation::WorthQueryDomainPackageValidationDenialKind::ConflictingGraphReadOperation
    );
    assert!(error.admission_denial().is_none());
    assert!(error.installation_denial().is_none());
}

#[test]
fn runtime_installation_reports_platform_support_admission_before_compilation() {
    let unsupported = WorthQueryDomainPackage::declare(InstalledDomain, identity())
        .requires_capability(WorthQueryCapabilityFamily::DurableArtifacts);

    let error = WorthQueryRuntimeBuilder::new(test_product_world_resources())
        .domain_package(unsupported)
        .err()
        .expect("deferred platform capability must not reach package compilation");
    assert_eq!(
        error.admission_denial().unwrap().kind(),
        crate::domain_installation::WorthQueryDomainPackageAdmissionDenialKind::DeferredCapability
    );
    assert!(error.validation_denial().is_none());
    assert!(error.installation_denial().is_none());
}

#[test]
fn installed_operation_declaration_cannot_cross_runtime_authority() {
    let owner = installed_runtime();
    let foreign = installed_runtime();
    let declaration = installed_operation_family(&owner.domain(InstalledDomain).unwrap());

    owner
        .workspace("installed-operation-owner")
        .unwrap()
        .explain_graph_read_access_shape(&declaration)
        .expect("the installing runtime must resolve its handle-bound operation");
    let denial = foreign
        .workspace("installed-operation-foreign")
        .unwrap()
        .explain_graph_read_access_shape(&declaration)
        .expect_err("an equivalent foreign runtime must not resolve another runtime's declaration");
    assert!(matches!(
        denial,
        crate::runtime::WorthQueryGraphReadAccessShapeExplanationError::
            OperationRequiresAccessCapabilityRegistration(_)
    ));
}

#[test]
fn handle_lookup_is_constant_width_and_installation_is_self_describing() {
    let runtime = installed_runtime();
    let handle = runtime.domain(InstalledDomain).unwrap();
    assert_eq!(handle.domain_key(), "WORTH.tests.installed-domain");
    assert_eq!(handle.display_name(), "InstalledDomain");
    let receipt = runtime
        .domain_installation_receipt(InstalledDomain)
        .unwrap();
    assert_eq!(receipt.construction_counters().package_lowerings(), 1);
    assert_eq!(
        receipt
            .construction_counters()
            .graph_read_operation_index_entries(),
        1
    );
    assert!(receipt.warnings().is_empty());
    assert_eq!(receipt.domain_owner(), "WORTH.tests.installed-domain");
    assert_eq!(receipt.definition_counts().graph_read_operations(), 1);

    let rebuild = runtime.verify_domain_execution_index_rebuild();
    assert!(rebuild.is_equivalent());
    assert_eq!(rebuild.operation_count(), 1);

    runtime.domain(InstalledDomain).unwrap();
    let counters = runtime.domain_installation_lookup_counters();
    assert_eq!(counters.handle_lookups(), 2);
    assert_eq!(counters.package_content_scans(), 0);
}

fn installed_operation_family(
    handle: &crate::domain_installation::WorthQueryInstalledDomainHandle<InstalledDomain>,
) -> WorthQueryReadFamily {
    let operation = handle.graph_read_operation(
        &WorthQueryDomainGraphReadOperationDefinition::new(
            WorthQueryDomainIdentityName::new("neighbors").unwrap(),
            1,
        )
        .accepts_relation(RelationName::new("manager").unwrap()),
    );
    installed_bound_operation_family(operation)
}

fn installed_bound_operation_family(
    operation: WorthQueryInstalledGraphReadOperation,
) -> WorthQueryReadFamily {
    let authored_operation = operation.clone();
    let graph = WorthQueryReadBuilder::standalone()
        .local_detail(
            "user",
            schema(),
            |query: DetailQueryBuilder| {
                authored_operation
                    .author(query.project(AspectFieldSelector::new("identity", "id").unwrap()))
            },
            |shape: DetailResultShapeBuilder| {
                shape.field(
                    crate::authoring::AuthoredResultShapeField::new("identity", "id", "id")
                        .unwrap(),
                )
            },
        )
        .unwrap();
    WorthQueryReadFamily::new_kernel_only("installed-neighbors", operation.bind(graph).unwrap())
}

fn operation_family(
    operation: WorthQueryGraphReadDomainOperationDeclaration,
) -> WorthQueryReadFamily {
    let graph = WorthQueryReadBuilder::standalone()
        .local_detail(
            "user",
            schema(),
            |query: DetailQueryBuilder| {
                query
                    .project(AspectFieldSelector::new("identity", "id").unwrap())
                    .domain_graph_operation(operation)
            },
            |shape: DetailResultShapeBuilder| {
                shape.field(
                    crate::authoring::AuthoredResultShapeField::new("identity", "id", "id")
                        .unwrap(),
                )
            },
        )
        .unwrap();
    WorthQueryReadFamily::new_kernel_only("installed-neighbors", graph)
}

fn schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "installed-domain-schema",
        [SchemaFieldView::new(
            crate::authoring::AspectName::new("identity").unwrap(),
            crate::authoring::FieldName::new("id").unwrap(),
            ScalarAspectType::String,
        )],
        [SchemaRelationView::new(
            RelationName::new("manager").unwrap(),
            1,
        )],
    )
}
