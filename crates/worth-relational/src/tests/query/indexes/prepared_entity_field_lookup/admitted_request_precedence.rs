use super::*;
use crate::facade::identity::EntityId;
use crate::facade::indexes::{BoundedEntityFieldSelectedLookup, PreparedEntityFieldLookup};
use crate::runtime::VisibilityProjectionView;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

pub(super) fn assert_admitted_request_precedence(
    runtime: &RelationalRuntime,
    view: &VisibilityProjectionView,
    prepared: &PreparedEntityFieldLookup,
    parity: BoundedIndexParityMode,
    expected: &[EntityId],
) {
    use crate::indexes::data::SelectedIndexReadWork::Operation;
    let value = string_aspect_value("cancellation-order");
    let candidate_work = view.exact_entity_state_read_work_bound().unwrap()
        - view
            .exact_entity_state_read_navigation_work_bound()
            .unwrap();
    let baseline_host = crate::test_execution_policy::serial_request();
    let baseline_request = ExecutionRequest::serial(&baseline_host);
    let mut candidate_visits = 0;
    let baseline = runtime
        .index_access()
        .execute_bounded_entity_field_lookup(
            BoundedEntityFieldSelectedLookup::new(
                view,
                prepared.index_id(),
                prepared.entity_kind(),
                prepared.field_locator(),
                &value,
                16,
                |work, _| {
                    if work == Operation(candidate_work) {
                        candidate_visits += 1;
                    }
                    Ok::<_, &str>(())
                },
            ),
            parity,
            &baseline_request,
        )
        .unwrap();
    assert_eq!(baseline.candidate_entity_ids(), expected);
    assert_eq!(
        candidate_visits,
        if parity == BoundedIndexParityMode::Certification {
            4
        } else {
            2
        }
    );
    // The root-issued record-read bound identifies a real candidate charge.
    // Visit one is the first index candidate; visit three is Certification's
    // first storage candidate. Stop inside that admission, before navigation.
    let stops: &[usize] = if parity == BoundedIndexParityMode::Certification {
        &[1, 3]
    } else {
        &[1]
    };
    let mut failures = Vec::new();
    for &stop_visit in stops {
        for exhaust_work in [false, true] {
            let cancellation = CancellationSource::new();
            let host = SerialRequest::from_memory(
                SerialMemoryBudget::new(1 << 26),
                cancellation.token(),
                None,
            );
            let execution = ExecutionRequest::serial(&host);
            let mut visits = 0;
            let mut calls = 0;
            let mut stopped_at = None;
            let identity = worth_foundational::PartitionIdentity::new(1);
            let scan = worth_execution::ExecutionScan::try_from_ordered(
                vec![identity],
                vec![(identity, ())],
            )
            .unwrap();
            let mut answer = None;
            execution
                .run(worth_execution::ExecutionWorkCeiling::new(0), |lease| {
                    scan.run::<(), (), (), _>(lease, (), 0, 0, 0, 0, |_, _, meter| {
                        answer = Some(runtime.index_access().execute_bounded_entity_field_lookup(
                            BoundedEntityFieldSelectedLookup::new(
                                view,
                                prepared.index_id(),
                                prepared.entity_kind(),
                                prepared.field_locator(),
                                &value,
                                16,
                                |work, _| {
                                    calls += 1;
                                    if stopped_at.is_some() {
                                        return Err("admission exhausted");
                                    }
                                    if work == Operation(candidate_work) {
                                        visits += 1;
                                        if visits == stop_visit {
                                            stopped_at = Some(calls);
                                            if exhaust_work {
                                                assert_eq!(
                                                    meter.checkpoint(1),
                                                    Err(
                                                        worth_execution::MapKernelStop::WorkCeiling
                                                    )
                                                );
                                            } else {
                                                cancellation.cancel();
                                            }
                                        }
                                    }
                                    Ok(())
                                },
                            ),
                            parity,
                            &execution,
                        ));
                        Ok(((), ()))
                    })
                })
                .unwrap();
            let expected = BoundedEntityFieldLookupDenialKind::Execution(if exhaust_work {
                crate::execution::RelationalExecutionDenialCause::WorkExhausted
            } else {
                crate::execution::RelationalExecutionDenialCause::Cancelled
            });
            let actual = match answer.expect("the active request ran the admitted lookup") {
                Err(Stop::Lookup(denial)) => Some(denial.kind()),
                _ => None,
            };
            if actual != Some(expected) || stopped_at != Some(calls) || visits != stop_visit {
                failures.push(format!("{parity:?}, admitted visit={stop_visit}, exhaust={exhaust_work}: cause={actual:?}, calls={calls}, stopped_at={stopped_at:?}, visits={visits}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
