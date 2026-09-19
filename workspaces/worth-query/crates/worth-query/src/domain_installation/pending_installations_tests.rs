use worth_relational::facade::identity::KindId;

use super::WorthQueryPendingDomainInstallations;
use super::*;
use crate::application::{
    WorthQueryCapabilityFamily, WorthQueryDeclarationEntryContributionCategoryFamily,
    WorthQueryDeclarationFamilyMarker, WorthQueryDeclarationLegalityContract,
    WorthQueryDomainEntryMarker, WorthQueryNeighborhoodCapableGrouping,
    WorthQueryRelationalTruthAuthority, WorthQuerySignalCompatiblePosture,
};
use crate::authoring::RelationName;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct StableDomain;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FailingDomain;

macro_rules! marker {
    ($marker:ty, $key:literal) => {
        impl WorthQueryDomainEntryMarker for $marker {
            fn domain_key(&self) -> &'static str {
                $key
            }

            fn display_name(&self) -> &'static str {
                $key
            }

            fn required_capability_families(&self) -> &'static [WorthQueryCapabilityFamily] {
                &[]
            }
        }
    };
}

marker!(StableDomain, "WORTH.tests.stable-installation");
marker!(FailingDomain, "WORTH.tests.failing-installation");

struct FailingDeclarationFamily;

impl WorthQueryDeclarationFamilyMarker<FailingDomain> for FailingDeclarationFamily {
    type PrimaryAuthority = WorthQueryRelationalTruthAuthority;
    type SignalCompatibility = WorthQuerySignalCompatiblePosture;
    type GroupedPosture = WorthQueryNeighborhoodCapableGrouping;

    fn semantic_family_key() -> &'static str {
        "failing.installation.family"
    }

    fn legality_contract() -> WorthQueryDeclarationLegalityContract {
        WorthQueryDeclarationLegalityContract::authoritative_hot_artifact()
    }
}

fn admitted<D>(marker: D, name: &str, invariant_major: u32) -> WorthQueryAdmittedDomainPackage<D>
where
    D: WorthQueryDomainEntryMarker,
{
    let invariant = WorthQueryDomainInvariantDefinition::new(
        WorthQueryDomainIdentityName::new("relation-presence").unwrap(),
        WorthQueryDomainSemanticVersion::new(invariant_major, 0),
        WorthQueryDomainInvariantPredicate::requires_outgoing_relations(
            vec![KindId::new(1)],
            vec![KindId::new(2)],
            1,
        ),
        std::num::NonZeroU64::new(4096).unwrap(),
    );
    let package = WorthQueryDomainPackage::declare(
        marker,
        WorthQueryDomainIdentityDeclaration::new(
            WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
            WorthQueryDomainIdentityName::new(name).unwrap(),
            WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
    .invariant(invariant);
    admit_domain_package(package.validate().unwrap()).unwrap()
}

fn fully_populated_failing_package() -> WorthQueryAdmittedDomainPackage<FailingDomain> {
    let package = WorthQueryDomainPackage::declare(
        FailingDomain,
        WorthQueryDomainIdentityDeclaration::new(
            WorthQueryDomainIdentityNamespace::new("WORTH.tests").unwrap(),
            WorthQueryDomainIdentityName::new("failing-installation").unwrap(),
            WorthQueryDomainSemanticVersion::new(1, 0),
        ),
    )
    .invariant(WorthQueryDomainInvariantDefinition::new(
        WorthQueryDomainIdentityName::new("relation-presence").unwrap(),
        WorthQueryDomainSemanticVersion::new(u32::from(u16::MAX) + 1, 0),
        WorthQueryDomainInvariantPredicate::requires_outgoing_relations(
            vec![KindId::new(1)],
            vec![KindId::new(2)],
            1,
        ),
        std::num::NonZeroU64::new(4096).unwrap(),
    ))
    .graph_read_operation(
        WorthQueryDomainGraphReadOperationDefinition::new(
            WorthQueryDomainIdentityName::new("neighbors").unwrap(),
            1,
        )
        .accepts_relation(RelationName::new("required").unwrap()),
    )
    .declaration_family(
        WorthQueryDomainDeclarationFamilyDefinition::from_marker::<
            FailingDomain,
            FailingDeclarationFamily,
        >(1)
        .unwrap(),
    )
    .permits_contribution(WorthQueryDeclarationEntryContributionCategoryFamily::Admission);
    admit_domain_package(package.validate().unwrap()).unwrap()
}

#[test]
fn failed_late_compilation_leaves_every_pending_installation_index_unchanged() {
    let mut pending = WorthQueryPendingDomainInstallations::default();
    pending
        .install(admitted(StableDomain, "stable-installation", 1))
        .unwrap();
    let before = pending.certification_snapshot();

    let denial = pending
        .install(fully_populated_failing_package())
        .expect_err("out-of-range invariant lowering must deny installation");

    assert_eq!(
        denial.kind(),
        WorthQueryDomainInstallationDenialKind::InvariantLoweringFailed
    );
    assert_eq!(pending.certification_snapshot(), before);
}

#[test]
fn compiled_invariant_retains_exact_installing_package_provenance() {
    let mut pending = WorthQueryPendingDomainInstallations::default();
    let package = admitted(StableDomain, "stable-installation", 1);
    let package_identity = package.package_identity.as_str().to_string();
    pending.install(package).unwrap();

    let compiled = pending.take_compiled_substrates();
    let rule_id = compiled.custom_invariants[0].rule_id().as_str();
    assert!(rule_id.contains("WORTH.tests.stable-installation"));
    assert!(rule_id.contains("package-1.0"));
    assert!(rule_id.contains(&package_identity));
}
