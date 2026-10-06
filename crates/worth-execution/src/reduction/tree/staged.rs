use std::collections::BTreeMap;

use worth_foundational::PartitionIdentity;

use crate::{oracle::CanonicalBits, report::ChargedBytes};

use super::{
    construction::{FrontierEvent, FrontierPlan, Shape, SubtreeResult, SubtreeSpec},
    recombine::Engine,
    ReductionDenial, ReductionMetrics, ReductionRunFailure, ReductionRunStop, ReductionTree,
};

pub(crate) enum FrontierCharge {
    Node,
    TaskUnit,
}

impl<T: Clone + ChargedBytes + CanonicalBits, F: Fn(&T, &T) -> T> ReductionTree<T, F> {
    pub(crate) fn prepare_shape_checked<E>(
        plan: super::ReductionPlan,
        values: Vec<T>,
        identity: &T,
        combine: &F,
        max_value_bytes: u64,
        before_work: impl FnMut() -> Result<(), E>,
    ) -> Result<(Shape<T>, ReductionMetrics), ReductionRunFailure<E>> {
        if plan.identities().len() != values.len() {
            return Err(ReductionRunFailure {
                reason: ReductionRunStop::Denial(ReductionDenial::ValueCountMismatch),
                metrics: ReductionMetrics::default(),
            });
        }
        let mut engine = Engine::new(identity, combine, max_value_bytes, before_work);
        engine
            .validate(identity)
            .map_err(|reason| engine.failure(reason))?;
        let shape = Shape::build(plan.identities(), values, &mut engine)
            .map_err(|reason| engine.failure(reason))?;
        Ok((shape, engine.metrics()))
    }

    pub(crate) fn evaluate_subtree_checked<E>(
        shape: &Shape<T>,
        spec: SubtreeSpec,
        identity: &T,
        combine: &F,
        max_value_bytes: u64,
        before_work: impl FnMut() -> Result<(), E>,
    ) -> Result<SubtreeResult<T>, ReductionRunFailure<E>> {
        let mut engine = Engine::new(identity, combine, max_value_bytes, before_work);
        let (root, span) = shape
            .evaluate(spec, &mut engine)
            .map_err(|reason| engine.failure(reason))?;
        engine.set_full_span(span);
        Ok(SubtreeResult {
            root,
            metrics: engine.metrics(),
        })
    }

    /// Resolve speculative frontier results in the same enter/left/right/
    /// finish order as the serial oracle. A task error is observed only when
    /// its subtree is reached, after every earlier canonical join.
    pub(crate) fn settle_frontier_checked<E>(
        shape: Shape<T>,
        identity: T,
        combine: F,
        shape_metrics: ReductionMetrics,
        frontier: FrontierPlan,
        children: Vec<SubtreeResult<T>>,
        mut failed: Option<(PartitionIdentity, ReductionRunFailure<E>, u64)>,
        max_value_bytes: u64,
        mut charge: impl FnMut(FrontierCharge) -> Result<(), E>,
    ) -> Result<(Self, ReductionMetrics), ReductionRunFailure<E>> {
        let len = shape.len();
        if children.len() > frontier.tasks.len() {
            return Err(denied(shape_metrics));
        }
        let mut built = BTreeMap::new();
        for (spec, child) in frontier.tasks.iter().copied().zip(children) {
            built.insert(spec.index(), child);
        }
        let mut metrics = shape_metrics;
        for event in frontier.events {
            match event {
                FrontierEvent::Enter(_spec) => {
                    let mut engine = Engine::new(&identity, &combine, max_value_bytes, || {
                        charge(FrontierCharge::Node)
                    });
                    engine
                        .visit()
                        .map_err(|reason| ReductionRunFailure { reason, metrics })?;
                    metrics = add(metrics, engine.metrics())?;
                }
                FrontierEvent::Task(spec) => {
                    if let Some(child) = built.get(&spec.index()) {
                        charge_task(
                            &shape,
                            spec,
                            child.metrics.charged_work,
                            &mut metrics,
                            &mut charge,
                        )?;
                        metrics = add(metrics, child.metrics)?;
                    } else if failed.as_ref().is_some_and(|item| item.0 == spec.identity) {
                        let (_, failure, target) = failed.take().expect("checked task failure");
                        charge_task(&shape, spec, target, &mut metrics, &mut charge)?;
                        metrics = add(metrics, failure.metrics)?;
                        return Err(ReductionRunFailure {
                            reason: failure.reason,
                            metrics,
                        });
                    } else {
                        return Err(denied(metrics));
                    }
                }
                FrontierEvent::Finish(spec) => {
                    let [left_spec, right_spec] = shape.children_of(spec);
                    let left = left_spec.and_then(|child| built.remove(&child.index()));
                    let right = right_spec.and_then(|child| built.remove(&child.index()));
                    if left_spec.is_some() != left.is_some()
                        || right_spec.is_some() != right.is_some()
                    {
                        return Err(denied(metrics));
                    }
                    let child_span = left
                        .as_ref()
                        .map_or(0, |child| child.metrics.charged_span)
                        .max(right.as_ref().map_or(0, |child| child.metrics.charged_span));
                    let (partition, value) = shape.node_value(spec);
                    let mut engine = Engine::new(&identity, &combine, max_value_bytes, || {
                        charge(FrontierCharge::Node)
                    });
                    let root = (|| {
                        engine.visit()?;
                        engine.build(
                            partition,
                            value.clone(),
                            left.as_ref().and_then(|child| child.root.clone()),
                            right.as_ref().and_then(|child| child.root.clone()),
                        )
                    })()
                    .map_err(|reason| ReductionRunFailure {
                        reason,
                        metrics: metrics.add(engine.metrics()).unwrap_or(metrics),
                    })?;
                    let local = engine.metrics();
                    metrics = add(metrics, local)?;
                    let mut subtree = entered().add(local).map_err(|_| overflow(metrics))?;
                    for child in [left, right].into_iter().flatten() {
                        subtree = subtree.add(child.metrics).map_err(|_| overflow(metrics))?;
                    }
                    subtree.charged_span =
                        child_span.checked_add(4).ok_or_else(|| overflow(metrics))?;
                    built.insert(
                        spec.index(),
                        SubtreeResult {
                            root: Some(root),
                            metrics: subtree,
                        },
                    );
                }
            }
        }
        let root = if let Some(spec) = shape.full_spec() {
            let root = built.remove(&spec.index()).ok_or_else(|| denied(metrics))?;
            metrics.charged_span = shape_metrics
                .charged_span
                .checked_add(root.metrics.charged_span)
                .ok_or_else(|| overflow(metrics))?;
            root.root
        } else {
            None
        };
        if !built.is_empty() || failed.is_some() {
            return Err(denied(metrics));
        }
        Ok((
            Self {
                root,
                identity,
                combine,
                len,
            },
            metrics,
        ))
    }
}

fn charge_task<T, E>(
    shape: &Shape<T>,
    spec: SubtreeSpec,
    target: u64,
    metrics: &mut ReductionMetrics,
    charge: &mut impl FnMut(FrontierCharge) -> Result<(), E>,
) -> Result<(), ReductionRunFailure<E>> {
    let mut accepted = 0_u64;
    for _ in 0..target {
        if let Err(stop) = charge(FrontierCharge::TaskUnit) {
            let partial = shape
                .prefix_metrics(spec, accepted)
                .ok_or_else(|| overflow(*metrics))?;
            *metrics = add(*metrics, partial)?;
            return Err(ReductionRunFailure {
                reason: ReductionRunStop::Hook(stop),
                metrics: *metrics,
            });
        }
        accepted = accepted.checked_add(1).ok_or_else(|| overflow(*metrics))?;
    }
    Ok(())
}

fn entered() -> ReductionMetrics {
    ReductionMetrics {
        structural_visits: 1,
        charged_work: 1,
        charged_span: 1,
        ..ReductionMetrics::default()
    }
}

fn add<E>(
    left: ReductionMetrics,
    right: ReductionMetrics,
) -> Result<ReductionMetrics, ReductionRunFailure<E>> {
    left.add(right).map_err(|_| overflow(left))
}

fn overflow<E>(metrics: ReductionMetrics) -> ReductionRunFailure<E> {
    ReductionRunFailure {
        reason: ReductionRunStop::WorkCounterOverflow,
        metrics,
    }
}

fn denied<E>(metrics: ReductionMetrics) -> ReductionRunFailure<E> {
    ReductionRunFailure {
        reason: ReductionRunStop::Denial(ReductionDenial::CoverageMismatch),
        metrics,
    }
}
