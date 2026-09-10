use std::sync::atomic::{AtomicUsize, Ordering};

use super::{always_eligible_contract, installation_fixture_with_budget};
use crate::facade::{
    BridgeConditionalComputeProvider, BridgeConditionalDenialKind,
    BridgeConditionalEvaluationAdmissionRequest, BridgeConditionalExecutionRequest,
    BridgeConditionalProviderSemantics, BridgeConditionalProviderSet,
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

struct BlockingCompute;

impl BridgeConditionalProviderSemantics for BlockingCompute {
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

impl BridgeConditionalComputeProvider for BlockingCompute {
    fn compute(
        &self,
        context: &mut dyn std::any::Any,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        let context = context
            .downcast_mut::<BlockingComputeContext>()
            .expect("only the admitted execution may reach the blocking provider");
        context.computes.fetch_add(1, Ordering::SeqCst);
        context
            .entered
            .send(())
            .map_err(|_| "the test entry observer was dropped".to_owned())?;
        context
            .release
            .recv_timeout(std::time::Duration::from_secs(5))
            .map_err(|_| "the test did not release the blocking compute".to_owned())?;
        Ok(worth_signal::facade::NodeEvaluationResult::from_version(
            worth_signal::facade::AspectVersion::from_updates([(
                worth_signal::facade::Aspect::new(0),
                1,
            )]),
        ))
    }
}

struct BlockingComputeContext {
    computes: std::sync::Arc<AtomicUsize>,
    entered: std::sync::mpsc::SyncSender<()>,
    release: std::sync::mpsc::Receiver<()>,
}

impl BridgeConditionalComputeProvider for CountingCompute {
    fn compute(
        &self,
        context: &mut dyn std::any::Any,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        context
            .downcast_ref::<AtomicUsize>()
            .expect("the test supplies its independent compute counter")
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
fn cold_source_recomputes_and_a_live_slot_forces_typed_capacity_denial() {
    let budget = worth_signal::facade::runtime::SignalConditionalEvaluationBudget {
        maximum_retained_slots: 1,
        maximum_retained_bytes: 512 * 1024 * 1024,
        maximum_attempt_visits: 8_000_000,
    };
    let (mut builder, installation) = installation_fixture_with_budget(
        always_eligible_contract("query:one"),
        &["bridge-main"],
        BridgeConditionalProviderSet::new().compute(CountingCompute),
        budget,
    );
    let lowering = builder.install(installation).unwrap();
    let owner = builder.seal().unwrap();
    let signal_basis = owner
        .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
        .unwrap();
    let source = crate::truth_identity_fixtures::truth_snapshot(1, 1);
    let session = owner
        .admit_conditional_evaluation(
            BridgeConditionalEvaluationAdmissionRequest::source_present_at_signal_basis(
                &signal_basis,
                &source,
            ),
        )
        .unwrap();
    let mut computes = AtomicUsize::new(0);
    let evidence = owner
        .execute_admitted_conditional(&session, request(&lowering, &source, 1), &mut computes)
        .unwrap();
    assert_eq!(computes.load(Ordering::SeqCst), 1);
    assert_eq!(evidence.signal().counters().compute_contacts, 1);
    assert_eq!(
        evidence
            .bridge_execution_counters()
            .snapshot_admission_attempts,
        1
    );
    assert_eq!(
        evidence.bridge_execution_counters().signal_slot_reuse_hits,
        0
    );

    let denial = owner
        .admit_conditional_evaluation(
            BridgeConditionalEvaluationAdmissionRequest::source_present_at_signal_basis(
                &signal_basis,
                &source,
            ),
        )
        .err()
        .expect("the retained first session must hold the only installed Signal slot");
    assert_eq!(
        denial.kind(),
        BridgeConditionalDenialKind::ConditionalEvaluationAdmissionCapacity
    );
    assert_eq!(computes.load(Ordering::SeqCst), 1);
    assert_eq!(denial.signal_counters(), Default::default());
}

#[test]
fn rejected_affinity_does_not_manufacture_a_later_slot_reuse() {
    let (mut builder, installation) = installation_fixture_with_budget(
        always_eligible_contract("query:one"),
        &["bridge-main"],
        BridgeConditionalProviderSet::new().compute(CountingCompute),
        worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
    );
    let lowering = builder.install(installation).unwrap();
    let owner = builder.seal().unwrap();
    let signal_basis = owner
        .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
        .unwrap();
    let source = crate::truth_identity_fixtures::truth_snapshot(1, 1);
    let wrong_source = crate::truth_identity_fixtures::truth_snapshot(1, 2);
    let session = owner
        .admit_conditional_evaluation(
            BridgeConditionalEvaluationAdmissionRequest::source_present_at_signal_basis(
                &signal_basis,
                &source,
            ),
        )
        .unwrap();
    let mut computes = AtomicUsize::new(0);

    let denial = match owner.execute_admitted_conditional(
        &session,
        request(&lowering, &wrong_source, 1),
        &mut computes,
    ) {
        Ok(_) => panic!("wrong source affinity must be denied"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        BridgeConditionalDenialKind::SnapshotAdmission
    );
    assert_eq!(denial.bridge_execution_counters().signal_slot_reuse_hits, 0);
    assert_eq!(
        denial
            .bridge_execution_counters()
            .snapshot_admission_attempts,
        0
    );

    let evidence = owner
        .execute_admitted_conditional(&session, request(&lowering, &source, 2), &mut computes)
        .unwrap();
    assert_eq!(computes.load(Ordering::SeqCst), 1);
    assert_eq!(
        evidence.bridge_execution_counters().signal_slot_reuse_hits,
        0
    );
    assert_eq!(
        evidence
            .bridge_execution_counters()
            .snapshot_admission_attempts,
        1
    );
}

#[test]
fn concurrent_busy_denial_does_not_claim_slot_reuse() {
    let (mut builder, installation) = installation_fixture_with_budget(
        always_eligible_contract("query:one"),
        &["bridge-main"],
        BridgeConditionalProviderSet::new().compute(BlockingCompute),
        worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
    );
    let lowering = builder.install(installation).unwrap();
    let owner = std::sync::Arc::new(builder.seal().unwrap());
    let signal_basis = owner
        .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
        .unwrap();
    let source = crate::truth_identity_fixtures::truth_snapshot(1, 1);
    let session = std::sync::Arc::new(
        owner
            .admit_conditional_evaluation(
                BridgeConditionalEvaluationAdmissionRequest::source_present_at_signal_basis(
                    &signal_basis,
                    &source,
                ),
            )
            .unwrap(),
    );
    let computes = std::sync::Arc::new(AtomicUsize::new(0));
    let (entered_sender, entered_receiver) = std::sync::mpsc::sync_channel(1);
    let (release_sender, release_receiver) = std::sync::mpsc::sync_channel(1);

    std::thread::scope(|scope| {
        let executing_owner = std::sync::Arc::clone(&owner);
        let executing_session = std::sync::Arc::clone(&session);
        let executing_lowering = std::sync::Arc::clone(&lowering);
        let executing_source = source.clone();
        let executing_computes = std::sync::Arc::clone(&computes);
        let first = scope.spawn(move || {
            let mut context = BlockingComputeContext {
                computes: executing_computes,
                entered: entered_sender,
                release: release_receiver,
            };
            executing_owner.execute_admitted_conditional(
                &executing_session,
                request(&executing_lowering, &executing_source, 1),
                &mut context,
            )
        });

        entered_receiver
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("the first execution must reach its compute provider");
        let contender =
            owner.execute_admitted_conditional(&session, request(&lowering, &source, 2), &mut ());
        release_sender
            .send(())
            .expect("the blocked execution must still be waiting");
        let first = first.join().unwrap().unwrap();
        let denial = match contender {
            Ok(_) => panic!("a concurrent execution must not enter the occupied Signal slot"),
            Err(denial) => denial,
        };
        assert_eq!(
            denial.kind(),
            BridgeConditionalDenialKind::ConditionalEvaluationBusy
        );
        assert_eq!(denial.bridge_execution_counters().signal_slot_reuse_hits, 0);
        assert_eq!(
            denial
                .bridge_execution_counters()
                .snapshot_admission_attempts,
            0
        );
        assert_eq!(first.bridge_execution_counters().signal_slot_reuse_hits, 0);
        assert_eq!(
            first
                .bridge_execution_counters()
                .snapshot_admission_attempts,
            1
        );
    });
    assert_eq!(computes.load(Ordering::SeqCst), 1);
}

fn request<'a>(
    lowering: &'a std::sync::Arc<crate::facade::BridgeInstalledConditionalLowering>,
    source: &'a crate::snapshot::TruthSnapshotIdentity,
    attempt: u64,
) -> BridgeConditionalExecutionRequest<'a> {
    BridgeConditionalExecutionRequest {
        lowering,
        query_binding_identity: "budget-query",
        query_capability_identity: 1,
        snapshot_identity: "budget-snapshot",
        truth_branch_identity: Some("budget-branch"),
        bridge_snapshot_identity: Some(source),
        execution_identity: "budget-execution",
        attempt,
    }
}
