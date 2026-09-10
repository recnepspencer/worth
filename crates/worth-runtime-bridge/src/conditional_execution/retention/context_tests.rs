use super::retention::BridgeRetentionReservation;
use super::*;
use crate::policy::BridgeConditionalRetentionBudget;
use std::alloc::Layout;
use std::mem::size_of;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use worth_foundational::facade::{CanonicalFieldPath, FieldKey};

#[derive(Default)]
pub(crate) struct ContextRetentionCapture {
    pub(crate) context: Mutex<Option<BridgeConditionalResolverContext>>,
    pub(crate) predicates: AtomicUsize,
    pub(crate) computes: AtomicUsize,
}

pub(crate) fn assert_context_retention(
    factory: impl Fn(
        BridgeConditionalRetentionBudget,
        Arc<ContextRetentionCapture>,
    ) -> (
        BridgeSealedRuntimeAssembly,
        Arc<BridgeInstalledConditionalLowering>,
    ),
) {
    let baseline = arc_layout::<observation_retention::BridgeObservationBaselines>();
    let decision = arc_layout::<retained_decision::BridgeRetainedConditionalDecisionCore>()
        + size_of::<BridgeConditionalDecisionEvidence>() as u64
        + text("snapshot-label")
        + text("execution")
        + text("query-binding");
    let context =
        arc_layout::<BridgeRetentionReservation>() + text("snapshot-label") + text("main");
    // One Arc backing contains a Vec header and the embedded reservation. Its
    // single observation has no value artifacts, and retains the `name` mask.
    let observations = arc_layout::<(
        Vec<BridgeConditionalSemanticObservation>,
        BridgeRetentionReservation,
    )>() + size_of::<BridgeConditionalSemanticObservation>() as u64
        + size_of::<CanonicalFieldPath>() as u64
        + size_of::<FieldKey>() as u64
        + "name".len() as u64;
    let prepared = baseline + decision + context;
    let exact = prepared + observations;
    for ceiling in [exact, exact - 1, prepared - 1] {
        let capture = Arc::new(ContextRetentionCapture::default());
        let (owner, lowering) = factory(
            BridgeConditionalRetentionBudget {
                maximum_retained_bytes: ceiling,
                ..BridgeConditionalRetentionBudget::development()
            },
            Arc::clone(&capture),
        );
        let ledger = Arc::clone(&owner.test_runtime().retention);
        let basis = owner
            .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
            .unwrap();
        let source = crate::truth_identity_fixtures::truth_snapshot(1, 1);
        let session = owner
            .admit_conditional_evaluation(
                BridgeConditionalEvaluationAdmissionRequest::source_present_at_signal_basis(
                    &basis, &source,
                ),
            )
            .unwrap();
        assert_eq!(ledger.usage(), (0, 0, baseline));
        let result = owner.execute_admitted_conditional(
            &session,
            BridgeConditionalExecutionRequest {
                lowering: &lowering,
                query_binding_identity: "query-binding",
                query_capability_identity: 1,
                snapshot_identity: "snapshot-label",
                truth_branch_identity: Some("main"),
                bridge_snapshot_identity: Some(&source),
                execution_identity: "execution",
                attempt: 1,
            },
            &mut (),
        );
        if ceiling != exact {
            let denial = result.err().expect("one missing byte must deny admission");
            assert_eq!(
                denial.kind(),
                BridgeConditionalDenialKind::ConditionalRetentionCapacity
            );
            assert_eq!(
                denial.bridge_execution_counters().signal_execution_contacts,
                usize::from(ceiling == exact - 1),
                "Context shortage rejects before Signal; observation shortage precedes providers"
            );
            assert_eq!(capture.predicates.load(Ordering::SeqCst), 0);
            assert_eq!(capture.computes.load(Ordering::SeqCst), 0);
            assert!(capture.context.lock().unwrap().is_none());
            assert_eq!(ledger.usage(), (0, 0, baseline));
            drop((session, basis, lowering, owner));
            assert_eq!(ledger.usage(), (0, 0, 0));
            continue;
        }
        let evidence = result.unwrap();
        assert_eq!(evidence.semantic_observation_reads(), 1);
        assert_eq!(capture.predicates.load(Ordering::SeqCst), 1);
        assert_eq!(capture.computes.load(Ordering::SeqCst), 1);
        assert_eq!(ledger.usage(), (0, 0, exact));
        let escaped = capture.context.lock().unwrap().take().unwrap();
        let last_clone = escaped.clone();
        drop(evidence);
        assert_eq!(ledger.usage(), (0, 0, baseline + context + observations));
        drop((session, basis, lowering, owner));
        assert_eq!(ledger.usage(), (0, 0, context + observations));
        assert_eq!(last_clone.truth_snapshot_identity(), "snapshot-label");
        assert_eq!(last_clone.truth_branch_identity(), Some("main"));
        assert_eq!(last_clone.observations().len(), 1);
        let observation = &last_clone.observations()[0];
        assert!(observation.previous().is_none() && observation.current().is_none());
        assert_eq!(
            observation.projection_mask().paths()[0].fields()[0].as_str(),
            "name"
        );
        drop(escaped);
        assert_eq!(ledger.usage(), (0, 0, context + observations));
        drop(last_clone);
        assert_eq!(ledger.usage(), (0, 0, 0));
    }
}

fn arc_layout<T>() -> u64 {
    Layout::new::<[AtomicUsize; 2]>()
        .extend(Layout::new::<T>())
        .unwrap()
        .0
        .pad_to_align()
        .size() as u64
}

fn text(value: &str) -> u64 {
    Layout::new::<[AtomicUsize; 2]>()
        .extend(Layout::array::<u8>(value.len()).unwrap())
        .unwrap()
        .0
        .pad_to_align()
        .size() as u64
}
