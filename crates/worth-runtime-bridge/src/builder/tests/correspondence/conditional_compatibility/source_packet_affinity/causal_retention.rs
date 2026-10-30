//! Actual Native observations outlive operational evidence without pinning readers.
use super::*;
mod envelope;
mod history;
use crate::facade::{
    BridgeConditionalDecisionReentryRequest, BridgeInstalledConditionalLowering,
    BridgeSealedRuntimeAssembly,
};

#[test]
fn causal_observation_preserves_native_custody_without_reader_or_second_performed_receipt() {
    let contacts = Arc::new(Contacts::default());
    let readers = Arc::new(AtomicUsize::new(0));
    let (owner, lowering) = owner(Arc::clone(&contacts), Arc::clone(&readers));
    let probe = owner.conditional_lifecycle_probe();
    let bridge_baseline = probe
        .bridge_conditional_retention()
        .unwrap()
        .retained_bytes();
    let serial = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let source = crate::truth_identity_fixtures::truth_snapshot(1, 1);
    let basis = owner
        .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
        .unwrap();
    let mut evidence = owner
        .execute(
            execution,
            &basis,
            BridgeConditionalExecutionRequest {
                lowering: &lowering,
                query_binding_identity: "query-binding",
                query_capability_identity: 1,
                snapshot_identity: "snapshot-label",
                truth_branch_identity: None,
                bridge_snapshot_identity: Some(&source),
                execution_identity: "execution",
                attempt: 1,
            },
            &mut (),
        )
        .unwrap();
    let causal = evidence.retain_causal_observation();
    let last_causal = causal.clone();
    assert!(causal.same_observation(&last_causal));
    let signal = causal.performed_signal_invalidation().unwrap().clone();
    let performed = evidence.take_performed_signal_invalidation().unwrap();
    assert!(signal.same_observation(&performed.retain_causal_observation()));
    assert!(
        signal.retains_executed_target(evidence.signal_graph_instance_id(), evidence.signal_node())
    );
    assert!(evidence.take_performed_signal_invalidation().is_none());
    let seed = evidence.retain_for_reentry();
    let reentered = owner
        .reenter_retained_conditional_decision(BridgeConditionalDecisionReentryRequest {
            seed: &seed,
            lowering: &lowering,
            query_binding_identity: "fresh-binding",
            query_capability_identity: 2,
            snapshot_identity: "snapshot-label",
            bridge_snapshot_identity: Some(&source),
        })
        .unwrap();
    assert!(reentered.performed_signal_invalidation().is_none());
    assert!(causal.same_observation(&reentered.retain_causal_observation()));
    assert_eq!(readers.load(Ordering::SeqCst), 1);
    drop((performed, evidence, reentered));
    assert_eq!(
        readers.load(Ordering::SeqCst),
        1,
        "operational seed still holds its reader"
    );
    drop(seed);
    assert_eq!(
        readers.load(Ordering::SeqCst),
        0,
        "causal handles must not pin the admitted reader"
    );
    assert_eq!(causal.bridge_snapshot_identity(), Some(&source));
    assert_eq!(causal.semantic_observations().len(), 1);
    let envelope = envelope::assemble_and_check(&owner, &causal);
    assert_eq!(contacts.reads.load(Ordering::SeqCst), 1);
    assert_eq!(contacts.predicates.load(Ordering::SeqCst), 1);
    assert_eq!(contacts.computes.load(Ordering::SeqCst), 1);
    let retained = probe
        .bridge_conditional_retention()
        .unwrap()
        .retained_bytes();
    assert!(retained > bridge_baseline);
    drop(causal);
    assert_eq!(
        probe
            .bridge_conditional_retention()
            .unwrap()
            .retained_bytes(),
        retained
    );
    drop(last_causal);
    assert_eq!(
        probe
            .bridge_conditional_retention()
            .unwrap()
            .retained_bytes(),
        retained,
        "the explanation still retains the exact original observation"
    );
    let last_envelope = envelope.clone();
    drop(envelope);
    assert_eq!(
        probe
            .bridge_conditional_retention()
            .unwrap()
            .retained_bytes(),
        retained
    );
    drop(last_envelope);
    assert_eq!(
        probe
            .bridge_conditional_retention()
            .unwrap()
            .retained_bytes(),
        bridge_baseline
    );
    let signal_retained = probe
        .signal_conditional_retention()
        .unwrap()
        .retained_bytes();
    drop(signal);
    assert!(
        probe
            .signal_conditional_retention()
            .unwrap()
            .retained_bytes()
            < signal_retained,
        "the last readonly Signal share releases the original receipt reservation"
    );
    drop((basis, lowering, owner));
    assert!(probe.bridge_conditional_retention().is_none());
    assert_eq!(readers.load(Ordering::SeqCst), 0);
}

fn owner(
    contacts: Arc<Contacts>,
    readers: Arc<AtomicUsize>,
) -> (
    BridgeSealedRuntimeAssembly,
    Arc<BridgeInstalledConditionalLowering>,
) {
    let providers = Providers(Arc::clone(&contacts));
    let (mut builder, installation) = installation_fixture_with_runtime(
        super::super::super::semantic_dependencies::runtime_predicate_contract("query:one"),
        &["bridge-main"],
        BridgeConditionalProviderSet::new()
            .condition(providers.clone())
            .compute(providers),
        &[],
        |registrations| bridge_runtime(registrations, contacts, readers),
    );
    let lowering = builder.install(installation).unwrap();
    (builder.seal().unwrap(), lowering)
}

fn bridge_runtime(
    registrations: Vec<crate::correspondence::BridgeSemanticCorrespondenceRegistration>,
    contacts: Arc<Contacts>,
    readers: Arc<AtomicUsize>,
) -> crate::facade::RuntimeBridge {
    let mapping = super::super::super::exact_mapping();
    let aspect = super::super::super::aspect_mapping(&mapping);
    let builder = RuntimeBridgeBuilder::new()
        .with_committed_patch_source(super::super::super::TestSource)
        .with_snapshot_read_source(CountingSource {
            source: PacketSource {
                contacts,
                wrong_packet: false,
            },
            readers,
        })
        .with_signal_sink(super::super::super::TestSink)
        .register_mapping(mapping)
        .register_aspect_mapping(aspect);
    registrations
        .into_iter()
        .fold(builder, |builder, registration| {
            builder.register_semantic_correspondence(registration)
        })
        .build()
        .unwrap()
}

struct CountingSource {
    source: PacketSource,
    readers: Arc<AtomicUsize>,
}

impl SnapshotReadSource for CountingSource {
    fn open_snapshot(
        &self,
        _: &TruthSnapshotIdentity,
        _: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        self.readers.fetch_add(1, Ordering::SeqCst);
        Ok(Box::new(CountingReader {
            source: self.source.clone(),
            readers: Arc::clone(&self.readers),
        }))
    }
}

struct CountingReader {
    source: PacketSource,
    readers: Arc<AtomicUsize>,
}

impl TruthSnapshotReader for CountingReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        self.source.snapshot_identity()
    }
    fn read_packet(
        &self,
        packet: &SnapshotReadPacket,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        self.source.read_packet(packet, execution)
    }
}

impl Drop for CountingReader {
    fn drop(&mut self) {
        self.readers.fetch_sub(1, Ordering::SeqCst);
    }
}
