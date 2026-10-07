use crate::facade::{access_planning, deterministic_plan_selection};
use crate::strategy::tests_support::{admit_persisted_lsm_scope, persisted_lsm_materialization};
use worth_store_budgets::PreExecutionBudgetEnvelope;
use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_security::{
    StoreAuthenticityRequirement, StoreAuthenticityRequirementClass, StoreCustodyPosture,
    StoreKeyScope, StoreTenantScope,
};

#[test]
fn lsm_point_selection_issues_exact_lookup_authority() {
    let (lifecycle, key_domain) = admit_wal_scope();
    let catalog = crate::bootstrap::test_support::bootstrap_catalog_read_admission();
    let (materialization, source) = persisted_lsm_materialization(lifecycle, &catalog);
    let outcome = deterministic_plan_selection().select_admitted_with_budget(
        crate::planning::AccessPlanSelector
            .admit_read_request(
                lifecycle,
                crate::keyspace::admit_wal_key(
                    key_domain,
                    worth_store_contracts::WalRecordFamily::DurableMutationIntent,
                    worth_store_wal::StoreWalRecordIdentity::new(1),
                )
                .expect("WAL identity must pass ordinary key admission"),
                materialization,
                access_planning().point_access(),
            )
            .expect("test request must pass ordinary admission"),
        PreExecutionBudgetEnvelope::foreground_default(),
    );
    let selected = outcome
        .into_lsm_lookup()
        .expect("LSM point selection must issue exact LSM lookup authority");
    assert_eq!(
        selected.selected_family(),
        crate::strategy::LayoutStrategyFamily::BaselineLsmWriteOptimized
    );
    let admission = crate::BaselineLsmLookupAdmission::admit(
        selected.clone(),
        access_planning().current_lsm_materialization_frontier(&catalog, &source),
    );
    assert!(matches!(
        admission.view(),
        crate::BaselineLsmLookupAdmissionView::Admitted(_)
    ));
    let admission = admission
        .into_admitted()
        .expect("current exact LSM selection must be admitted");
    assert_eq!(
        admission.selected(),
        &selected,
        "LSM admission must retain the owner-issued selection rather than reconstruct authority from its projections",
    );
}

#[test]
fn lsm_lookup_readiness_declares_exactly_the_cases_owner_admission_observes() {
    let (family, key_domain) = admit_wal_scope();
    let current_catalog = crate::bootstrap::test_support::bootstrap_catalog_read_admission();
    let (materialization, source) = persisted_lsm_materialization(family, &current_catalog);
    let selected = deterministic_plan_selection()
        .select_admitted_with_budget(
            crate::planning::AccessPlanSelector
                .admit_read_request(
                    family,
                    crate::keyspace::admit_wal_key(
                        key_domain,
                        worth_store_contracts::WalRecordFamily::DurableMutationIntent,
                        worth_store_wal::StoreWalRecordIdentity::new(1),
                    )
                    .expect("WAL identity must pass ordinary key admission"),
                    materialization,
                    access_planning().point_access(),
                )
                .expect("test request must pass ordinary admission"),
            PreExecutionBudgetEnvelope::foreground_default(),
        )
        .into_lsm_lookup()
        .unwrap();
    let advanced_catalog =
        crate::bootstrap::test_support::advanced_bootstrap_catalog_read_admission();
    let observed = [
        crate::BaselineLsmLookupAdmission::admit(
            selected.clone(),
            access_planning().current_lsm_materialization_frontier(&current_catalog, &source),
        )
        .case_id()
        .name(),
        crate::BaselineLsmLookupAdmission::admit(
            selected,
            access_planning().current_lsm_materialization_frontier(&advanced_catalog, &source),
        )
        .case_id()
        .name(),
    ]
    .into_iter()
    .collect::<std::collections::BTreeSet<_>>();
    let declared = crate::baseline_lsm_lookup_admission_cases()
        .map(|case| case.name())
        .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(observed, declared);
}

#[test]
fn lsm_mutation_and_recovery_intents_issue_exact_operation_capabilities() {
    let (lifecycle, key_domain) = admit_wal_scope();
    let budget = PreExecutionBudgetEnvelope::maintenance_default();

    let publication = deterministic_plan_selection()
        .select_admitted_with_budget(
            crate::planning::AccessPlanSelector
                .admit_mutation_request(
                    lifecycle,
                    crate::keyspace::admit_wal_key(
                        key_domain,
                        worth_store_contracts::WalRecordFamily::DurableMutationIntent,
                        worth_store_wal::StoreWalRecordIdentity::new(1),
                    )
                    .expect("WAL identity must pass ordinary key admission"),
                    crate::access_shapes()
                        .append(crate::PhysicalMutationShape::LogStructuredAppend)
                        .unwrap(),
                )
                .expect("test request must pass ordinary admission"),
            budget,
        )
        .into_lsm_run_publication()
        .expect("LSM append must issue run-publication authority");
    let replay = deterministic_plan_selection()
        .select_admitted_with_budget(
            crate::planning::AccessPlanSelector
                .admit_recovery_request(
                    lifecycle,
                    crate::keyspace::admit_wal_key(
                        key_domain,
                        worth_store_contracts::WalRecordFamily::DurableMutationIntent,
                        worth_store_wal::StoreWalRecordIdentity::new(1),
                    )
                    .expect("WAL identity must pass ordinary key admission"),
                    wal_materialization(lifecycle, 23),
                    crate::access_shapes()
                        .rebuild_read(crate::AccessLaneClassification::Maintenance)
                        .unwrap(),
                )
                .expect("test request must pass ordinary admission"),
            budget,
        )
        .into_lsm_replay_recovery()
        .expect("LSM rebuild read must issue replay authority");
    let compaction = deterministic_plan_selection()
        .select_admitted_with_budget(
            crate::planning::AccessPlanSelector
                .admit_mutation_request(
                    lifecycle,
                    crate::keyspace::admit_wal_key(
                        key_domain,
                        worth_store_contracts::WalRecordFamily::DurableMutationIntent,
                        worth_store_wal::StoreWalRecordIdentity::new(1),
                    )
                    .expect("WAL identity must pass ordinary key admission"),
                    crate::access_shapes()
                        .compaction_read(crate::PhysicalMutationShape::CompactionRewrite)
                        .unwrap(),
                )
                .expect("test request must pass ordinary admission"),
            budget,
        )
        .into_lsm_compaction()
        .expect("LSM compaction read must issue compaction authority");

    let publication_admission =
        crate::BaselineLsmRunPublicationAdmission::admit(publication.clone());
    let compaction_admission = crate::BaselineLsmCompactionAdmission::admit(compaction.clone());

    assert_eq!(publication_admission.selected(), &publication);
    assert_eq!(compaction_admission.selected(), &compaction);
    assert_eq!(publication.request_identity(), replay.request_identity());
    assert_eq!(replay.request_identity(), compaction.request_identity());
    assert_ne!(publication.fingerprint(), replay.fingerprint());
    assert_ne!(replay.fingerprint(), compaction.fingerprint());
}

#[test]
fn degraded_owner_rebind_preserves_the_selected_operation_identity() {
    let ready = super::tests_support::rebound_owner_degraded_scan();
    assert_eq!(ready.basis().fingerprint(), ready.selected().fingerprint());
}

#[test]
fn degraded_rebind_rejects_replacement_from_another_store_authority() {
    let stale = super::tests_support::stale_owner_degraded_scan();
    let advanced = crate::bootstrap::test_support::advanced_bootstrap_catalog_read_admission();
    let (foreign_family, foreign_domain) =
        crate::strategy::tests_support::admit_strategy_scope_for_store(
            DurableArtifactFamilyId::PhysicalPage,
            StoreKeyScope::PageEnvelope,
            StoreTenantScope::TenantPhysicalBoundary,
            StoreAuthenticityRequirement::required(
                StoreAuthenticityRequirementClass::AuthenticatedFrame,
            ),
            StoreCustodyPosture::InternalStoreCustody,
            "store.foreign.degraded_readmission",
        );
    let foreign =
        super::tests_support::selected_degraded_scan(foreign_family, foreign_domain, &advanced);

    assert!(matches!(
        crate::degraded_scan_runtime().admit_rebind(&stale, &foreign),
        Err(crate::DegradedScanAdmissionDenied::ReplacementAuthorityMismatch { .. })
    ));
}

#[test]
fn degraded_rebind_rejects_replacement_from_the_displaced_source() {
    let stale = super::tests_support::stale_owner_degraded_scan();
    let (replacement, _) = super::tests_support::selected_owner_degraded_scan();

    assert!(matches!(
        crate::degraded_scan_runtime().admit_rebind(&stale, &replacement),
        Err(crate::DegradedScanAdmissionDenied::ReplacementFrontierMismatch { .. })
    ));
}

#[test]
fn degraded_owner_current_readiness_retains_materialization_authority() {
    let ready = super::tests_support::ready_owner_degraded_scan();
    assert_eq!(
        ready.current_materialization().materialization(),
        ready.selected().materialization()
    );
}

#[test]
fn degraded_owner_execution_retains_current_materialization_and_physical_observation() {
    let ready = super::tests_support::ready_owner_degraded_scan();
    let current = ready.current_materialization().clone();
    let selected = ready.selected().clone();
    let mut physical = crate::bootstrap::test_support::open_physical_facade_for_store(
        crate::strategy::tests_support::strategy_test_store_identity(),
    );
    physical
        .publish_physical_root()
        .expect("degraded scan fixture requires an admitted physical root");
    let execution = crate::degraded_scan_runtime()
        .execute_physical(ready, &mut physical)
        .expect("admitted degraded scan must execute through the physical facade");

    assert_eq!(execution.selected(), &selected);
    assert_eq!(execution.current_materialization(), &current);
    assert_eq!(execution.observed_rows(), 1);
    assert_eq!(
        execution
            .physical_observation()
            .scan()
            .request()
            .budget_rows(),
        8
    );
}

#[test]
fn degraded_readiness_declares_exactly_the_cases_ordinary_execution_observes() {
    let declared = crate::degraded_scan_readiness_cases()
        .map(|case| case.name())
        .collect::<std::collections::BTreeSet<_>>();
    let observed = super::tests_support::observed_degraded_readiness_cases()
        .into_iter()
        .map(|case| case.name())
        .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(declared, observed);
}

fn admit_wal_scope() -> (
    crate::AdmittedPhysicalArtifactFamily,
    crate::AdmittedPhysicalKeyDomain,
) {
    admit_persisted_lsm_scope()
}

fn wal_materialization(
    family: crate::AdmittedPhysicalArtifactFamily,
    _lsn: u64,
) -> crate::AdmittedLayoutMaterialization {
    let catalog = crate::bootstrap::test_support::bootstrap_catalog_read_admission();
    persisted_lsm_materialization(family, &catalog).0
}
