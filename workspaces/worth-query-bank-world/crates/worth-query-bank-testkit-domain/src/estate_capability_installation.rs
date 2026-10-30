pub use std::collections::BTreeSet;

#[path = "estate_capability_installation/composition_contracts.rs"]
pub mod composition_contracts;
#[path = "estate_capability_installation/delegation_activation.rs"]
pub mod delegation_activation;
#[path = "estate_capability_installation/disburse_estate.rs"]
pub mod disburse_estate;
#[path = "estate_capability_installation/emergency_access_activity.rs"]
pub mod emergency_access_activity;
#[path = "estate_capability_installation/graph_authority_source.rs"]
pub mod graph_authority_source;
#[path = "estate_capability_installation/installed_contract_views.rs"]
pub mod installed_contract_views;
#[path = "estate_capability_installation/release_estate.rs"]
pub mod release_estate;
#[path = "estate_capability_installation/transition_dimensions.rs"]
pub mod transition_dimensions;

pub use bank_domain::schema::*;
pub use installed_contract_views::{
    installed_program_targets, installed_read_targets, InstalledProgramTarget, InstalledReadTarget,
};
pub use worth_query_host::facade::domain::{
    WorthQueryApplicationCapabilityInstallationDenialKind, WorthQueryInstallationAdmissionProfile,
    WorthQueryInstallationGeneration, WorthQueryInstallationRuntimeIdentity,
    WorthQueryInstalledApplicationSchema, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};

#[macro_export]
macro_rules! worlds_domain_estate_capability_installation_rs_assert_installed {
    ($bank:expr, $identities:expr, $capability:ident, $operation:ident) => {{
        let capability = $bank
            .capability($capability::reference(), $operation::reference())
            .unwrap();
        $bank.validate_installed_capability(&capability).unwrap();
        assert_eq!(capability.canonical_basis().basis_preparation_count(), 1);
        assert_eq!(capability.canonical_basis().digest_derivation_count(), 1);
        assert!(
            capability.canonical_basis().canonical_entry_count()
                <= capability.canonical_basis().maximum_canonical_entry_count()
        );
        assert!(capability.canonical_basis().canonical_encoded_bytes() > 0);
        assert!(
            capability.canonical_basis().canonical_encoded_bytes()
                <= capability
                    .canonical_basis()
                    .maximum_canonical_encoded_bytes()
        );
        let lookup = capability.lookup_evidence();
        assert_eq!(lookup.registry_probes(), 1);
        assert_eq!(lookup.basis_preparations(), 0);
        assert_eq!(lookup.digest_derivations(), 0);
        assert_eq!(lookup.digest_text_materializations(), 0);
        assert!($identities.insert(capability.identity().clone()));
    }};
}
pub use crate::worlds_domain_estate_capability_installation_rs_assert_installed as assert_installed;

pub fn installed_bank(
    runtime: WorthQueryInstallationRuntimeIdentity,
) -> (
    WorthQueryInstalledPackageIndex,
    WorthQueryInstalledApplicationSchema<BankSchema>,
) {
    installed_bank_at(runtime, WorthQueryInstallationGeneration::initial())
}

pub fn installed_bank_at(
    runtime: WorthQueryInstallationRuntimeIdentity,
    generation: WorthQueryInstallationGeneration,
) -> (
    WorthQueryInstalledPackageIndex,
    WorthQueryInstalledApplicationSchema<BankSchema>,
) {
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "WORTH.bank",
        1,
        0,
    ))
    .application_schema(BankSchema::declaration().unwrap())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support-v1", "config-v1")
        .admit(package)
        .unwrap();
    let index = WorthQueryInstalledPackageIndex::build(runtime, generation, [admitted]).unwrap();
    let bank = index
        .bind_application_schema(BankSchema::declaration().unwrap())
        .unwrap();
    (index, bank)
}
