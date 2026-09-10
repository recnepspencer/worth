use std::sync::Arc;

use worth_query_installation::facade::WorthQueryInstallationGeneration;

use super::{
    WorthQueryApplicationQueryResourceProfile, WorthQueryExecutionInstallationCommitDenial,
    WorthQueryExecutionRuntimeInstaller,
};
use crate::domain_computation::operation_binding::WorthQueryInstalledDomainExecutionAuthority;

fn empty_runtime() -> super::WorthQueryExecutionRuntime {
    WorthQueryExecutionRuntimeInstaller::new()
        .install(
            WorthQueryInstallationGeneration::initial(),
            std::iter::empty(),
        )
        .unwrap()
        .into_parts()
        .0
}

#[test]
fn each_installer_mints_one_distinct_runtime_authority() {
    let first = WorthQueryExecutionRuntimeInstaller::new();
    let second = WorthQueryExecutionRuntimeInstaller::new();

    assert_ne!(first.authority_identity(), second.authority_identity());
    assert_ne!(
        first.installation_runtime().ordinal(),
        second.installation_runtime().ordinal()
    );
}

#[test]
fn runtime_retains_the_exact_installed_index_owner() {
    let runtime = empty_runtime();
    let first_retained = runtime.retain_installed_packages();
    let second_retained = runtime.retain_installed_packages();

    assert!(Arc::ptr_eq(&first_retained, &second_retained));
    assert_eq!(
        runtime.installed_packages().identity(),
        first_retained.identity()
    );
    assert_eq!(runtime.installed_packages().installed_definition_count(), 0);
}

#[test]
fn runtime_retains_the_installer_owned_query_resource_profile() {
    let resources =
        WorthQueryApplicationQueryResourceProfile::bounded(12_000, 3_000, 400, 64).unwrap();
    let runtime = WorthQueryExecutionRuntimeInstaller::new()
        .application_query_resources(resources)
        .install(
            WorthQueryInstallationGeneration::initial(),
            std::iter::empty(),
        )
        .unwrap()
        .into_parts()
        .0;

    assert_eq!(runtime.application_query_resource_profile(), resources);
}

#[test]
fn deterministic_rebuild_replaces_storage_without_changing_identity() {
    let mut runtime = empty_runtime();
    let rebuilt = Arc::new(runtime.installed_packages().rebuild());

    runtime
        .replace_rebuilt_installation(Arc::clone(&rebuilt))
        .unwrap();

    assert!(Arc::ptr_eq(&runtime.retain_installed_packages(), &rebuilt));
}

#[test]
fn successor_commit_requires_the_current_owner_and_a_new_identity() {
    let mut runtime = empty_runtime();
    let same_generation = Arc::new(runtime.installed_packages().rebuild());

    assert_eq!(
        runtime.commit_successor_installation(same_generation),
        Err(WorthQueryExecutionInstallationCommitDenial::ExactSuccessorRequired)
    );

    let successor = Arc::new(runtime.installed_packages().successor_generation());
    let successor_identity = successor.identity().clone();
    runtime.commit_successor_installation(successor).unwrap();
    assert_eq!(runtime.installed_packages().identity(), &successor_identity);
}

#[test]
fn successor_commit_revokes_prior_installed_domain_execution_authority() {
    let mut runtime = empty_runtime();
    let authority = WorthQueryInstalledDomainExecutionAuthority::mint(
        runtime.authority_identity(),
        "test-domain",
        runtime.installed_packages().generation(),
        runtime.retain_current_generation(),
    );

    assert!(authority.is_current_installation_generation());

    let successor = Arc::new(runtime.installed_packages().successor_generation());
    runtime.commit_successor_installation(successor).unwrap();

    assert!(!authority.is_current_installation_generation());
}

#[test]
fn foreign_runtime_index_cannot_replace_or_advance_the_root() {
    let mut runtime = empty_runtime();
    let foreign = empty_runtime().retain_installed_packages();

    assert_eq!(
        runtime.replace_rebuilt_installation(Arc::clone(&foreign)),
        Err(WorthQueryExecutionInstallationCommitDenial::ForeignRuntime)
    );
    assert_eq!(
        runtime.commit_successor_installation(foreign),
        Err(WorthQueryExecutionInstallationCommitDenial::ForeignRuntime)
    );
}

#[test]
fn skipped_generation_cannot_advance_the_execution_root() {
    let mut runtime = empty_runtime();
    let first_successor = runtime.installed_packages().successor_generation();
    let skipped = Arc::new(first_successor.successor_generation());

    assert_eq!(
        runtime.commit_successor_installation(skipped),
        Err(WorthQueryExecutionInstallationCommitDenial::ExactSuccessorRequired)
    );
}
