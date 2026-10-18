//! Completes one computation with its actual per-compute execution report.
use super::*;
use worth_query_host::facade::application_contribution::WorthQueryCompletedPartitionedComputation;

/// Installs the computation's partitioned owner and returns the demand's
/// handler, which runs the computation over the demanded set at every
/// decision, reading through its own reader with its request's execution.
pub(in super::super) fn running<Computation, Owner>(
    setup: &mut Setup<'_>,
    owner: Owner,
) -> Installed
where
    Computation: ApplicationManagedComputation<
        CheckpointSchema,
        PlanarFinalOutputFeature,
        Input = RegionEntries,
    >,
    Owner: WorthQueryPartitionedComputationOwner<
        CheckpointSchema,
        PlanarFinalOutputFeature,
        Computation,
        Operation = TotalRegions,
        Output = f64,
        Stopped = u32,
    >,
{
    let installed =
        setup.partitioned_computation::<PlanarFinalOutputFeature, Computation, _>(owner)?;
    Ok(RegionTotalsHandler::running(move |reader, set| {
        let prepared = installed.prepare(reader, set);

        let observed = std::rc::Rc::new(std::cell::RefCell::new(None));

        let completing = std::rc::Rc::clone(&observed);
        let finish = move |computed: Result<
            WorthQueryCompletedPartitionedComputation<
                CheckpointSchema,
                PlanarFinalOutputFeature,
                Computation,
                Owner,
            >,
            _,
        >| {
            let outcome: RegionOutcome = (|| {
                let computed = computed?;
                let charged_work = computed.charged_work();

                let report = observed_report(
                    completing
                        .borrow_mut()
                        .take()
                        .expect("the computing thread captured this decision's probes"),
                );
                Ok(RegionTotal {
                    bits: computed.complete()?.to_bits(),
                    charged_work,

                    report,
                })
            })();
            room().outcomes.push(outcome);
            worth_query_host::facade::primary_graph::HandlerResult::Completed(())
        };
        match prepared {
            Ok(prepared) => {
                let plan = prepared.into_decision_plan(finish);

                plan.observe_computes_for_test(
                    || {
                        runs_on_this_thread();
                    },
                    runs_on_this_thread,
                    move |mut reports| {
                        assert_eq!(reports.len(), 1, "one report for this decision's compute");
                        *observed.borrow_mut() = reports.pop();
                    },
                )
            }
            Err(error) => {
                worth_query_host::facade::primary_graph::WorthQueryDecisionPlan::immediate(finish(
                    Err(error),
                ))
            }
        }
    }))
}

/// Execution's report of the decision's one run, which ran in full. How a
/// run ran is the observer's, never the handler's value.
fn observed_report(
    runs: Vec<(
        WorthQueryPartitionedComputationRunForTest,
        Option<ExecutionReport>,
    )>,
) -> ExecutionReport {
    assert_eq!(runs.len(), 1, "one run per decision");
    assert_eq!(runs[0].0, WorthQueryPartitionedComputationRunForTest::Full(
        worth_query_host::facade::application_contribution::WorthQueryPartitionedComputationFullCause::Unretained));
    runs[0]
        .1
        .expect("the unretained full run reports its execution work")
}
