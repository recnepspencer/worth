//! Scratch and work limits at the actual native invariant packet boundary.
use super::super::validation_engine_fixtures::*;
use super::fixture::*;
use crate::execution::{execute_read_only_packets, PacketExecutionStop, ReadOnlyPacket};
use crate::validation::engine::InvariantRuntimeView;
use worth_execution::CancellationToken;
use worth_foundational::PartitionIdentity;

fn run(
    fixture: &DagFixture,
    forward_edges: usize,
    scratch: u64,
    work: u64,
) -> (Result<(), PacketExecutionStop>, u64) {
    let lease = request(1024 * 1024, work, CancellationToken::new(), None);
    let plan = fixture.merged_plan(fixture.chain_batch(forward_edges));
    let runtime = &fixture.runtime;
    let request_view = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        runtime,
        InvariantObservation::committed(runtime.storage_access().current_edition()),
        runtime.current_version_id(),
        Some(&plan),
        Some(crate::validation::data::InvariantPlanContract::from_merged_plan(&plan)),
    );
    let view = InvariantRuntimeView::from_runtime(runtime);
    let prepared = crate::validation::execution::plan_invariant_execution(
        &view,
        worth_execution::ExecutionRequest::leased(&lease),
        &request_view,
        None,
    );
    let packets = prepared.packets.into_iter().map(|packet| {
        assert!(matches!(&packet.registration,
            crate::authority::commit::preparation::packets::invariant::InvariantPacketRegistration::Native(
                crate::validation::data::InvariantRegistration {
                    rule: crate::validation::data::InvariantRule::AcyclicityContract(_), ..
                }
            )
        ), "the actual installed rule must be acyclicity");
        ReadOnlyPacket {
            identity: PartitionIdentity::new(packet.packet_index as u64), value: packet,
            input_bytes: 0, memory: crate::execution::PacketMemory::Declared { scratch: scratch, result: 16 * 1024 },
        }
    }).collect();
    let (outcome, charges) = crate::execution::work_observation::capture(|| {
        execute_read_only_packets(
            packets,
            &lease,
            |packet, context| {
                crate::validation::execution::evaluate_invariant_packet_checked(
                    &view,
                    worth_execution::ExecutionRequest::leased(&lease),
                    packet,
                    context,
                    lease.status(),
                )
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
                crate::validation::data::InvariantVerdict::Pass
            )));
    });
    assert_eq!(lease.charged_memory_bytes(), 0);
    (outcome, charges.into_iter().sum())
}

#[test]
fn scratch_admission_covers_all_simultaneous_native_buffers_and_convergent_targets() {
    let fixture = DagFixture::new(32);
    let before = fixture.head();
    // Each prepared edge needs one borrowed pair and two potential vertices.
    // Both graph capacities coexist with a visited byte and frontier index for
    // every distinct vertex. Convergent paths do not create extra vertices.
    let edges: usize = (1..=3).map(|offset| fixture.nodes.len() - offset).sum();
    let extent = (edges
        * std::mem::size_of::<(
            &crate::transactions::data::EntityReference,
            &crate::transactions::data::EntityReference,
        )>()
        + 2 * edges * std::mem::size_of::<&crate::transactions::data::EntityReference>()
        + fixture.nodes.len() * (std::mem::size_of::<u8>() + std::mem::size_of::<usize>()))
        as u64;
    run(&fixture, 3, extent, 4_000_000)
        .0
        .expect("the entire declared live extent fits");
    let (failure, _) = run(&fixture, 3, extent - 1, 4_000_000);
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
    let fixture = DagFixture::new(32);
    let before = fixture.head();
    let (result, single_work) = run(&fixture, 1, 64 * 1024, 4_000_000);
    result.unwrap();
    let (result, repeated_work) = run(&fixture, 3, 64 * 1024, 4_000_000);
    result.unwrap();
    assert!(
        repeated_work > single_work,
        "convergent paths still cause real traversal work"
    );
    let (failure, _) = run(&fixture, 3, 64 * 1024, repeated_work - 1);
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
    let small = minimum_scratch(&DagFixture::new(16));
    let large = minimum_scratch(&DagFixture::new(64));
    assert!(large > small, "the larger live graph requires more scratch");
    // Four times the nodes, with headroom for endpoint rounding. Summing
    // freed buffers along every chain suffix grows quadratically instead.
    assert!(large <= small * 6, "scratch growth: {small} -> {large}");
}

fn minimum_scratch(fixture: &DagFixture) -> u64 {
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
