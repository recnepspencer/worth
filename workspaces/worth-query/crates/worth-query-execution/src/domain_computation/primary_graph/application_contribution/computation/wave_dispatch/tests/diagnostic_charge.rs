//! Certification is a diagnostic rerun, never part of an advancement's bill.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::request_execution::{
    place_managed_computations_on_this_thread_for_test as place,
    WorthQueryExecutionPlacementForTest as Placement,
};

fn reducing_member() -> PreparedMember {
    PreparedMember {
        requests: vec![PreparedCompute::new(
            0,
            0,
            |member| {
                member.compute(|managed, _, _| {
                let map = ExecutionMap::<_, u64>::from_keyless_partitions(
                    [3_u64, 7].into_iter().enumerate().map(|(rank, value)| (
                        PartitionIdentity::new(rank as u64 + 1),
                        worth_execution::KeylessPartition {
                            value, kernel_scratch_bytes: 0, max_result_bytes: 0,
                        },
                    )).collect(),
                ).unwrap();
                let inputs = managed.request.reserve(0).unwrap();
                let mut held = managed.request.reserve(0).unwrap();
                let (tree, report, _) = managed.request.dispatch().unwrap().reduce(
                    u64::MAX, &map, inputs, &mut held,
                    |value, context| {
                        context.checkpoint(*value)?;
                        Ok::<_, MapKernelFailure<()>>(*value)
                    }, 0_u64, |left, right| left + right, 8,
                    crate::domain_computation::primary_graph::application_contribution::partitioned_computation::WorthQueryPartitionedComputationFullCause::Unretained,
                ).unwrap().unwrap();
                Ok::<_, ()>((*tree.result(), report.charged_work(), report.charged_span()))
            })
            },
            |_| 0,
        )],
    }
}

#[test]
fn certification_never_changes_the_advancements_work_or_span() {
    let _observation = crate::domain_computation::execution_observation::WorthQueryExecutionObservation::install_on_this_thread();
    let mut bills = Vec::new();
    for placement in [
        Placement::Leased(NonZeroUsize::MIN),
        Placement::Certified {
            workers: NonZeroUsize::MIN,
            seed: 1,
        },
    ] {
        place(placement);
        let (settled, report) = run(Some(1), vec![reducing_member()]);
        let value = settled
            .into_iter()
            .next()
            .unwrap()
            .unwrap()
            .into_slots()
            .pop()
            .unwrap()
            .into_value::<Result<(u64, u64, u64), ()>>()
            .unwrap();
        assert_eq!(value.0, 10);
        bills.push((value, report.charged_work(), report.charged_span()));
    }
    assert_eq!(
        bills[0], bills[1],
        "a diagnostic certification never changes the advancement's work or span"
    );
}
