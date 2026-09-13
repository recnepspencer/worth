use std::sync::atomic::{AtomicUsize, Ordering};

use super::{always_eligible_contract, installation_fixture};
use crate::facade::{
    BridgeConditionalComputeProvider, BridgeConditionalDenialKind,
    BridgeConditionalExecutionRequest, BridgeConditionalProviderSemantics,
    BridgeConditionalProviderSet,
};

struct CountingCompute;

impl BridgeConditionalProviderSemantics for CountingCompute {
    type SemanticContract = ();

    fn semantic_contract(&self) {}

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        crate::facade::BridgeConditionalProviderHeapRetention,
        crate::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(crate::facade::BridgeConditionalProviderHeapRetention::none())
    }
}

impl BridgeConditionalComputeProvider for CountingCompute {
    fn compute(
        &self,
        context: &mut dyn std::any::Any,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        context
            .downcast_ref::<AtomicUsize>()
            .unwrap()
            .fetch_add(1, Ordering::SeqCst);
        Ok(worth_signal::facade::NodeEvaluationResult::from_version(
            worth_signal::facade::AspectVersion::from_updates([(
                worth_signal::facade::Aspect::new(0),
                1,
            )]),
        ))
    }
}

#[test]
fn installed_source_dependency_denies_missing_or_mismatched_reader_before_compute() {
    // Native Bridge installation over the explicit TestSource adapter. Its
    // reader admits only snapshot (1, 1); this is not a Relational owner proof.
    let (mut owner, installation) = installation_fixture(
        always_eligible_contract("query:one"),
        &["bridge-main"],
        BridgeConditionalProviderSet::new().compute(CountingCompute),
    );
    let lowering = owner.install(installation).unwrap();
    let owner = owner.seal().unwrap();
    let signal_basis = owner
        .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
        .unwrap();
    let mut computes = AtomicUsize::new(0);
    let wrong = crate::truth_identity_fixtures::truth_snapshot(1, 2);
    let exact = crate::truth_identity_fixtures::truth_snapshot(1, 1);
    for (source, expected) in [
        (
            None,
            Some(BridgeConditionalDenialKind::MissingSourceObservation),
        ),
        (
            Some(&wrong),
            Some(BridgeConditionalDenialKind::SnapshotAdmission),
        ),
        (Some(&exact), None),
    ] {
        let result = owner.execute(
            &signal_basis,
            BridgeConditionalExecutionRequest {
                lowering: &lowering,
                query_binding_identity: "query-binding",
                query_capability_identity: 1,
                snapshot_identity: "descriptive-snapshot",
                truth_branch_identity: Some("descriptive-branch"),
                bridge_snapshot_identity: source,
                execution_identity: "execution",
                attempt: 1,
            },
            &mut computes,
        );
        if let Some(expected) = expected {
            let denial = result.err().expect("source admission must fail");
            assert_eq!(denial.kind(), expected);
            assert_eq!(computes.load(Ordering::SeqCst), 0);
            assert_eq!(denial.signal_counters(), Default::default());
            let counters = denial.bridge_execution_counters();
            assert_eq!(counters.compute_provider_checks, 0);
            assert_eq!(counters.signal_execution_contacts, 0);
            assert_eq!(counters.observation_baseline_writes, 0);
            assert_eq!(counters.decisions_retained, 0);
            if source.is_none() {
                assert_eq!(counters, Default::default());
            }
        } else {
            let evidence = result.unwrap();
            assert_eq!(computes.load(Ordering::SeqCst), 1);
            assert_eq!(
                evidence
                    .bridge_execution_counters()
                    .snapshot_admission_attempts,
                1
            );
            assert_eq!(
                evidence
                    .bridge_execution_counters()
                    .signal_execution_contacts,
                1
            );
            assert!(evidence.performed_signal_invalidation().is_some());
        }
    }

    let product = owner
        .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
        .expect("the exact product Signal basis admits the installed definition");
    let session = owner
        .admit_conditional_evaluation(
            crate::facade::BridgeConditionalEvaluationAdmissionRequest::source_present_at_signal_basis(
                &product, &exact,
            ),
        )
        .expect("source-present product admission uses the exact source and Signal basis");
    let evidence = owner
        .execute_admitted_conditional(
            &session,
            BridgeConditionalExecutionRequest {
                lowering: &lowering,
                query_binding_identity: "query-binding",
                query_capability_identity: 1,
                snapshot_identity: "product-snapshot",
                truth_branch_identity: Some("product-branch"),
                bridge_snapshot_identity: Some(&exact),
                execution_identity: "product-execution",
                attempt: 2,
            },
            &mut computes,
        )
        .unwrap();
    assert_eq!(
        evidence
            .bridge_execution_counters()
            .snapshot_admission_attempts,
        1
    );
    assert_eq!(
        evidence
            .bridge_execution_counters()
            .signal_execution_contacts,
        1
    );
}
