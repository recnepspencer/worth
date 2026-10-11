//! Each truth seam must refuse without returning a partial row set.
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::facade::{config::CascadeDeletePolicy, identity::KindId};
use crate::tests::support::*;
use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionWorkCeiling, SerialMemoryBudget, SerialRequest,
};

#[test]
fn truth_seams_spend_the_callers_work_and_preserve_interruption() {
    let setup = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&setup);
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let first = create_entity_outcome(&execution, &runtime, "truth-paid-first");
    let source = changed_entities(&first)[0];
    release_test_commit_snapshot(&runtime, &first);
    let second = create_entity_outcome(&execution, &runtime, "truth-paid-second");
    let target = changed_entities(&second)[0];
    release_test_commit_snapshot(&runtime, &second);
    let edge = create_relation_outcome(&execution, &runtime, source, target, "truth-paid-edge");
    let relation = changed_relations(&edge)[0];
    let version = runtime.current_version_id();
    let truth = runtime.read_truth();
    // The fixture has two entity slots and one relation slot. Kind and
    // adjacency scans spend one unit examining and one reserving each row;
    // keyed reads spend one unit before materializing their record.
    for (seam, units, rows) in [
        (0, 1, 1),
        (1, 1, 1),
        (2, 4, 2),
        (3, 4, 2),
        (4, 2, 1),
        (5, 2, 1),
        (6, 2, 1),
        (7, 2, 1),
        (8, 4, 2),
        (9, 2, 1),
        (10, 2, 1),
        (11, 2, 1),
    ] {
        let read = |request: &ExecutionRequest<'_, '_>| -> Result<usize, Cause> {
            match seam {
                0 => truth
                    .visible_entity_at_version(request, source, version)
                    .map(|row| usize::from(row.is_some())),
                1 => truth
                    .visible_relation_at_version(request, relation, version)
                    .map(|row| usize::from(row.is_some())),
                2 => truth
                    .visible_entities_of_kind(request, KindId(1), version)
                    .map(|rows| rows.len()),
                3 => truth
                    .visible_entities_of_kind_in_partition(
                        request,
                        source.partition_id,
                        KindId(1),
                        version,
                    )
                    .map(|rows| rows.len()),
                4 => truth
                    .visible_relations_of_kind(request, KindId(2), version)
                    .map(|rows| rows.len()),
                5 => truth
                    .visible_relations_of_kind_in_partition(
                        request,
                        source.partition_id,
                        KindId(2),
                        version,
                    )
                    .map(|rows| rows.len()),
                6 => truth
                    .outgoing_relations_of_kind_at_version(request, source, KindId(2), version)
                    .map(|rows| rows.len()),
                7 => truth
                    .incoming_relations_of_kind_at_version(request, target, KindId(2), version)
                    .map(|rows| rows.len()),
                8 => truth
                    .bounded_visible_entities_of_kind(request, KindId(1), version, 16)
                    .map(|read| read.unwrap().into_records().len()),
                9 => truth
                    .bounded_visible_relations_of_kind(request, KindId(2), version, 16)
                    .map(|read| read.unwrap().into_records().len()),
                10 => truth
                    .bounded_outgoing_relations_of_kind_at_version(
                        request,
                        source,
                        KindId(2),
                        version,
                        16,
                    )
                    .map(|read| read.unwrap().into_records().len()),
                11 => truth
                    .bounded_incoming_relations_of_kind_at_version(
                        request,
                        target,
                        KindId(2),
                        version,
                        16,
                    )
                    .map(|read| read.unwrap().into_records().len()),
                _ => unreachable!(),
            }
        };
        let live = crate::test_execution_policy::serial_request();
        let request = ExecutionRequest::serial(&live);
        let (answer, report) = request
            .run(ExecutionWorkCeiling::new(units), |_| read(&request))
            .unwrap();
        assert_eq!(answer, Ok(rows), "seam {seam} exact budget");
        assert_eq!(report.charged_work(), units, "seam {seam} payer");
        let short = crate::test_execution_policy::serial_request();
        let request = ExecutionRequest::serial(&short);
        assert_eq!(
            request
                .run(ExecutionWorkCeiling::new(units - 1), |_| read(&request))
                .unwrap()
                .0,
            Err(Cause::WorkExhausted),
            "seam {seam} one short"
        );
        let cancellation = CancellationSource::new();
        let host = SerialRequest::from_memory(
            SerialMemoryBudget::new(1 << 26),
            cancellation.token(),
            None,
        );
        let request = ExecutionRequest::serial(&host);
        cancellation.cancel();
        assert_eq!(
            read(&request),
            Err(Cause::Cancelled),
            "seam {seam} cancellation"
        );
    }
    release_test_commit_snapshot(&runtime, &edge);
}
