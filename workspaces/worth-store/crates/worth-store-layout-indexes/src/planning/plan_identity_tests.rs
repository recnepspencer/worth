use worth_store_budgets::{PreExecutionBudgetEnvelope, PreExecutionBudgetScope};
use super::AccessPlanSelector;

#[test]
fn selected_read_authority_is_a_compact_handle_to_native_plan_identity() {
    use std::mem::size_of;

    assert_eq!(size_of::<super::AccessPlanIdentity>(), size_of::<usize>());
    for (name, size) in [
        ("LSM lookup", size_of::<super::SelectedLsmLookup>()),
        (
            "degraded exact scan",
            size_of::<super::SelectedDegradedExactScan>(),
        ),
    ] {
        assert!(
            size <= 256,
            "{name} selected authority embedded {size} bytes instead of retaining a compact identity handle"
        );
    }
}

#[test]
fn plan_identity_equality_includes_exact_admitted_budget_posture() {
    let (family, domain) = crate::strategy::tests_support::admit_persisted_lsm_scope();
    let catalog = crate::bootstrap::test_support::bootstrap_catalog_read_admission();
    let materialization = crate::strategy::tests_support::persisted_lsm_materialization(family, &catalog).0;
    let select = |budget| {
        let key = crate::keyspace::admit_wal_key(
            domain,
            worth_store_contracts::WalRecordFamily::DurableMutationIntent,
            worth_store_wal::StoreWalRecordIdentity::new(1),
        )
        .expect("WAL key must admit");
        let request = AccessPlanSelector
            .admit_read_request(
                family,
                key,
                materialization.clone(),
                crate::access_planning().point_access(),
            )
            .expect("WAL point request must admit");
        AccessPlanSelector
            .select_admitted_with_budget(request, budget)
            .into_lsm_lookup()
            .expect("WAL point request must select LSM lookup")
    };

    let broad = select(PreExecutionBudgetEnvelope::foreground_default());
    let replayed = select(PreExecutionBudgetEnvelope::foreground_default());
    let exact = select(PreExecutionBudgetEnvelope::new(
        PreExecutionBudgetScope::Foreground,
        u64::MAX,
        u16::MAX,
        u16::MAX,
        u16::MAX,
        u64::MAX,
    ));

    assert_eq!(broad.fingerprint(), replayed.fingerprint());
    assert_ne!(broad.fingerprint(), exact.fingerprint());
    assert_eq!(
        exact.fingerprint().budget_request(),
        exact.budget_receipt().request()
    );
    assert_eq!(
        exact.fingerprint().budget_envelope(),
        exact.budget_receipt().admitted_envelope()
    );
    assert_eq!(exact.fingerprint().cost_estimate(), exact.cost_estimate());
}
