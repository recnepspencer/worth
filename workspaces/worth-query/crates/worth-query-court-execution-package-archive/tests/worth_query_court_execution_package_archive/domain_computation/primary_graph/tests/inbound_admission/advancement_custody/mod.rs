//! The receive entry must refuse before contacting either its verifier or its reader.
use crate::domain_computation::primary_graph::tests::inbound_admission::fixture::installation::installed_world_with_verifier;
use crate::domain_computation::primary_graph::tests::inbound_admission::verifier::signed_envelope;
use std::num::NonZeroUsize;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial as Denial;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
use worth_query_execution::facade::domain_computation::advancement_requests_on_this_thread_for_test as reports;
use worth_query_execution::facade::domain_computation::bound_advancement_requests_on_this_thread_for_test as bound;
use worth_query_execution::facade::domain_computation::installed_source_reads_on_this_thread_for_test as reads;
use worth_query_execution::facade::domain_computation::place_managed_computations_on_this_thread_for_test as place;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionPlacementForTest as Placement;
use worth_query_execution::facade::domain_computation::WorthQueryInboundAdmissionDenial;
use worth_query_execution::facade::domain_computation::WorthQueryInboundOccurrenceClaims;
use worth_query_execution::facade::domain_computation::WorthQueryInboundOccurrenceVerifier;
use worth_query_execution::facade::domain_computation::WorthQueryInboundVerificationDenial;

pub struct CountedVerifier(AtomicUsize);
impl WorthQueryInboundOccurrenceVerifier for CountedVerifier {
    fn audience(&self) -> &str {
        self.0.fetch_add(1, Ordering::Relaxed);
        crate::domain_computation::primary_graph::tests::inbound_admission::verifier::TestVerifier
            .audience()
    }
    fn source_identity(&self) -> &str {
        self.0.fetch_add(1, Ordering::Relaxed);
        crate::domain_computation::primary_graph::tests::inbound_admission::verifier::TestVerifier
            .source_identity()
    }
    fn protocol_identity(&self) -> &worth_foundational::facade::BoundaryProtocolIdentity {
        self.0.fetch_add(1, Ordering::Relaxed);
        crate::domain_computation::primary_graph::tests::inbound_admission::verifier::TestVerifier
            .protocol_identity()
    }
    fn protocol_version(&self) -> worth_foundational::facade::BoundaryProtocolVersion {
        self.0.fetch_add(1, Ordering::Relaxed);
        crate::domain_computation::primary_graph::tests::inbound_admission::verifier::TestVerifier
            .protocol_version()
    }
    fn verify(
        &self,
        payload: &[u8],
        now: u64,
        identity: std::num::NonZeroU64,
    ) -> Result<WorthQueryInboundOccurrenceClaims, WorthQueryInboundVerificationDenial> {
        self.0.fetch_add(1, Ordering::Relaxed);
        crate::domain_computation::primary_graph::tests::inbound_admission::verifier::TestVerifier
            .verify(payload, now, identity)
    }
}
pub struct Restore(Placement, Option<worth_foundational::ExecutionBudget>);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}

#[test]
fn inbound_receive_refuses_before_authentication_or_correlation_reads() {
    let _observation =
        worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let verifier = std::sync::Arc::new(CountedVerifier(AtomicUsize::new(0)));
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _placement = Restore(place(placement), bound(None));
        // A fresh correlation in each placement exercises the real source read.
        // An already completed duplicate legitimately returns retained terminal data.
        let world = installed_world_with_verifier(verifier.clone());
        let dispatch = { world.commit_dispatch(147, "custody-probe") };
        let record = dispatch.dispatch_outbox().unwrap();
        let envelope = signed_envelope(record, [0x97; 32], record.payload(), false);
        verifier.0.store(0, Ordering::Relaxed);
        let before = reads();
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &envelope, &request)
            .unwrap();
        assert!(verifier.0.load(Ordering::Relaxed) > 0);
        assert!(
            reads() > before,
            "admitted receive enters the same source reader"
        );
        verifier.0.store(0, Ordering::Relaxed);
        for zero_memory in [false, true] {
            let _restore = Restore(
                place(placement),
                bound(Some(worth_foundational::ExecutionBudget::new(
                    NonZeroUsize::MIN,
                    if zero_memory { Some(0) } else { None },
                    Some(if zero_memory { 8_000_000 } else { 0 }),
                ))),
            );
            let before = reads();
            reports();
            let denial = world
                .application
                .receive_inbound_occurrence(&world.verifier, &envelope, &request)
                .err()
                .expect("the host's request is refused");
            let WorthQueryInboundAdmissionDenial::PublicationExecutionDenied {
                stage,
                kind:
                    worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionResource {
                        denial: resource,
                        partition_identity,
                        policy_ancestor,
                    },
            } = denial
            else {
                panic!("receive must retain its admission cause: {denial:?}");
            };
            assert_eq!(stage, worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage::ResourceAdmission);
            assert_eq!(partition_identity, None);
            assert_eq!(policy_ancestor, None);
            let cause = Denial::Resource(resource);
            if !zero_memory {
                assert_eq!(cause, Denial::Resource(Resource::WorkExhausted));
            } else {
                match placement {
                    Placement::World | Placement::Certified { .. } => {
                        unreachable!("this probe declares its placement")
                    }
                    Placement::Serial | Placement::Leased(_) => {
                        let Denial::Resource(Resource::MemoryLimit {
                            level,
                            requested,
                            admitted,
                        }) = cause
                        else {
                            panic!("the exact lease refusal is retained: {cause:?}");
                        };
                        assert_eq!(level, worth_query_execution::facade::application_contribution::WorthQueryMemoryLimitLevel::Policy);
                        assert!(requested > 0);
                        assert_eq!(admitted, 0);
                    }
                }
            }
            assert_eq!(verifier.0.load(Ordering::Relaxed), 0);
            assert_eq!(reads(), before);
            assert_eq!(reports(), vec![Err(cause)]);
        }
    }
}
