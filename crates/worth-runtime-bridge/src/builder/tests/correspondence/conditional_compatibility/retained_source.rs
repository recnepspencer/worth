use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use super::{always_eligible_contract, installation_fixture_with_runtime, Compute};
use crate::facade::{
    BridgeConditionalDecisionReentryRequest, BridgeConditionalDenialKind,
    BridgeConditionalExecutionRequest, BridgeConditionalProviderSet, RelationalBridgeSourceError,
    RuntimeBridgeBuilder, SnapshotReadSource, SnapshotReaderPool, TruthSnapshotIdentity,
    TruthSnapshotReader,
};

#[derive(Default)]
struct ReaderCounts {
    live: AtomicUsize,
    opened: AtomicUsize,
    dropped: AtomicUsize,
    returned: AtomicUsize,
}

struct SingleReaderSource(Arc<ReaderCounts>);
struct CountedReader(Arc<ReaderCounts>);

struct ReusableReaderSource(Arc<AtomicUsize>);

struct IdentityReader {
    identity: TruthSnapshotIdentity,
    value: &'static str,
}

impl SnapshotReadSource for ReusableReaderSource {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        let value = if identity == &crate::truth_identity_fixtures::truth_snapshot(1, 1) {
            "B"
        } else {
            "A"
        };
        Ok(Box::new(IdentityReader {
            identity: identity.clone(),
            value,
        }))
    }
}

impl TruthSnapshotReader for IdentityReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        self.identity.clone()
    }

    fn read_packet(
        &self,
        packet: &crate::facade::SnapshotReadPacket,
    ) -> Result<crate::facade::SnapshotReadPacketResult, crate::facade::BridgeSnapshotReadError>
    {
        Ok(crate::facade::SnapshotReadPacketResult::new(
            self.identity.clone(),
            packet
                .reads()
                .iter()
                .map(|request| {
                    crate::facade::SnapshotReadRecord::for_request(
                        request,
                        worth_foundational::facade::StructAspectValue::new([(
                            worth_foundational::facade::FieldKey::new("name").unwrap(),
                            worth_foundational::facade::AspectValue::String(self.value.into()),
                        )])
                        .unwrap(),
                    )
                })
                .collect(),
        ))
    }
}

#[derive(Clone, Default)]
struct SessionObservationPredicate(Arc<std::sync::Mutex<Vec<(Option<String>, String)>>>);

impl crate::facade::BridgeConditionalProviderSemantics for SessionObservationPredicate {
    type SemanticContract = &'static str;

    fn semantic_contract(&self) -> Self::SemanticContract {
        "session-local-observation-baseline"
    }

    fn retained_heap_bytes(
        &self,
        _: &Self::SemanticContract,
    ) -> Result<
        crate::facade::BridgeConditionalProviderHeapRetention,
        crate::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        let observations = self.0.lock().unwrap();
        let vector =
            (observations.capacity() * std::mem::size_of::<(Option<String>, String)>()) as u64;
        let strings = observations
            .iter()
            .try_fold(0u64, |total, (branch, snapshot)| {
                total
                    .checked_add(branch.as_ref().map_or(0, String::capacity) as u64)
                    .and_then(|total| total.checked_add(snapshot.capacity() as u64))
                    .ok_or(crate::facade::BridgeConditionalProviderRetentionOverflow)
            })?;
        crate::facade::BridgeConditionalProviderHeapRetention::try_from_parts(
            [
                crate::facade::BridgeConditionalProviderHeapRetention::arc_allocation_bytes(
                    self.0.as_ref(),
                ),
                vector,
                strings,
            ],
            [],
        )
    }
}

impl crate::facade::BridgeConditionalConditionProvider for SessionObservationPredicate {
    fn resolve(
        &self,
        context: crate::facade::BridgeConditionalResolverContext,
    ) -> Result<worth_signal::facade::InstalledSignalConditionDecision, String> {
        use worth_foundational::facade::{AspectValue, ContractValidatedAspectValueView};
        let observation = context
            .observation(0)
            .ok_or_else(|| "missing runtime-predicate observation".to_string())?;
        let read =
            |artifact: &worth_foundational::facade::ContractValidatedAspectArtifact| match artifact
                .payload()
                .view()
            {
                ContractValidatedAspectValueView::Struct(value) => {
                    match value.get(&worth_foundational::facade::FieldKey::new("name").unwrap()) {
                        Some(AspectValue::String(
                            worth_foundational::facade::InternedString::Raw(value),
                        )) => Ok(value.clone()),
                        _ => Err("runtime-predicate name field was not a string".to_string()),
                    }
                }
                _ => Err("runtime-predicate observation was not a string".to_string()),
            };
        let previous = observation.previous().map(read).transpose()?;
        let current = read(
            observation
                .current()
                .ok_or_else(|| "runtime-predicate current observation was absent".to_string())?,
        )?;
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push((previous, current));
        Ok(worth_signal::facade::InstalledSignalConditionDecision::Eligible)
    }
}

impl SnapshotReadSource for SingleReaderSource {
    fn open_snapshot(
        &self,
        _: &TruthSnapshotIdentity,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        if self
            .0
            .live
            .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err(RelationalBridgeSourceError::new(
                "one retained reader exhausts source capacity",
            ));
        }
        self.0.opened.fetch_add(1, Ordering::SeqCst);
        Ok(Box::new(CountedReader(Arc::clone(&self.0))))
    }
}

impl SnapshotReaderPool for SingleReaderSource {
    fn acquire(
        &self,
        identity: &TruthSnapshotIdentity,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        self.open_snapshot(identity)
    }

    fn release(&self, reader: Box<dyn TruthSnapshotReader>) {
        self.0.returned.fetch_add(1, Ordering::SeqCst);
        drop(reader);
    }
}

impl TruthSnapshotReader for CountedReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        crate::truth_identity_fixtures::truth_snapshot(1, 1)
    }

    fn read_packet(
        &self,
        _: &crate::facade::SnapshotReadPacket,
    ) -> Result<crate::facade::SnapshotReadPacketResult, crate::facade::BridgeSnapshotReadError>
    {
        unreachable!("Always execution retains a source without semantic condition reads")
    }
}

impl Drop for CountedReader {
    fn drop(&mut self) {
        assert_eq!(self.0.live.fetch_sub(1, Ordering::SeqCst), 1);
        self.0.dropped.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn retained_decisions_keep_exact_reader_and_pool_custody_without_reacquisition() {
    for pooled in [false, true] {
        // Explicit one-reader adapter limit exercises observable backpressure.
        // This is Bridge reader/pool custody, not a source-owner lease proof.
        let counts = Arc::new(ReaderCounts::default());
        let (mut owner, installation) = installation_fixture_with_runtime(
            always_eligible_contract("query:one"),
            &["bridge-main"],
            BridgeConditionalProviderSet::new().compute(Compute(1)),
            &[],
            |registrations| {
                let mapping = super::super::exact_mapping();
                let aspect = super::super::aspect_mapping(&mapping);
                let mut builder = RuntimeBridgeBuilder::new()
                    .with_committed_patch_source(super::super::TestSource)
                    .with_snapshot_read_source(SingleReaderSource(Arc::clone(&counts)))
                    .with_signal_sink(super::super::TestSink)
                    .register_mapping(mapping)
                    .register_aspect_mapping(aspect);
                if pooled {
                    builder =
                        builder.with_snapshot_reader_pool(SingleReaderSource(Arc::clone(&counts)));
                }
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
        let signal_basis = owner
            .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
            .unwrap();
        let source = crate::truth_identity_fixtures::truth_snapshot(1, 1);
        let request = || BridgeConditionalExecutionRequest {
            lowering: &lowering,
            query_binding_identity: "query-binding",
            query_capability_identity: 1,
            snapshot_identity: "snapshot-label",
            truth_branch_identity: None,
            bridge_snapshot_identity: Some(&source),
            execution_identity: "execution",
            attempt: 1,
        };
        let evidence = owner.execute(&signal_basis, request(), &mut ()).unwrap();
        let seed = evidence.retain_for_reentry();
        drop(evidence);
        assert_eq!(counts.live.load(Ordering::SeqCst), 1);
        let reentered = owner
            .reenter_retained_conditional_decision(BridgeConditionalDecisionReentryRequest {
                seed: &seed,
                lowering: &lowering,
                query_binding_identity: "reentered",
                query_capability_identity: 2,
                snapshot_identity: "snapshot-label",
                bridge_snapshot_identity: Some(&source),
            })
            .unwrap();
        drop(seed);
        assert_eq!(
            counts.opened.load(Ordering::SeqCst),
            1,
            "reentry never reopens a reader"
        );
        assert!(reentered.retains_bridge_snapshot_identity(&source));
        let exhausted = owner
            .execute(&signal_basis, request(), &mut ())
            .err()
            .expect("retained reader still occupies capacity");
        assert_eq!(
            exhausted.kind(),
            BridgeConditionalDenialKind::SnapshotAdmission
        );
        assert_eq!(
            exhausted
                .bridge_execution_counters()
                .signal_execution_contacts,
            0
        );
        assert_eq!(counts.dropped.load(Ordering::SeqCst), 0);
        assert_eq!(counts.returned.load(Ordering::SeqCst), 0);
        drop(reentered);
        assert_eq!(counts.live.load(Ordering::SeqCst), 0);
        assert_eq!(counts.dropped.load(Ordering::SeqCst), 1);
        assert_eq!(counts.returned.load(Ordering::SeqCst), usize::from(pooled));
        let final_evidence = owner.execute(&signal_basis, request(), &mut ()).unwrap();
        assert_eq!(counts.opened.load(Ordering::SeqCst), 2);
        drop(owner);
        assert_eq!(
            counts.live.load(Ordering::SeqCst),
            1,
            "evidence outlives the Bridge runtime"
        );
        drop(final_evidence);
        assert_eq!(counts.live.load(Ordering::SeqCst), 0);
        assert_eq!(counts.dropped.load(Ordering::SeqCst), 2);
        assert_eq!(
            counts.returned.load(Ordering::SeqCst),
            2 * usize::from(pooled)
        );
    }
}

mod session_isolation;
