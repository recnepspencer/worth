use super::*;
use crate::domain_computation::execution_resource_admission::{
    prepare_execution_resource_plan, tests,
};

#[test]
fn carried_resource_basis_preserves_canonical_identity_and_changes_each_invocation() {
    let (prepared, counters) = prepare_execution_resource_plan(
        &tests::contract(8),
        &tests::request(8),
        support(),
        Default::default(),
    )
    .unwrap();
    let ordinary = prepared.bind("invocation-a", counters);
    let admitted = prepared
        .bind_admitted("invocation-a", counters, &mut |_, _| Ok::<(), ()>(()))
        .unwrap();
    let different = prepared.bind("invocation-b", counters);
    assert_eq!(ordinary, admitted);
    assert_ne!(ordinary.identity(), different.identity());
    assert!(Arc::ptr_eq(&ordinary.basis, &different.basis));

    // Original public canonical grammar, independently framed from borrowed
    // fields. This exposes delimiter or prefix changes in the streaming writer.
    let mut oracle = sha2::Sha256::new();
    use sha2::{Digest, Sha256};
    for part in [
        "worth_query_admitted_execution_resource_plan_v1".to_owned(),
        "binding:invocation-a".to_owned(),
        format!("contract:{}", ordinary.contract_identity()),
        format!("request:{}", ordinary.request_identity()),
        format!("support:{}", ordinary.support_snapshot().identity()),
        format!("strategy:{}", ordinary.strategy().as_str()),
        format!("envelope:{}", ordinary.envelope_identity()),
    ] {
        oracle.update((part.len() as u64).to_le_bytes());
        oracle.update(part.as_bytes());
    }
    let expected = format!("{:x}", oracle.finalize());
    assert_eq!(ordinary.identity(), expected);
    assert_eq!(ordinary.identity().len(), Sha256::output_size() * 2);
}

#[test]
fn carried_basis_preserves_original_refusal_before_constructing_invocation() {
    let (prepared, counters) = prepare_execution_resource_plan(
        &tests::contract(8),
        &tests::request(8),
        support(),
        Default::default(),
    )
    .unwrap();
    let marker = Arc::new(());
    let mut calls = 0;
    let stop = prepared.bind_admitted("invocation", counters, &mut |_, _| {
        calls += 1;
        Err(Arc::clone(&marker))
    });
    match stop {
        Err(PreparedResourcePlanAdmissionStop::Admission(original)) => {
            assert!(Arc::ptr_eq(&marker, &original))
        }
        _ => panic!("the first preparation refusal must be preserved"),
    }
    assert_eq!(calls, 1);
}

fn support() -> WorthQueryExecutionResourceSupportSnapshot {
    tests::support_with_capacity(
        tests::envelope(8),
        Arc::new(crate::domain_computation::execution_resource_admission::WorthQueryFixedExecutionCapacity::new("prepared-test", 1).unwrap()),
    )
}
