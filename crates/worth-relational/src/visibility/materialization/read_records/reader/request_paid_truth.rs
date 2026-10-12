//! Each truth seam must refuse without returning a partial row set.
use super::super::read_prices::{
    ASPECT_BINDING, ASPECT_COMPARISON, EXAMINE_SLOT, KEYED_RECORD, RESERVE_RECORD,
};
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
    let entity_scan = 2 * (EXAMINE_SLOT + RESERVE_RECORD);
    let relation_scan = EXAMINE_SLOT + RESERVE_RECORD;
    let aspect_units = |plan: &crate::schema::data::LoweredAspectContractPlan| {
        let n = plan.executable_bindings.len();
        let ordered = plan
            .executable_bindings
            .windows(2)
            .all(|w| w[0].aspect_key() < w[1].aspect_key());
        let sort = if ordered || n <= 1 {
            0
        } else {
            n as u64 * u64::from(usize::BITS - (n - 1).leading_zeros())
        };
        KEYED_RECORD
            + n as u64 * ASPECT_BINDING
            + (n.saturating_sub(1) as u64 + sort) * ASPECT_COMPARISON
    };
    let entity_aspects = aspect_units(
        &runtime
            .schema_contract_runtime
            .aspect_contract_plans
            .entity_plans[&KindId(1)],
    );
    let relation_aspects = aspect_units(
        &runtime
            .schema_contract_runtime
            .aspect_contract_plans
            .relation_plans[&KindId(2)],
    );
    let aspect_rows = |plan: &crate::schema::data::LoweredAspectContractPlan| {
        plan.executable_bindings
            .iter()
            .map(|binding| binding.aspect_key())
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    };
    let entity_aspect_rows = aspect_rows(
        &runtime
            .schema_contract_runtime
            .aspect_contract_plans
            .entity_plans[&KindId(1)],
    );
    let relation_aspect_rows = aspect_rows(
        &runtime
            .schema_contract_runtime
            .aspect_contract_plans
            .relation_plans[&KindId(2)],
    );
    for (seam, units, rows) in [
        (0, KEYED_RECORD, 1),
        (1, KEYED_RECORD, 1),
        (2, entity_scan, 2),
        (3, entity_scan, 2),
        (4, relation_scan, 1),
        (5, relation_scan, 1),
        (6, relation_scan, 1),
        (7, relation_scan, 1),
        (8, entity_scan, 2),
        (9, relation_scan, 1),
        (10, relation_scan, 1),
        (11, relation_scan, 1),
        (12, entity_scan, 2),
        (13, entity_scan, 2),
        (14, relation_scan, 1),
        (15, relation_scan, 1),
        (16, entity_scan, 2),
        (17, relation_scan, 1),
        (18, entity_aspects, entity_aspect_rows),
        (19, relation_aspects, relation_aspect_rows),
        (20, entity_scan, 2),
        (21, entity_scan, 2),
        (22, relation_scan, 1),
        (23, relation_scan, 1),
        (24, entity_scan + relation_scan, 3),
        (25, entity_scan + relation_scan, 3),
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
                12..=17 => {
                    let view = truth.project_snapshot(&edge.snapshot, &execution).unwrap();
                    match seam {
                        12 => view
                            .authoritative_entity_records(request, KindId(1))
                            .map(|rows| rows.len()),
                        13 => view
                            .authoritative_entity_records_in(
                                request,
                                source.partition_id,
                                KindId(1),
                            )
                            .map(|rows| rows.len()),
                        14 => view
                            .authoritative_relation_records(request, KindId(2))
                            .map(|rows| rows.len()),
                        15 => view
                            .authoritative_relation_records_in(
                                request,
                                source.partition_id,
                                KindId(2),
                            )
                            .map(|rows| rows.len()),
                        16 => view
                            .all_authoritative_entity_records(request)
                            .map(|rows| rows.len()),
                        17 => view
                            .all_authoritative_relation_records(request)
                            .map(|rows| rows.len()),
                        _ => unreachable!(),
                    }
                }
                18 => truth
                    .entity_aspects_at_version(request, source, version)
                    .map(|rows| rows.unwrap().len()),
                19 => truth
                    .relation_aspects_at_version(request, relation, version)
                    .map(|rows| rows.unwrap().len()),
                20..=23 => super::super::read_work::metered(
                    request,
                    |context| {
                        let state = runtime.storage_access().current_edition();
                        let mut charge = |units| context.checkpoint(units);
                        match seam {
                            20 => truth
                                .visible_entity_slots_from_state(&state, version, &mut charge)
                                .map(|rows| {
                                    rows.into_iter().map(|(_, slots)| slots.count_ones()).sum()
                                }),
                            21 => truth
                                .visible_entity_slots_in_partition_from_state(
                                    &state,
                                    source.partition_id,
                                    version,
                                    &mut charge,
                                )
                                .map(|rows| rows.map_or(0, |slots| slots.count_ones())),
                            22 => truth
                                .visible_relation_slots_from_state(&state, version, &mut charge)
                                .map(|rows| {
                                    rows.into_iter().map(|(_, slots)| slots.count_ones()).sum()
                                }),
                            23 => truth
                                .visible_relation_slots_in_partition_from_state(
                                    &state,
                                    source.partition_id,
                                    version,
                                    &mut charge,
                                )
                                .map(|rows| rows.map_or(0, |slots| slots.count_ones())),
                            _ => unreachable!(),
                        }
                    },
                    |_| 0,
                )?
                .map_err(crate::execution::denial::kernel_stop),
                24 => truth.read_snapshot(&edge.snapshot, request).map(|view| {
                    view.map_or(0, |view| view.entities().len() + view.relations().len())
                }),
                25 => {
                    let identity = runtime.main_branch_identity();
                    let (_, basis) = runtime.observe_branch(&identity).unwrap();
                    truth
                        .read_observation(request, &basis.observation())
                        .map(|view| {
                            let view = view.unwrap();
                            view.entities().len() + view.relations().len()
                        })
                }
                _ => unreachable!(),
            }
        };
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
        let live = crate::test_execution_policy::serial_request();
        let request = ExecutionRequest::serial(&live);
        let (answer, report) = request
            .run(ExecutionWorkCeiling::new(units), |_| read(&request))
            .unwrap();
        assert_eq!(answer, Ok(rows), "seam {seam} exact budget");
        assert_eq!(report.charged_work(), units, "seam {seam} payer");
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
