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
        &self,
        lease: Option<&ExecutionResourceLease<'_>>,
        backend: BackendKind,
        changed: &ExecutionMap<T, K>,
        editions: DecomposeKernelEditions,
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
        self.check_changed(changed.identities(), editions)
            .map_err(DecomposeFailure::Input)?;
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
        check_cap(&changed_values, self.max_staged_bytes).map_err(DecomposeFailure::Input)?;
        let changed_contributions = self
            .changed_contributions(changed.identities(), &changed_values)
            .map_err(DecomposeFailure::Input)?;
        let (interiors, contributions) = self.merge_interiors(changed.identities(), changed_values);
        check_cap(&interiors, self.max_staged_bytes).map_err(DecomposeFailure::Input)?;
        check_cap(&contributions, self.max_staged_bytes).map_err(DecomposeFailure::Input)?;
        let mut reuse = DecomposeReuse::default();
        if self.snapshot.is_some() {
            reuse.unchanged_contributions =
                self.identities.as_slice().len() - changed_contributions.len();
        }
        stage.set(DecomposeStage::Reduction);
        let before = context.cost();
        let (tree, reduction_metrics, needs_span_rewrite) = if let Some(previous) = &self.snapshot {
            let mut tree = previous.tree.clone();
            let mut metrics = ReductionMetrics::default();
            for identity in changed_contributions {
                let index = self
                    .identities
                    .as_slice()
                    .binary_search(&identity)
                    .expect("checked identity");
                let edit = tree
                    .update_checked(
                        identity,
                        contributions[index].clone(),
                        self.max_reduction_value_bytes,
                        || context.checkpoint(1),
                    )
                    .map_err(DecomposeFailure::ReductionRun)?;
                add_metrics(&mut metrics, edit).map_err(DecomposeFailure::ReductionRun)?;
            }
            (tree, metrics, true)
        } else {
            let plan = ReductionPlan::from_canonical(self.identities.clone());
            if let Some(lease) = lease {
                let (tree, metrics) = ReductionTree::build_scheduled_checked(
                    lease,
                    backend,
                    plan,
                    contributions.clone(),
                    self.reduction_identity.clone(),
                    self.combine.clone(),
                    self.max_reduction_value_bytes,
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
                let entries = self
                    .identities
                    .as_slice()
                    .iter()
                    .copied()
                    .zip(contributions.iter().cloned())
                    .collect();
                let (tree, metrics) = ReductionTree::try_from_declared_checked(
                    plan,
                    entries,
                    self.reduction_identity.clone(),
                    self.combine.clone(),
                    self.max_reduction_value_bytes,
                    || context.checkpoint(1),
                )
                .map_err(DecomposeFailure::ReductionRun)?;
                (tree, metrics, true)
            }
        };
        if needs_span_rewrite
            && !context.apply_structural_span(
                before,
                reduction_metrics.charged_work,
                reduction_metrics.charged_span,
            )
        {
            return Err(DecomposeFailure::Input(
                DecomposeInputDenial::StagedCapacityExceeded,
            ));
        }
        let assembled = tree.result().clone();
        check_cap(&assembled, self.max_staged_bytes).map_err(DecomposeFailure::Input)?;
        let interface_unchanged = match &self.snapshot {
            Some(previous) if previous.editions.interface == editions.interface => {
                same_encoding(&previous.assembled, &assembled, self.max_staged_bytes)
                    .map_err(DecomposeFailure::Input)?
            }
            None => false,
            Some(_) => false,
        };
        stage.set(DecomposeStage::Interface);
        let (interface, interface_report) = if interface_unchanged {
            reuse.interface_solve_reused = true;
            (
                self.snapshot.as_ref().expect("checked").interface.clone(),
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
                    max_result_bytes: self.max_interface_result_bytes,
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
        check_cap(&interface, self.max_staged_bytes).map_err(DecomposeFailure::Input)?;
        if interface.slices.len() != self.identities.as_slice().len() {
            return Err(DecomposeFailure::Input(
                DecomposeInputDenial::InterfaceSliceCoverageMismatch,
            ));
        }
        let previous = self
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
            self.identities.as_slice(),
            previous,
            &interiors,
            &interface.slices,
            &mut reuse,
            self.max_staged_bytes,
        )
        .map_err(DecomposeFailure::Input)?;
        stage.set(DecomposeStage::Back);
        let back_map = admit_back_map(
            &self.identities,
            &back_ids,
            &interiors,
            &interface.slices,
            self.max_back_result_bytes,
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
        check_cap(&back_values, self.max_staged_bytes).map_err(DecomposeFailure::Input)?;
        let mut outputs = self
            .snapshot
            .as_ref()
            .map_or_else(Vec::new, |old| old.outputs.clone());
        if self.snapshot.is_none() {
            outputs = back_values;
        } else {
            for (identity, value) in back_ids.into_iter().zip(back_values) {
                let index = self
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
        if snapshot.stage_bytes() > self.max_staged_bytes {
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
        check_cap(&complete.values, self.max_staged_bytes).map_err(DecomposeFailure::Input)?;
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
    let Some(nodes) = aggregate
        .recombined_nodes
        .checked_add(edit.recombined_nodes)
    else {
        return Err(metric_overflow(*aggregate));
    };
    let Some(calls) = aggregate.combine_calls.checked_add(edit.combine_calls) else {
        return Err(metric_overflow(*aggregate));
    };
    let Some(visits) = aggregate
        .structural_visits
        .checked_add(edit.structural_visits)
    else {
        return Err(metric_overflow(*aggregate));
    };
    let Some(work) = aggregate.charged_work.checked_add(edit.charged_work) else {
        return Err(metric_overflow(*aggregate));
    };
    let Some(span) = aggregate.charged_span.checked_add(edit.charged_span) else {
        return Err(metric_overflow(*aggregate));
    };
    *aggregate = ReductionMetrics {
        structural_visits: visits,
        recombined_nodes: nodes,
        combine_calls: calls,
        charged_work: work,
        charged_span: span,
    };
    Ok(())
}

fn metric_overflow(metrics: ReductionMetrics) -> ReductionRunFailure<MapKernelStop> {
    ReductionRunFailure {
        reason: crate::reduction::ReductionRunStop::WorkCounterOverflow,
        metrics,
    }
}
