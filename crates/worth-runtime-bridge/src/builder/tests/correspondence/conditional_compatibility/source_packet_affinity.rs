use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use super::installation_fixture_with_runtime;
use crate::facade::{
    BridgeConditionalComputeProvider, BridgeConditionalConditionProvider,
    BridgeConditionalDenialKind, BridgeConditionalExecutionRequest,
    BridgeConditionalProviderSemantics, BridgeConditionalProviderSet,
    BridgeConditionalResolverContext, BridgeSnapshotReadError, RelationalBridgeSourceError,
    RuntimeBridgeBuilder, SnapshotReadPacket, SnapshotReadPacketResult, SnapshotReadRecord,
    SnapshotReadSource, TruthSnapshotIdentity, TruthSnapshotReader,
};

mod context_retention;

#[derive(Default)]
struct Contacts {
    reads: AtomicUsize,
    predicates: AtomicUsize,
    computes: AtomicUsize,
}

#[derive(Clone)]
struct PacketSource {
    contacts: Arc<Contacts>,
    wrong_packet: bool,
}

impl SnapshotReadSource for PacketSource {
    fn open_snapshot(
        &self,
        _: &TruthSnapshotIdentity,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        Ok(Box::new(self.clone()))
    }
}

impl TruthSnapshotReader for PacketSource {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        crate::truth_identity_fixtures::truth_snapshot(1, 1)
    }

    fn read_packet(
        &self,
        packet: &SnapshotReadPacket,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        self.contacts.reads.fetch_add(1, Ordering::SeqCst);
        // Explicit adapter fault: correct record correlations and authoritative
        // absence postures, but a packet from a different selected snapshot.
        Ok(SnapshotReadPacketResult::new(
            crate::truth_identity_fixtures::truth_snapshot(
                1,
                if self.wrong_packet { 2 } else { 1 },
            ),
            packet
                .reads()
                .iter()
                .map(SnapshotReadRecord::absent_for_request)
                .collect(),
        ))
    }
}

#[derive(Clone)]
struct Providers(Arc<Contacts>);

impl BridgeConditionalProviderSemantics for Providers {
    type SemanticContract = ();
    fn semantic_contract(&self) {}

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        crate::facade::BridgeConditionalProviderHeapRetention,
        crate::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        crate::facade::BridgeConditionalProviderHeapRetention::try_from_parts(
            [
                crate::facade::BridgeConditionalProviderHeapRetention::arc_allocation_bytes(
                    self.0.as_ref(),
                ),
            ],
            [],
        )
    }
}

impl BridgeConditionalConditionProvider for Providers {
    fn resolve(
        &self,
        context: BridgeConditionalResolverContext,
    ) -> Result<worth_signal::facade::InstalledSignalConditionDecision, String> {
        self.0.predicates.fetch_add(1, Ordering::SeqCst);
        assert_eq!(context.observations().len(), 1);
        assert!(context.observations()[0].current().is_none());
        Ok(worth_signal::facade::InstalledSignalConditionDecision::Eligible)
    }
}

impl BridgeConditionalComputeProvider for Providers {
    fn compute(
        &self,
        _: &mut dyn std::any::Any,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        self.0.computes.fetch_add(1, Ordering::SeqCst);
        Ok(worth_signal::facade::NodeEvaluationResult::from_version(
            worth_signal::facade::AspectVersion::from_updates([(
                worth_signal::facade::Aspect::new(0),
                1,
            )]),
        ))
    }
}

#[test]
fn conditional_packet_source_must_match_admitted_reader_before_predicate_or_compute() {
    for wrong_packet in [true, false] {
        let contacts = Arc::new(Contacts::default());
        let providers = Providers(Arc::clone(&contacts));
        let (mut owner, installation) = installation_fixture_with_runtime(
            super::super::semantic_dependencies::runtime_predicate_contract("query:one"),
            &["bridge-main"],
            BridgeConditionalProviderSet::new()
                .condition(providers.clone())
                .compute(providers),
            &[],
            |registrations| {
                let mapping = super::super::exact_mapping();
                let aspect = super::super::aspect_mapping(&mapping);
                let builder = RuntimeBridgeBuilder::new()
                    .with_committed_patch_source(super::super::TestSource)
                    .with_snapshot_read_source(PacketSource {
                        contacts: Arc::clone(&contacts),
                        wrong_packet,
                    })
                    .with_signal_sink(super::super::TestSink)
                    .register_mapping(mapping)
                    .register_aspect_mapping(aspect);
                registrations
                    .into_iter()
                    .fold(builder, |builder, registration| {
                        builder.register_semantic_correspondence(registration)
                    })
                    .build()
                    .unwrap()
            },
        );
        let lowering = owner.install(installation).unwrap();
        let owner = owner.seal().unwrap();
        let source = crate::truth_identity_fixtures::truth_snapshot(1, 1);
        let signal_basis = owner
            .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
            .unwrap();
        let result = owner.execute(
            &signal_basis,
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
        );
        assert_eq!(
            contacts.reads.load(Ordering::SeqCst),
            1,
            "the source read was performed"
        );
        if wrong_packet {
            let denial = result.err().expect("foreign packet must be denied");
            assert_eq!(denial.kind(), BridgeConditionalDenialKind::SnapshotMismatch);
            assert_eq!(contacts.predicates.load(Ordering::SeqCst), 0);
            assert_eq!(contacts.computes.load(Ordering::SeqCst), 0);
            assert_eq!(
                denial.bridge_execution_counters().signal_execution_contacts,
                1
            );
            assert_eq!(
                denial
                    .bridge_execution_counters()
                    .observation_baseline_writes,
                0
            );
            assert_eq!(denial.bridge_execution_counters().decisions_retained, 0);
            assert_eq!(denial.signal_counters().compute_contacts, 0);
        } else {
            let evidence = result.unwrap();
            assert_eq!(contacts.predicates.load(Ordering::SeqCst), 1);
            assert_eq!(contacts.computes.load(Ordering::SeqCst), 1);
            assert_eq!(evidence.semantic_observation_reads(), 1);
            assert_eq!(
                evidence
                    .bridge_execution_counters()
                    .observation_baseline_writes,
                1
            );
            assert!(evidence.retains_bridge_snapshot_identity(&source));
            assert!(evidence.performed_signal_invalidation().is_some());
        }
    }
}
