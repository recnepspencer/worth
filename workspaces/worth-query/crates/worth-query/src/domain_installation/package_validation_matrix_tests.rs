use worth_relational::facade::identity::KindId;

use crate::application::{
    WorthQueryCapabilityFamily, WorthQueryConfigSectionFamily,
    WorthQueryDeclarationEntryContributionCategoryFamily, WorthQueryDeclarationFamilyMarker,
    WorthQueryDeclarationLegalityContract, WorthQueryDomainEntryMarker,
    WorthQueryNeighborhoodCapableGrouping, WorthQueryRelationalTruthAuthority,
    WorthQuerySignalCompatiblePosture,
};
use crate::authoring::RelationName;
use crate::runtime::WorthQueryGraphReadTraversalOperator;

use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MatrixDomain;

impl WorthQueryDomainEntryMarker for MatrixDomain {
    fn domain_key(&self) -> &'static str {
        "WORTH.tests.package-matrix"
    }

    fn display_name(&self) -> &'static str {
        "Package Matrix"
    }

    fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
        &[]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CurrentFamily;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NeighborFamily;

macro_rules! declaration_family {
    ($family:ty, $key:literal) => {
        impl WorthQueryDeclarationFamilyMarker<MatrixDomain> for $family {
            type PrimaryAuthority = WorthQueryRelationalTruthAuthority;
            type SignalCompatibility = WorthQuerySignalCompatiblePosture;
            type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

            fn semantic_family_key() -> &'static str {
                $key
            }

            fn legality_contract() -> WorthQueryDeclarationLegalityContract {
                WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
            }
        }
    };
}

declaration_family!(CurrentFamily, "package.current");
declaration_family!(NeighborFamily, "package.neighbor");

fn identity() -> WorthQueryDomainIdentityDeclaration<MatrixDomain> {
    WorthQueryDomainIdentityDeclaration::new(
        WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
        WorthQueryDomainIdentityName::new("package-matrix").unwrap(),
        WorthQueryDomainSemanticVersion::new(1, 0),
    )
}

fn base_package() -> WorthQueryDomainPackage<MatrixDomain> {
    WorthQueryDomainPackage::declare(MatrixDomain, identity())
        .requires_capability(WorthQueryCapabilityFamily::QueryRead)
        .requires_configuration(WorthQueryConfigSectionFamily::Query)
}

fn invariant(name: &str, minor: u32, relevant_kind: u32) -> WorthQueryDomainInvariantDefinition {
    WorthQueryDomainInvariantDefinition::new(
        WorthQueryDomainIdentityName::new(name).unwrap(),
        WorthQueryDomainSemanticVersion::new(1, minor),
        WorthQueryDomainInvariantPredicate::requires_outgoing_relations(
            vec![KindId::new(relevant_kind)],
            vec![KindId::new(100 + relevant_kind)],
            1,
        ),
        std::num::NonZeroU64::new(4096).unwrap(),
    )
}

fn operation(name: &str, relation: &str) -> WorthQueryDomainGraphReadOperationDefinition {
    WorthQueryDomainGraphReadOperationDefinition::new(
        WorthQueryDomainIdentityName::new(name).unwrap(),
        1,
    )
    .accepts_relation(RelationName::new(relation).unwrap())
    .lowers_to(WorthQueryGraphReadTraversalOperator::DeclarationTraversal)
}

fn family<F: WorthQueryDeclarationFamilyMarker<MatrixDomain>>(
    version: u32,
) -> WorthQueryDomainDeclarationFamilyDefinition {
    WorthQueryDomainDeclarationFamilyDefinition::from_marker::<MatrixDomain, F>(version).unwrap()
}

fn full_package(ordering_mask: u8) -> WorthQueryDomainPackage<MatrixDomain> {
    let mut package = base_package();
    let mut invariants = vec![
        invariant("shell-membership", 0, 1),
        invariant("loop-successor", 0, 2),
    ];
    let mut operations = vec![
        operation("incidences", "incident_to"),
        operation("neighbors", "adjacent_to"),
    ];
    let mut families = vec![family::<CurrentFamily>(1), family::<NeighborFamily>(1)];
    let mut contributions = vec![
        WorthQueryDeclarationEntryContributionCategoryFamily::Admission,
        WorthQueryDeclarationEntryContributionCategoryFamily::SupportTraceability,
    ];
    if ordering_mask & 0b00001 != 0 {
        invariants.reverse();
    }
    if ordering_mask & 0b00010 != 0 {
        operations.reverse();
    }
    if ordering_mask & 0b00100 != 0 {
        families.reverse();
    }
    if ordering_mask & 0b01000 != 0 {
        contributions.reverse();
    }
    for definition in invariants {
        package = package.invariant(definition);
    }
    for definition in operations {
        package = package.graph_read_operation(definition);
    }
    package = package.declaration_families(families);
    for category in contributions {
        package = package.permits_contribution(category);
    }
    package
}

fn denial_kind(
    package: WorthQueryDomainPackage<MatrixDomain>,
) -> WorthQueryDomainPackageValidationDenialKind {
    package
        .validate()
        .err()
        .expect("hostile package must not produce a validated successor")
        .kind()
}

#[test]
fn representative_package_orderings_converge() {
    let canonical = full_package(0).validate().unwrap();
    let canonical_identity = canonical.identity().clone();
    let canonical_admitted = admit_domain_package(canonical).unwrap();
    let mut canonical_installation = super::WorthQueryPendingDomainInstallations::default();
    canonical_installation.install(canonical_admitted).unwrap();
    let canonical_installation_snapshot = canonical_installation.certification_snapshot();

    // A fully reversed package and a mixed-axis permutation exercise the two
    // ordering equivalence classes without reconstructing the same installed
    // authority for every bit-mask combination.
    for ordering_mask in [0b1111, 0b1010] {
        let permuted = full_package(ordering_mask).validate().unwrap();
        assert_eq!(
            permuted.identity(),
            &canonical_identity,
            "canonical package identity drifted for independent ordering mask {ordering_mask:04b}"
        );
        assert_eq!(permuted.invariant_count(), 2);
        assert_eq!(permuted.graph_read_operation_count(), 2);
        assert_eq!(permuted.declaration_family_count(), 2);
        assert_eq!(permuted.contribution_category_count(), 2);

        let mut installation = super::WorthQueryPendingDomainInstallations::default();
        installation
            .install(admit_domain_package(permuted).unwrap())
            .unwrap();
        assert_eq!(
            installation.certification_snapshot(),
            canonical_installation_snapshot,
            "compiled installation products drifted for independent ordering mask {ordering_mask:04b}"
        );
    }
}

#[test]
fn duplicate_and_conflicting_invariant_slots_deny() {
    let exact = invariant("loop-successor", 0, 1);
    assert_eq!(
        denial_kind(base_package().invariant(exact.clone()).invariant(exact)),
        WorthQueryDomainPackageValidationDenialKind::DuplicateInvariant
    );
    assert_eq!(
        denial_kind(
            base_package()
                .invariant(invariant("loop-successor", 0, 1))
                .invariant(invariant("loop-successor", 1, 2))
        ),
        WorthQueryDomainPackageValidationDenialKind::ConflictingInvariant
    );
}

#[test]
fn duplicate_operation_family_and_contribution_slots_deny() {
    let exact_operation = operation("neighbors", "adjacent_to");
    assert_eq!(
        denial_kind(
            base_package()
                .graph_read_operation(exact_operation.clone())
                .graph_read_operation(exact_operation)
        ),
        WorthQueryDomainPackageValidationDenialKind::DuplicateGraphReadOperation
    );
    let exact_family = family::<CurrentFamily>(1);
    assert_eq!(
        denial_kind(
            base_package()
                .declaration_family(exact_family.clone())
                .declaration_family(exact_family)
        ),
        WorthQueryDomainPackageValidationDenialKind::DuplicateDeclarationFamily
    );
    assert_eq!(
        denial_kind(
            base_package()
                .declaration_family(family::<CurrentFamily>(1))
                .declaration_family(family::<CurrentFamily>(2))
        ),
        WorthQueryDomainPackageValidationDenialKind::ConflictingDeclarationFamily
    );
    assert_eq!(
        denial_kind(
            base_package()
                .permits_contribution(
                    WorthQueryDeclarationEntryContributionCategoryFamily::Admission,
                )
                .permits_contribution(
                    WorthQueryDeclarationEntryContributionCategoryFamily::Admission,
                )
        ),
        WorthQueryDomainPackageValidationDenialKind::DuplicateContributionCategory
    );
}
