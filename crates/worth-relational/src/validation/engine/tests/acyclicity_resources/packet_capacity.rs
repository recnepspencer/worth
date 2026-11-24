//! Scratch and work limits at the actual native invariant packet boundary.
use super::super::validation_engine_fixtures::*;
use crate::branch::basis_admission_tests::installed_history;
use crate::config::data::RelationalRuntimeConfig;
use crate::execution::{execute_read_only_packets, PacketExecutionStop, ReadOnlyPacket};
use crate::history::data::coordinates::BranchId;
use crate::identity::data::{EntityId, VersionId};
use crate::runtime::state::subsystems::{
    history::HistorySubsystem,
    services::instrumentation::RuntimeInstrumentation,
    storage::{PartitionEditionCopyLane, StorageSubsystem},
};
use crate::transactions::data::aspect_field_patch::AspectFieldPatch;
use crate::transactions::data::primitives::EntityReference;
use crate::validation::engine::{
    test_view::invariant_view_with_custom_rules, InvariantRuntimeView,
};
use worth_execution::{CancellationToken, ExecutionResourceLease, LeaseRequest};
use worth_foundational::PartitionIdentity;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CallbackResult {
    Completed,
    WorkCeiling,
    OtherFailure,
}

fn run(
    fixture: &InvariantDag,
    forward_edges: usize,
    scratch: u64,
    work: u64,
) -> (Result<(), PacketExecutionStop>, u64, Option<CallbackResult>) {
    let lease = paid_dag_request(1024 * 1024, work, CancellationToken::new(), None);
    let plan = fixture.plan(forward_edges);
    let view = fixture.view();
    let request_view = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        &view,
        InvariantObservation::committed(fixture.storage.acquire()),
        fixture.version(),
        Some(&plan),
        Some(crate::validation::data::contracts::InvariantPlanContract::from_merged_plan(&plan)),
    );
    let prepared = crate::validation::execution::plan_invariant_execution(
        &view,
        worth_execution::ExecutionRequest::leased(&lease),
        &request_view,
        None,
        crate::validation::execution::InvariantStage::Native,
    );
    assert_eq!(
        prepared.packets.len(),
        1,
        "one declared acyclicity contract"
    );
    let packets = prepared.packets.into_iter().map(|packet| {
        assert!(matches!(&packet.registration,
            crate::authority::commit::preparation::packets::invariant::InvariantPacketRegistration::Native(
                crate::validation::data::catalog::InvariantRegistration {
                    rule: crate::validation::data::rules::InvariantRule::AcyclicityContract(_), ..
                }
            )
        ), "the actual installed rule must be acyclicity");
        ReadOnlyPacket {
            identity: PartitionIdentity::new(packet.packet_index as u64), value: packet,
            input_bytes: 0, memory: crate::execution::PacketMemory::Declared { scratch, result: 16 * 1024 },
        }
    }).collect();
    let callback = std::sync::Mutex::new(None);
    let (outcome, charges) = crate::execution::request_work::account_observation::observe(|| {
        execute_read_only_packets(
            packets,
            &lease,
            |packet, context| {
                let result = crate::validation::execution::evaluate_invariant_packet_checked(
                    &view,
                    worth_execution::ExecutionRequest::leased(&lease),
                    packet,
                    context,
                    lease.status(),
                );
                *callback.lock().unwrap() = Some(match &result {
                    Ok(_) => CallbackResult::Completed,
                    Err(worth_execution::MapKernelFailure::Stop(
                        worth_execution::MapKernelStop::WorkCeiling,
                    )) => CallbackResult::WorkCeiling,
                    Err(_) => CallbackResult::OtherFailure,
                });
                result
            },
            crate::validation::execution::InvariantWorkerEnvelope::owned_allocation_capacity_bytes,
        )
    });
    let outcome = outcome.map(|envelopes| {
        assert!(!envelopes.is_empty(), "the genuine rule must execute");
        assert!(envelopes
            .iter()
            .flat_map(|envelope| &envelope.results)
            .all(|result| matches!(
                result.result.verdict,
                crate::validation::data::execution::InvariantVerdict::Pass
            )));
    });
    assert_eq!(lease.charged_memory_bytes(), 0);
    (
        outcome,
        charges.into_iter().sum(),
        callback.into_inner().unwrap(),
    )
}

#[test]
fn scratch_admission_covers_all_simultaneous_native_buffers_and_convergent_targets() {
    let fixture = InvariantDag::without_custom_rules(32);
    let before = fixture.head();
    // Each prepared edge needs one borrowed pair and two potential vertices.
    // Both graph capacities coexist with a visited byte and frontier index for
    // every distinct vertex. Convergent paths do not create extra vertices.
    let edges: usize = (1..=3).map(|offset| fixture.nodes.len() - offset).sum();
    let extent = (edges
        * std::mem::size_of::<(
            &crate::transactions::data::primitives::EntityReference,
            &crate::transactions::data::primitives::EntityReference,
        )>()
        + 2 * edges
            * std::mem::size_of::<&crate::transactions::data::primitives::EntityReference>()
        + fixture.nodes.len() * (std::mem::size_of::<u8>() + std::mem::size_of::<usize>()))
        as u64;
    run(&fixture, 3, extent, 4_000_000)
        .0
        .expect("the entire declared live extent fits");
    let (failure, _, _) = run(&fixture, 3, extent - 1, 4_000_000);
    assert!(
        matches!(
            failure,
            Err(PacketExecutionStop::Execution {
                reason: worth_execution::MapStop::Failure {
                    cause: worth_execution::MapKernelFailure::Domain(
                        crate::execution::PacketBudgetDenial::ScratchCapacityExceeded
                    ),
                    ..
                },
                ..
            })
        ),
        "{failure:?}"
    );
    assert_eq!(fixture.head(), before);
}

#[test]
fn repeated_searches_keep_work_charged_and_stop_one_unit_below_observed_need() {
    let fixture = InvariantDag::without_custom_rules(32);
    let before = fixture.head();
    let (result, single_work, _) = run(&fixture, 1, 64 * 1024, 4_000_000);
    result.unwrap();
    let (result, repeated_work, callback) = run(&fixture, 3, 64 * 1024, 4_000_000);
    result.unwrap();
    assert!(
        matches!(callback, Some(CallbackResult::Completed)),
        "the checked evaluator completed: {callback:?}"
    );
    assert!(
        repeated_work > single_work,
        "convergent paths still cause real traversal work"
    );
    let (failure, _, callback) = run(&fixture, 3, 64 * 1024, repeated_work - 1);
    assert!(
        matches!(callback, Some(CallbackResult::WorkCeiling)),
        "the evaluator itself must retain its checkpoint failure: {callback:?}"
    );
    assert!(
        matches!(
            failure,
            Err(PacketExecutionStop::Execution {
                reason: worth_execution::MapStop::WorkExhausted { .. }
                    | worth_execution::MapStop::Failure {
                        cause: worth_execution::MapKernelFailure::Stop(
                            worth_execution::MapKernelStop::WorkCeiling
                        ),
                        ..
                    },
                ..
            })
        ),
        "{failure:?}"
    );
    assert_eq!(fixture.head(), before);
}

#[test]
fn caller_observed_scratch_need_grows_with_live_graph_not_repeated_visits() {
    let small = minimum_scratch(&InvariantDag::without_custom_rules(16));
    let large = minimum_scratch(&InvariantDag::without_custom_rules(64));
    assert!(large > small, "the larger live graph requires more scratch");
    // Four times the nodes, with headroom for endpoint rounding. Summing
    // freed buffers along every chain suffix grows quadratically instead.
    assert!(large <= small * 6, "scratch growth: {small} -> {large}");
}

fn minimum_scratch(fixture: &InvariantDag) -> u64 {
    let before = fixture.head();
    // The fixed caller allowance also funds result and framework storage;
    // the probe's initial scratch declaration must leave that storage room.
    let (mut refused, mut funded) = (0, 1024 * 1024 / 2);
    run(fixture, 2, funded, 4_000_000).0.unwrap();
    while refused + 1 < funded {
        let candidate = refused + (funded - refused) / 2;
        match run(fixture, 2, candidate, 4_000_000).0 {
            Ok(()) => funded = candidate,
            Err(PacketExecutionStop::Execution {
                reason:
                    worth_execution::MapStop::Failure {
                        cause:
                            worth_execution::MapKernelFailure::Domain(
                                crate::execution::PacketBudgetDenial::ScratchCapacityExceeded,
                            ),
                        ..
                    },
                ..
            }) => refused = candidate,
            Err(other) => {
                panic!("a different stop cannot stand in for scratch pressure: {other:?}")
            }
        }
    }
    assert_eq!(fixture.head(), before);
    funded
}

/// The native executor receives an installed Storage edition and a compiled
/// rule view. Packet-level tests own these inputs directly.
pub(super) struct InvariantDag {
    storage: StorageSubsystem,
    history: HistorySubsystem,
    instrumentation: RuntimeInstrumentation,
    config: RelationalRuntimeConfig,
    rule: Option<CustomInvariantRegistration>,
    pub(super) nodes: Vec<EntityId>,
}

impl InvariantDag {
    fn without_custom_rules(count: usize) -> Self {
        Self::new(count, None, None)
    }

    pub(super) fn new(
        count: usize,
        catalog: Option<InvariantCatalog>,
        rule: Option<CustomInvariantRegistration>,
    ) -> Self {
        let partition = PartitionId::main();
        let (state, nodes, _) =
            crate::storage::overlay::small_relation_partition::typed_relation_partition_for_test(
                partition,
                std::iter::repeat_n(KindId(3), count),
                &[],
            );
        let storage = StorageSubsystem::default();
        storage.install_owned(
            &Default::default(),
            PartitionEditionCopyLane::Ordinary,
            [(partition, state)],
        );
        let history = installed_history(1);
        history.install_recovered_sequence_floor(2, 2);
        let mut config = RelationalRuntimeConfig::default();
        config.schema.registry = acyclicity_and_connectivity_registry();
        if let Some(catalog) = catalog {
            config.schema.invariant_catalog = catalog;
        }
        Self {
            storage,
            history,
            instrumentation: RuntimeInstrumentation::default(),
            config,
            rule,
            nodes,
        }
    }

    pub(super) fn head(
        &self,
    ) -> crate::branch::reference::observation::RelationalBranchReferenceObservation {
        self.history
            .branch_cell(&BranchId("main".into()))
            .unwrap()
            .observation()
    }

    pub(super) fn version(&self) -> VersionId {
        self.history.current_version_id()
    }

    pub(super) fn observation(&self) -> InvariantObservation<'_> {
        InvariantObservation::committed(self.storage.acquire())
    }

    pub(super) fn view(&self) -> InvariantRuntimeView<'_> {
        invariant_view_with_custom_rules(
            &self.instrumentation,
            self.config.clone(),
            self.version(),
            self.rule.clone().into_iter().collect(),
        )
    }

    pub(super) fn plan(&self, forward_edges: usize) -> MergedCommitPlan {
        let mut merged_intents = Vec::new();
        for (index, &source) in self.nodes.iter().enumerate() {
            for (offset, &target) in self
                .nodes
                .iter()
                .skip(index + 1)
                .take(forward_edges)
                .enumerate()
            {
                merged_intents.push(edge(source, target, format!("dag-edge-{index}-{offset}")));
            }
        }
        MergedCommitPlan {
            transaction_id: TransactionId(2),
            merged_intents,
        }
    }
}

pub(super) fn edge(
    source: crate::identity::data::EntityId,
    target: crate::identity::data::EntityId,
    key: String,
) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Relation(RelationSpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(2),
        client_key: ClientKey::raw(key),
        source: EntityReference::Existing(source),
        target: EntityReference::Existing(target),
        fields: AspectFieldPatch::default(),
    }))
}

pub(super) fn paid_dag_request(
    bytes: u64,
    work: u64,
    cancellation: CancellationToken,
    deadline: Option<std::time::Instant>,
) -> ExecutionResourceLease<'static> {
    worth_harness::facade::execution_fixture::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(bytes), Some(work)),
            ),
            cancellation,
            deadline: deadline
                .map(|instant| worth_execution::MonotonicClock::system().deadline_at(instant)),
        })
        .unwrap()
}
