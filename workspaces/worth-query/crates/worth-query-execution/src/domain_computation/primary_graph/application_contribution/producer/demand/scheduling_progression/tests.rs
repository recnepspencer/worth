use super::*;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandRegistry;
use crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world;
use std::sync::{mpsc, Mutex};
use std::time::Duration;
use worth_runtime_bridge::facade::*;

struct Compute;
struct Block {
    entered: mpsc::SyncSender<()>,
    release: Mutex<mpsc::Receiver<()>>,
}
impl BridgeConditionalProviderSemantics for Compute {
    type SemanticContract = ();
    fn semantic_contract(&self) {}
    fn retained_heap_bytes(
        &self,
        _: &(),
    ) -> Result<BridgeConditionalProviderHeapRetention, BridgeConditionalProviderRetentionOverflow>
    {
        Ok(BridgeConditionalProviderHeapRetention::none())
    }
}
impl BridgeConditionalComputeProvider for Compute {
    fn compute(
        &self,
        context: &mut dyn std::any::Any,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        if let Some(block) = context.downcast_ref::<Block>() {
            block.entered.send(()).unwrap();
            block
                .release
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(5))
                .expect("the controller releases the occupied slot");
            return Err("the competing attempt releases its slot before computation".to_owned());
        }
        Ok(worth_signal::facade::NodeEvaluationResult::from_version(
            worth_signal::facade::AspectVersion::from_updates([(
                worth_signal::facade::Aspect::new(0),
                if context.is::<Block>() { 1 } else { 2 },
            )]),
        ))
    }
}
struct UnusedSink;
impl InvalidationSink for UnusedSink {
    fn deliver_invalidation(
        &self,
        _: BridgeSignalInvalidationDelivery,
        _: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        panic!("source-free evaluation never delivers a patch")
    }
}
fn installation() -> (
    BridgeSealedRuntimeAssembly,
    Arc<BridgeInstalledConditionalLowering>,
) {
    use worth_foundational::facade::{AspectKey, FieldKey, ScalarAspectType};
    use worth_signal::facade::{
        SignalConditionalArtifactReuse, SignalConditionalVersionComparator,
    };
    let world = installed_authorization_world(true);
    let bridge = RuntimeBridge::builder()
        .with_relational_source(world.application.product_runtime.source.clone())
        .with_signal_sink(UnusedSink)
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("scheduling-busy"),
            TruthPatchScope::for_entity_field(
                MappingSelector::exact("scheduling-busy"),
                AspectKey::new("probe").unwrap(),
                FieldKey::new("value".to_owned()).unwrap(),
            ),
            SnapshotReadContract::scalar(
                AspectKey::new("probe").unwrap(),
                ScalarAspectType::String,
            ),
            SignalInvalidationScope::from_stable_name("scheduling-busy"),
            CoarseRoutingMode::Direct,
        ))
        .build()
        .unwrap();
    let mut builder = BridgeConditionalRuntimeBuilder::with_owned_signal_graph(
        bridge,
        worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
    )
    .unwrap();
    let lowering = builder
        .install_owned_conditional(BridgeOwnedConditionalInstallationRequest {
            contract: BridgeConditionalContract::new(BridgeConditionalContractParts {
                identity: Arc::from("scheduling-busy"),
                dependency_count: 0,
                condition_dependency_ordinals: vec![],
                condition: BridgeConditionalCondition::Always,
                dependency_comparator: SignalConditionalVersionComparator::Exact,
                output_comparator: SignalConditionalVersionComparator::Exact,
                artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
            }),
            location: BridgeConditionalLocation::operation("scheduling-busy"),
            dependencies: vec![],
            providers: BridgeConditionalProviderSet::new().compute(Compute),
        })
        .unwrap();
    (builder.seal().unwrap(), lowering)
}
fn operation(
    lowering: &Arc<BridgeInstalledConditionalLowering>,
    attempt: u64,
) -> BridgeConditionalExecutionRequest<'_> {
    BridgeConditionalExecutionRequest {
        lowering,
        query_binding_identity: "scheduling-busy",
        query_capability_identity: 1,
        snapshot_identity: "source-free",
        truth_branch_identity: None,
        bridge_snapshot_identity: None,
        execution_identity: "scheduling-busy",
        attempt,
    }
}

#[test]
fn busy_signal_slot_defers_with_custody_then_a_later_advance_completes() {
    let (owner, lowering) = installation();
    let policy = BridgeExecutionPolicyBaseline::operational().request_policy();
    let serial = worth_execution::SerialRequest::from_policy(
        &policy,
        worth_execution::CancellationToken::new(),
        None,
    );
    let request = worth_execution::ExecutionRequest::serial(&serial);
    let basis = owner
        .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
        .unwrap();
    let session = owner
        .admit_conditional_evaluation(
            BridgeConditionalEvaluationAdmissionRequest::source_free_at_signal_basis(&basis),
            request,
        )
        .unwrap();
    let (entered, ready) = mpsc::sync_channel(1);
    let (release, released) = mpsc::sync_channel(1);
    std::thread::scope(|threads| {
        let first = threads.spawn(|| {
            owner.execute_admitted_conditional(
                request,
                &session,
                operation(&lowering, 1),
                &mut Block {
                    entered,
                    release: Mutex::new(released),
                },
            )
        });
        ready
            .recv_timeout(Duration::from_secs(5))
            .expect("compute occupies the actual Signal slot");
        let denial = owner
            .execute_admitted_conditional(request, &session, operation(&lowering, 2), &mut ())
            .err()
            .unwrap();
        release.send(()).unwrap();
        assert!(first.join().unwrap().is_err());
        assert_eq!(
            denial.kind(),
            BridgeConditionalDenialKind::SignalExecution(BridgeSignalDenial::ConditionalExecution(
                worth_signal::facade::branch::SignalConditionalServiceExecutionDenial::SlotBusy
            ))
        );
        WorthQueryOutputDemandRegistry::verify_bridge_scheduling_for_test(
            scheduling_refusal("scheduling-busy", denial),
            || {
                let result = owner
                    .execute_admitted_conditional(
                        request,
                        &session,
                        operation(&lowering, 3),
                        &mut (),
                    )
                    .unwrap();
                assert!(matches!(
                    result.signal().class(),
                    worth_signal::facade::SignalConditionalDecisionClass::ComputedChanged
                ));
                WorthQueryOutputSchedulingResult::Scheduled
            },
            true,
        );
    });
}

#[test]
fn non_temporary_bridge_refusal_remains_terminal() {
    let (owner, lowering) = installation();
    // A native lowering from another owner is intrinsically foreign.
    let (foreign, _) = installation();
    let denial = foreign
        .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
        .err()
        .unwrap();
    let native = denial.kind();
    let result = scheduling_refusal("scheduling-busy", denial);
    assert_eq!(
        result.as_ref().err().unwrap().kind(),
        WorthQueryOutputDemandDenialKind::BridgeConditional(Box::new(native))
    );
    WorthQueryOutputDemandRegistry::verify_bridge_scheduling_for_test(
        result,
        || panic!("terminal row cannot retry"),
        false,
    );
}
