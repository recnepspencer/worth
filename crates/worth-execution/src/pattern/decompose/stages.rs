use super::*;
use crate::backend::BackendKind;
use crate::reduction::ScheduledReductionError;
use std::cell::Cell;
use worth_foundational::ExecutionFallbackCause;

impl<I, C, S, B, O, F> ExecutionDecompose<I, C, S, B, O, F>
where
    I: Clone + Send + Sync + ChargedBytes + CanonicalBits,
    C: Clone + Send + Sync + ChargedBytes + CanonicalBits,
    S: Clone + Send + ChargedBytes,
    B: Clone + Send + Sync + ChargedBytes + CanonicalBits,
    O: Clone + Send + ChargedBytes,
    F: Fn(&C, &C) -> C + Clone + Sync,
{
    pub(super) fn run_stages<T, K, E, Interior, Interface, Back>(
        checked_changed: CheckedChangedInput<'_, Self, T, K>,
        lease: Option<&ExecutionResourceLease<'_>>,
        backend: BackendKind,
        interior: &Interior,
        solve_interface: &Interface,
        back_substitute: &Back,
        context: &mut MapKernelContext<'_, '_>,
        stage: &Cell<DecomposeStage>,
        fallback: &Cell<Option<ExecutionFallbackCause>>,
    ) -> Result<Staged<I, C, S, B, O, F>, DecomposeFailure<E>>
    where
        T: Sync + ChargedBytes,
        E: Send + ChargedBytes,
        Interior: Fn(
                &T,
                &mut MapKernelContext<'_, '_>,
            ) -> Result<InteriorResult<I, C>, MapKernelFailure<E>>
            + Sync,
        Interface: Fn(
                &C,
                &mut MapKernelContext<'_, '_>,
            ) -> Result<InterfaceSolution<S, B>, MapKernelFailure<E>>
            + Sync,
        Back: Fn(&BackInput<I, B>, &mut MapKernelContext<'_, '_>) -> Result<O, MapKernelFailure<E>>
            + Sync,
    {
        let (decomposition, changed, editions) = checked_changed.into_parts();
        stage.set(DecomposeStage::Interior);
        let interior_outcome = changed.run_with_backend(lease, backend, interior);
        note_fallback(fallback, &interior_outcome);
        let (changed_values, interior_report) = match interior_outcome {
            MapOutcome::Complete { values, report } => (values, report),
            MapOutcome::Stopped {
                boundary,
                reason,
                report,
                ..
            } => {
                return Err(DecomposeFailure::Interior {
                    boundary,
                    reason,
                    report,
                });
            }
        };
        check_cap(&changed_values, decomposition.max_staged_bytes)
            .map_err(DecomposeFailure::Input)?;
        let changed_contributions = decomposition
            .changed_contributions(changed.identities(), &changed_values)
            .map_err(DecomposeFailure::Input)?;
        let (interiors, contributions) =
            decomposition.merge_interiors(changed.identities(), changed_values);
        check_cap(&interiors, decomposition.max_staged_bytes).map_err(DecomposeFailure::Input)?;
        check_cap(&contributions, decomposition.max_staged_bytes)
            .map_err(DecomposeFailure::Input)?;
        let mut reuse = DecomposeReuse::default();
        if decomposition.snapshot.is_some() {
            reuse.unchanged_contributions =
                decomposition.identities.as_slice().len() - changed_contributions.len();
        }
        stage.set(DecomposeStage::Reduction);
        let before = context.cost();
        let (tree, reduction_metrics, needs_span_rewrite) =
            if let Some(previous) = &decomposition.snapshot {
                let mut tree = previous.tree.clone();
                let mut metrics = ReductionMetrics::default();
                for identity in changed_contributions {
                    let index = decomposition
                        .identities
                        .as_slice()
                        .binary_search(&identity)
                        .expect("checked identity");
                    let edit = tree
                        .update_checked(
                            identity,
                            contributions[index].clone(),
                            decomposition.max_reduction_value_bytes,
                            || context.checkpoint(1),
                        )
                        .map_err(DecomposeFailure::ReductionRun)?;
                    add_metrics(&mut metrics, edit).map_err(DecomposeFailure::ReductionRun)?;
                }
                (tree, metrics, true)
            } else {
                let plan = ReductionPlan::from_canonical(decomposition.identities.clone());
                if let Some(lease) = lease {
                    let (tree, metrics) = ReductionTree::build_scheduled_checked(
                        lease,
                        backend,
                        plan,
                        contributions.clone(),
                        decomposition.reduction_identity.clone(),
                        decomposition.combine.clone(),
                        decomposition.max_reduction_value_bytes,
                        context,
                    )
                    .map_err(|error| match error {
                        ScheduledReductionError::Admission(denial) => {
                            DecomposeFailure::ScopeAdmission(denial)
                        }
                        ScheduledReductionError::Reduction(failure) => {
                            DecomposeFailure::ReductionRun(failure)
                        }
                    })?;
                    (tree, metrics, false)
                } else {
                    let entries = decomposition
                        .identities
                        .as_slice()
                        .iter()
                        .copied()
                        .zip(contributions.iter().cloned())
                        .collect();
                    let (tree, metrics) = ReductionTree::try_from_declared_checked(
                        plan,
                        entries,
                        decomposition.reduction_identity.clone(),
                        decomposition.combine.clone(),
                        decomposition.max_reduction_value_bytes,
                        || context.checkpoint(1),
                    )
                    .map_err(DecomposeFailure::ReductionRun)?;
                    (tree, metrics, true)
                }
            };
        if needs_span_rewrite
            && context
                .apply_structural_span(
                    before,
                    reduction_metrics
                        .accounted_metrics()
                        .expect("bounded decomposition")
                        .charged_work,
                    reduction_metrics
                        .accounted_metrics()
                        .expect("bounded decomposition")
                        .charged_span,
                )
                .is_err()
        {
            return Err(DecomposeFailure::Input(
                DecomposeInputDenial::StagedCapacityExceeded,
            ));
        }
        let assembled = tree.result().clone();
        check_cap(&assembled, decomposition.max_staged_bytes).map_err(DecomposeFailure::Input)?;
        let interface_unchanged = match &decomposition.snapshot {
            Some(previous) if previous.editions.interface == editions.interface => same_encoding(
                &previous.assembled,
                &assembled,
                decomposition.max_staged_bytes,
            )
            .map_err(DecomposeFailure::Input)?,
            None => false,
            Some(_) => false,
        };
        stage.set(DecomposeStage::Interface);
        let (interface, interface_report) = if interface_unchanged {
            reuse.interface_solve_reused = true;
            (
                decomposition
                    .snapshot
                    .as_ref()
                    .expect("checked")
                    .interface
                    .clone(),
                None,
            )
        } else {
            let map = ExecutionMap::try_from_declared_partitions(
                vec![PartitionIdentity::new(0)],
                vec![MapPartition {
                    identity: PartitionIdentity::new(0),
                    value: assembled.clone(),
                    read_keys: Vec::<PartitionIdentity>::new(),
                    write_keys: Vec::new(),
                    kernel_scratch_bytes: 0,
                    max_result_bytes: decomposition.max_interface_result_bytes,
                }],
            )
            .map_err(DecomposeFailure::InterfaceAdmission)?;
            let outcome = map.run_with_backend(lease, backend, solve_interface);
            note_fallback(fallback, &outcome);
            match outcome {
                MapOutcome::Complete { mut values, report } => (values.remove(0), Some(report)),
                MapOutcome::Stopped { reason, report, .. } => {
                    return Err(DecomposeFailure::Interface { reason, report });
                }
            }
        };
        check_cap(&interface, decomposition.max_staged_bytes).map_err(DecomposeFailure::Input)?;
        if interface.slices.len() != decomposition.identities.as_slice().len() {
            return Err(DecomposeFailure::Input(
                DecomposeInputDenial::InterfaceSliceCoverageMismatch,
            ));
        }
        let previous = decomposition
            .snapshot
            .as_ref()
            .filter(|snapshot| snapshot.editions.back == editions.back)
            .map(|snapshot| {
                (
                    snapshot.interiors.as_slice(),
                    snapshot.interface.slices.as_slice(),
                )
            });
        let back_ids = changed_back_identities(
            decomposition.identities.as_slice(),
            previous,
            &interiors,
            &interface.slices,
            &mut reuse,
            decomposition.max_staged_bytes,
        )
        .map_err(DecomposeFailure::Input)?;
        stage.set(DecomposeStage::Back);
        let back_map = admit_back_map(
            &decomposition.identities,
            &back_ids,
            &interiors,
            &interface.slices,
            decomposition.max_back_result_bytes,
        )
        .map_err(DecomposeFailure::BackAdmission)?;
        let (back_values, back_report) = if back_ids.is_empty() {
            (Vec::new(), None)
        } else {
            let outcome = back_map.run_with_backend(lease, backend, back_substitute);
            note_fallback(fallback, &outcome);
            match outcome {
                MapOutcome::Complete { values, report } => (values, Some(report)),
                MapOutcome::Stopped {
                    boundary,
                    reason,
                    report,
                    ..
                } => {
                    return Err(DecomposeFailure::Back {
                        boundary,
                        reason,
                        report,
                    });
                }
            }
        };
        check_cap(&back_values, decomposition.max_staged_bytes).map_err(DecomposeFailure::Input)?;
        let mut outputs = decomposition
            .snapshot
            .as_ref()
            .map_or_else(Vec::new, |old| old.outputs.clone());
        if decomposition.snapshot.is_none() {
            outputs = back_values;
        } else {
            for (identity, value) in back_ids.into_iter().zip(back_values) {
                let index = decomposition
                    .identities
                    .as_slice()
                    .binary_search(&identity)
                    .expect("checked identity");
                outputs[index] = value;
            }
        }
        let snapshot = Snapshot {
            editions,
            tree,
            interiors,
            contributions,
            assembled,
            interface,
            outputs: outputs.clone(),
        };
        if snapshot.stage_bytes() > decomposition.max_staged_bytes {
            return Err(DecomposeFailure::Input(
                DecomposeInputDenial::StagedCapacityExceeded,
            ));
        }
        let complete = DecomposeComplete {
            values: outputs,
            total_report: scope::empty_report(),
            interior_report,
            reduction_metrics,
            interface_report,
            back_report,
            reuse,
        };
        check_cap(&complete.values, decomposition.max_staged_bytes)
            .map_err(DecomposeFailure::Input)?;
        Ok(Staged { snapshot, complete })
    }
}

/// Keeps the first stage's reason for running serially, in stage order, for
/// the decomposition's total report.
fn note_fallback<R, E>(
    fallback: &Cell<Option<ExecutionFallbackCause>>,
    outcome: &MapOutcome<R, E>,
) {
    if fallback.get().is_none() {
        fallback.set(outcome.report().fallback());
    }
}

fn check_cap<T: ChargedBytes>(value: &T, max_bytes: u64) -> Result<(), DecomposeInputDenial> {
    if value.additional_charged_bytes() > max_bytes {
        Err(DecomposeInputDenial::StagedCapacityExceeded)
    } else {
        Ok(())
    }
}

fn add_metrics(
    aggregate: &mut ReductionMetrics,
    edit: ReductionMetrics,
) -> Result<(), ReductionRunFailure<MapKernelStop>> {
    let accumulated = aggregate
        .accounted_metrics()
        .expect("bounded decomposition");
    let delta = edit.accounted_metrics().expect("bounded decomposition");
    let Some(nodes) = accumulated
        .recombined_nodes
        .checked_add(delta.recombined_nodes)
    else {
        return Err(metric_overflow(*aggregate));
    };
    let Some(calls) = accumulated.combine_calls.checked_add(delta.combine_calls) else {
        return Err(metric_overflow(*aggregate));
    };
    let Some(visits) = accumulated
        .structural_visits
        .checked_add(delta.structural_visits)
    else {
        return Err(metric_overflow(*aggregate));
    };
    let Some(work) = accumulated.charged_work.checked_add(delta.charged_work) else {
        return Err(metric_overflow(*aggregate));
    };
    let Some(span) = accumulated.charged_span.checked_add(delta.charged_span) else {
        return Err(metric_overflow(*aggregate));
    };
    *aggregate = ReductionMetrics::from_accounting(crate::ReductionAccounting {
        structural_visits: visits,
        recombined_nodes: nodes,
        combine_calls: calls,
        charged_work: work,
        charged_span: span,
    });
    Ok(())
}

fn metric_overflow(metrics: ReductionMetrics) -> ReductionRunFailure<MapKernelStop> {
    ReductionRunFailure {
        reason: crate::reduction::ReductionRunStop::work_counter_overflow(),
        metrics,
    }
}

#[cfg(test)]
mod tests;
