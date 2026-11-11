use super::*;

use crate::admission::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationSupportStatus,
};
use crate::authority_cryptography::InstallationAuthorityRootKey;
use crate::package::{
    WorthQueryPortableDefinition, WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};

impl WorthQueryInstalledPackageIndex {
    pub(crate) fn assert_admitted_declaration_budgets(&self) -> std::collections::BTreeSet<String> {
        let mut families = std::collections::BTreeSet::new();
        for record in self.application_schemas.values() {
            families.extend(
                crate::declaration_budget_tests::assert_declaration_allowance(
                    record.declaration(),
                    record.identity_budget(),
                ),
            );
        }
        families
    }
}

fn admitted(owner: &str, semantics: &str) -> WorthQueryAdmittedPortableDomainPackage {
    admitted_with_slot(owner, format!("{owner}.read"), semantics)
}

fn admitted_with_slot(
    owner: &str,
    slot: impl Into<String>,
    semantics: &str,
) -> WorthQueryAdmittedPortableDomainPackage {
    admitted_with_slot_and_profile(owner, slot, semantics, "support-v1", "config-v1")
}

fn admitted_with_slot_and_profile(
    owner: &str,
    slot: impl Into<String>,
    semantics: &str,
    support_identity: &str,
    configuration_identity: &str,
) -> WorthQueryAdmittedPortableDomainPackage {
    let package =
        WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(owner, 1, 0))
            .requires_capability("query-read")
            .requires_configuration("query")
            .definition(WorthQueryPortableDefinition::graph_read_operation(
                slot, semantics,
            ))
            .validate()
            .unwrap();
    WorthQueryInstallationAdmissionProfile::new(support_identity, configuration_identity)
        .capability("query-read", WorthQueryInstallationSupportStatus::Admitted)
        .configuration("query", true)
        .admit(package)
        .unwrap()
}

#[test]
fn admission_profile_identity_is_part_of_installed_authority() {
    let runtime = WorthQueryInstallationRuntimeIdentity::fresh();
    let generation = WorthQueryInstallationGeneration::initial();
    let build = |runtime, support| {
        WorthQueryInstalledPackageIndex::build(
            runtime,
            generation,
            [admitted_with_slot_and_profile(
                "worth.alpha",
                "worth.alpha.read",
                "direct",
                support,
                "config-v1",
            )],
        )
        .unwrap()
    };
    let left = build(runtime.retained(), "support-v1");
    let right = build(runtime, "support-v2");

    let left_domain = left.domain("worth.alpha").unwrap();
    let right_domain = right.domain("worth.alpha").unwrap();
    let left_operation = left.operation("worth.alpha", "worth.alpha.read").unwrap();
    let right_operation = right.operation("worth.alpha", "worth.alpha.read").unwrap();

    assert_ne!(left.identity(), right.identity());
    assert_ne!(left_domain, right_domain);
    assert_ne!(left_operation, right_operation);
    assert_eq!(
        right.validate(&left_domain).unwrap_err().kind(),
        WorthQueryInstalledPackageIndexDenialKind::AdmissionIdentityChanged
    );
    assert_eq!(
        right
            .validate_operation(&left_operation)
            .unwrap_err()
            .kind(),
        WorthQueryInstalledPackageIndexDenialKind::AdmissionIdentityChanged
    );
}

#[test]
fn authority_entropy_failure_denies_index_construction() {
    let denial = WorthQueryInstalledPackageIndex::build_with_authority_root_result(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted("worth.alpha", "direct")],
        Err(()),
    )
    .unwrap_err();

    assert_eq!(
        denial.kind(),
        WorthQueryInstalledPackageIndexDenialKind::AuthorityEntropyUnavailable
    );
}

#[test]
fn mixed_admission_profiles_cannot_converge_as_equivalent_packages() {
    let denial = WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [
            admitted_with_slot_and_profile(
                "worth.alpha",
                "worth.alpha.read",
                "direct",
                "support-v1",
                "config-v1",
            ),
            admitted_with_slot_and_profile(
                "worth.alpha",
                "worth.alpha.read",
                "direct",
                "support-v2",
                "config-v1",
            ),
        ],
    )
    .unwrap_err();

    assert_eq!(
        denial.kind(),
        WorthQueryInstalledPackageIndexDenialKind::ConflictingAdmissionProfile
    );
}

#[test]
fn declaration_order_and_equivalent_packages_converge() {
    let runtime = WorthQueryInstallationRuntimeIdentity::fresh();
    let generation = WorthQueryInstallationGeneration::initial();
    let left = WorthQueryInstalledPackageIndex::build(
        runtime.retained(),
        generation,
        [
            admitted("worth.alpha", "direct"),
            admitted("worth.beta", "walk"),
        ],
    )
    .unwrap();
    let right = WorthQueryInstalledPackageIndex::build(
        runtime,
        generation,
        [
            admitted("worth.beta", "walk"),
            admitted("worth.alpha", "direct"),
        ],
    )
    .unwrap();
    assert_eq!(left.identity(), right.identity());
    assert_ne!(
        left.domain("worth.alpha").unwrap(),
        right.domain("worth.alpha").unwrap()
    );
    assert_ne!(
        left.operation("worth.alpha", "worth.alpha.read").unwrap(),
        right.operation("worth.alpha", "worth.alpha.read").unwrap()
    );

    let converged = WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        generation,
        [
            admitted("worth.alpha", "direct"),
            admitted("worth.alpha", "direct"),
        ],
    )
    .unwrap();
    assert_eq!(converged.counters().equivalent_packages_converged, 1);
    assert_eq!(converged.counters().installed_package_count, 1);
}

#[test]
fn conflicts_are_atomic_and_dimension_specific() {
    let denial = WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [
            admitted("worth.alpha", "direct"),
            admitted("worth.alpha", "walk"),
        ],
    )
    .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryInstalledPackageIndexDenialKind::ConflictingPackage
    );
}

#[test]
fn foreign_stale_and_rebuilt_authorities_remain_exact() {
    let runtime = WorthQueryInstallationRuntimeIdentity::fresh();
    let generation = WorthQueryInstallationGeneration::initial();
    let index = WorthQueryInstalledPackageIndex::build(
        runtime.retained(),
        generation,
        [admitted("worth.alpha", "direct")],
    )
    .unwrap();
    let authority = index.domain("worth.alpha").unwrap();
    let operation = index.operation("worth.alpha", "worth.alpha.read").unwrap();
    index.validate(&authority).unwrap();
    index.validate_operation(&operation).unwrap();

    let foreign = WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        generation,
        [admitted("worth.alpha", "direct")],
    )
    .unwrap();
    assert_eq!(
        foreign.validate(&authority).unwrap_err().kind(),
        WorthQueryInstalledPackageIndexDenialKind::ForeignRuntime
    );

    let successor = WorthQueryInstalledPackageIndex::build(
        runtime,
        generation.successor(),
        [admitted("worth.alpha", "direct")],
    )
    .unwrap();
    assert_eq!(
        successor.validate(&authority).unwrap_err().kind(),
        WorthQueryInstalledPackageIndexDenialKind::StaleGeneration
    );
    assert!(!successor
        .domain("worth.alpha")
        .unwrap()
        .authority_key
        .matches(&authority.authority_key));

    let rebuilt = index.rebuild();
    assert_eq!(index.identity(), rebuilt.identity());
    assert_eq!(rebuilt.counters().installed_package_count, 1);
    assert_eq!(rebuilt.counters().installed_definition_count, 1);
    assert_eq!(rebuilt.domain("worth.alpha").unwrap(), authority);
    assert_eq!(
        rebuilt
            .operation("worth.alpha", "worth.alpha.read")
            .unwrap(),
        operation
    );
    rebuilt.validate(&authority).unwrap();
    rebuilt.validate_operation(&operation).unwrap();
    assert_eq!(rebuilt.indexed_operation_lookups(), 2);
}

#[test]
fn independently_rooted_same_ordinal_index_is_foreign_authority() {
    let runtime = WorthQueryInstallationRuntimeIdentity::fresh();
    let first = WorthQueryInstalledPackageIndex::build_with_authority_root(
        runtime.retained(),
        WorthQueryInstallationGeneration::initial(),
        [admitted("worth.alpha", "direct")],
        InstallationAuthorityRootKey::repeated_byte_for_test(0x41),
    )
    .unwrap();
    let second = WorthQueryInstalledPackageIndex::build_with_authority_root(
        runtime,
        WorthQueryInstallationGeneration::initial(),
        [admitted("worth.alpha", "direct")],
        InstallationAuthorityRootKey::repeated_byte_for_test(0x42),
    )
    .unwrap();

    assert_eq!(first.identity(), second.identity());
    assert_eq!(
        first.relation_to(&second),
        WorthQueryInstalledPackageIndexRelation::ForeignRuntime
    );
}

#[test]
fn copied_operation_key_cannot_resolve_foreign_package_authority() {
    let runtime = WorthQueryInstallationRuntimeIdentity::fresh();
    let index = WorthQueryInstalledPackageIndex::build(
        runtime,
        WorthQueryInstallationGeneration::initial(),
        [
            admitted_with_slot("worth.alpha", "shared.read", "direct"),
            admitted_with_slot("worth.beta", "shared.read", "direct"),
        ],
    )
    .unwrap();
    let alpha = index.operation("worth.alpha", "shared.read").unwrap();
    let beta = index
        .operation("worth.beta", alpha.operation_slot())
        .unwrap();

    assert_ne!(alpha, beta);
    assert_eq!(alpha.owner(), "worth.alpha");
    assert_eq!(beta.owner(), "worth.beta");
    assert_ne!(alpha.package_identity(), beta.package_identity());
    index.validate_operation(&alpha).unwrap();
    index.validate_operation(&beta).unwrap();
}
