use std::sync::atomic::Ordering;
use std::sync::Arc;

use super::*;

#[test]
fn ordered_record_capacity_denies_before_insertion() {
    let registry =
        WorthQueryOutputDemandRegistry::with_budgets(4 * 1_024 * 1_024, 1, 4 * 1_024 * 1_024);
    let demand = key("capacity-short", 1, 1);
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(root(1));
    let denial = match registry.admit(
        demand.clone(),
        None,
        scope,
        occurrence(),
        super::super::DemandAdmissionKind::Ordinary,
        None,
        None,
        &mut record_admission(),
    ) {
        Ok(_) => panic!("a demand without record capacity was admitted"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
    );
    let state = registry.state.lock().unwrap();
    assert!(state.records.is_empty());
    assert_eq!(state.record_retained_bytes.load(Ordering::Acquire), 0);
}

#[test]
fn notification_keeps_wake_capacity_after_record_removal() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let demand = key("retained-wake", 1, 1);
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(root(1));
    let interest = registry
        .admit(
            demand.clone(),
            None,
            scope,
            occurrence(),
            super::super::DemandAdmissionKind::Ordinary,
            None,
            None,
            &mut record_admission(),
        )
        .expect("the actual registry admits the demand");
    let retained = Arc::clone(&registry.state.lock().unwrap().record_retained_bytes);
    let notification = interest.notifications.clone();
    drop(interest);
    {
        let state = registry.state.lock().unwrap();
        assert!(!state.records.contains_key(&demand));
    }
    let with_notification = retained.load(Ordering::Acquire);
    drop(notification);
    let empty_root = retained.load(Ordering::Acquire);
    assert!(
        with_notification > empty_root,
        "the retained wake owns extra capacity"
    );
    assert!(
        empty_root > 0,
        "the empty ordered map retains its root credit"
    );
    drop(registry);
    assert_eq!(retained.load(Ordering::Acquire), 0);
}
