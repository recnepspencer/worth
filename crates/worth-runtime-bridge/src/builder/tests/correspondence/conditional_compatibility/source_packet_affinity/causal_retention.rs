//! Actual Native observations outlive operational evidence without pinning readers.
use super::*;
mod envelope;
mod history;
use crate::facade::{
    BridgeConditionalDecisionReentryRequest, BridgeInstalledConditionalLowering,
    BridgeSealedRuntimeAssembly,
};

#[test]
fn envelope_private_associations_retain_only_the_original_native_observation() {
    let contacts = Arc::new(Contacts::default());
    let readers = Arc::new(AtomicUsize::new(0));
    let (owner, lowering) = owner(contacts, Arc::clone(&readers));
    let probe = owner.conditional_lifecycle_probe();
    let baseline = probe
        .bridge_conditional_retention()
        .unwrap()
        .retained_bytes();
    let serial = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let source = crate::truth_identity_fixtures::truth_snapshot(1, 1);
    let basis = owner
        .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
        .unwrap();
    let mut evidence = owner
        .execute(
            worth_execution::ExecutionRequest::serial(&serial),
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
    let signal = causal.performed_signal_invalidation().unwrap().clone();
    let performed = evidence.take_performed_signal_invalidation().unwrap();
    assert!(signal.same_observation(&performed.retain_causal_observation()));
    assert!(
        signal.retains_executed_target(evidence.signal_graph_instance_id(), evidence.signal_node())
    );
    assert!(evidence.take_performed_signal_invalidation().is_none());
    drop((signal, performed));
    drop(evidence);
    assert_eq!(readers.load(Ordering::SeqCst), 0);
    let envelope = envelope::assemble_and_check(&owner, &causal);
    let retained = probe
        .bridge_conditional_retention()
        .unwrap()
        .retained_bytes();
    assert!(retained > baseline);
    drop(causal);
    let last = envelope.clone();
    drop(envelope);
    assert_eq!(
        probe
            .bridge_conditional_retention()
            .unwrap()
            .retained_bytes(),
        retained
    );
    drop(last);
    assert_eq!(
        probe
            .bridge_conditional_retention()
            .unwrap()
            .retained_bytes(),
        baseline
    );
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
