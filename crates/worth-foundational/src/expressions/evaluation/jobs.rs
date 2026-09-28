//! Resumable operations whose work depends on their operands' contents.
//!
//! A job keeps its own progress, charges each step as it takes it, and
//! returns `Pending` when the slice cannot pay, so no builtin hides an
//! unbounded step and the total charge never depends on where slices end.

use std::cmp::Ordering;
use std::sync::Arc;
use std::task::Poll;

use crate::expressions::denial::{ExpressionDenial, ExpressionDenialDetail, ExpressionResult};
use crate::expressions::syntax::ast::BinaryOp;
use crate::expressions::types::ExpressionType;

use super::compare::Comparison;
use super::cost::DECIMAL_WORK;
use super::machine::{Held, Runtime};
use super::maps::{Entries, Lookup, MapSort};
use super::meter::EvaluationMeter;
use super::numbers;
use super::scan::TextJob;
use super::value::{Composite, ExpressionValue, Repr};

/// Unwraps a ready step, or returns `Pending` from the enclosing job.
macro_rules! ready {
    ($step:expr) => {
        match $step? {
            Poll::Ready(value) => value,
            Poll::Pending => return Ok(Poll::Pending),
        }
    };
}
pub(super) use ready;

#[derive(Debug)]
pub(super) enum Job {
    Compare(BinaryOp, Comparison),
    /// Two-argument `min` or `max`; ties keep the first argument.
    Pick {
        max: bool,
        values: [ExpressionValue; 2],
        comparison: Comparison,
    },
    Clamp(Box<Clamp>),
    Fold(Box<Fold>),
    Lookup(Box<Lookup>),
    Entries(Box<Entries>),
    Sort(Box<MapSort>),
    Text(Box<TextJob>),
}

impl Job {
    pub(super) fn compare(op: BinaryOp, a: ExpressionValue, b: ExpressionValue) -> Self {
        Self::Compare(op, Comparison::new(a, b))
    }

    pub(super) fn pick(max: bool, a: ExpressionValue, b: ExpressionValue) -> Self {
        Self::Pick {
            max,
            comparison: Comparison::new(b.clone(), a.clone()),
            values: [a, b],
        }
    }

    pub(super) fn step(&mut self, runtime: &mut Runtime) -> ExpressionResult<Poll<Held>> {
        let meter = &mut runtime.meter;
        let value = match self {
            Self::Compare(op, comparison) => {
                let ordering = ready!(comparison.step(meter));
                ExpressionValue::bool(match op {
                    BinaryOp::Less => ordering.is_lt(),
                    BinaryOp::LessEqual => ordering.is_le(),
                    BinaryOp::Greater => ordering.is_gt(),
                    BinaryOp::GreaterEqual => ordering.is_ge(),
                    BinaryOp::Equal => ordering.is_eq(),
                    BinaryOp::NotEqual => ordering.is_ne(),
                    _ => unreachable!("comparison jobs compare"),
                })
            }
            Self::Pick {
                max,
                values,
                comparison,
            } => {
                // The second argument against the first.
                let ordering = ready!(comparison.step(meter));
                let wanted = if *max {
                    Ordering::Greater
                } else {
                    Ordering::Less
                };
                values[usize::from(ordering == wanted)].clone()
            }
            Self::Clamp(clamp) => ready!(clamp.step(meter)),
            Self::Fold(fold) => ready!(fold.step(meter)),
            Self::Lookup(lookup) => return lookup.step(runtime),
            Self::Entries(entries) => return entries.step(runtime),
            Self::Sort(sort) => ready!(sort.step(meter)),
            Self::Text(text) => ready!(text.step(meter)),
        };
        Ok(Poll::Ready(Held::computed(value)))
    }
}

/// `clamp(x, low, high)`: reversed bounds deny before `x` is compared.
#[derive(Debug)]
pub(super) struct Clamp {
    /// `[x, low, high]`.
    values: [ExpressionValue; 3],
    /// Which comparison is running: bounds, then low, then high.
    stage: usize,
    comparison: Comparison,
}

impl Clamp {
    pub(super) fn job(values: [ExpressionValue; 3]) -> Job {
        let comparison = Comparison::new(values[1].clone(), values[2].clone());
        Job::Clamp(Box::new(Self {
            values,
            stage: 0,
            comparison,
        }))
    }

    fn step(&mut self, meter: &mut EvaluationMeter) -> ExpressionResult<Poll<ExpressionValue>> {
        loop {
            let ordering = ready!(self.comparison.step(meter));
            let [x, low, high] = &self.values;
            match (self.stage, ordering) {
                (0, Ordering::Greater) => {
                    return Err(ExpressionDenial::new(ExpressionDenialDetail::Bounds(
                        "clamp bounds are reversed",
                    )));
                }
                (0, _) => self.comparison = Comparison::new(x.clone(), low.clone()),
                (1, Ordering::Less) => return Ok(Poll::Ready(low.clone())),
                (1, _) => self.comparison = Comparison::new(x.clone(), high.clone()),
                (_, Ordering::Greater) => return Ok(Poll::Ready(high.clone())),
                _ => return Ok(Poll::Ready(x.clone())),
            }
            self.stage += 1;
        }
    }
}

#[derive(Debug)]
enum FoldKind {
    /// `min(list)` or `max(list)`: the index of the extreme so far; ties keep
    /// the earliest element.
    Extreme { max: bool, best: usize },
    /// `sum(list)` from the typed zero, left to right.
    Sum {
        ty: ExpressionType,
        total: ExpressionValue,
    },
    /// List `contains`: stops at the first equal element.
    Member { probe: ExpressionValue },
}

/// A left-to-right pass over a list, one visited element per step.
#[derive(Debug)]
pub(super) struct Fold {
    kind: FoldKind,
    list: Arc<Composite>,
    /// The next element to visit.
    index: usize,
    comparison: Option<Comparison>,
}

impl Fold {
    fn job(kind: FoldKind, list: &ExpressionValue) -> Job {
        let Repr::List(list) = &list.0 else {
            unreachable!("admission folds only lists");
        };
        Job::Fold(Box::new(Self {
            kind,
            list: list.clone(),
            index: 0,
            comparison: None,
        }))
    }

    pub(super) fn extreme(max: bool, list: &ExpressionValue) -> Job {
        Self::job(FoldKind::Extreme { max, best: 0 }, list)
    }

    pub(super) fn sum(ty: &ExpressionType, list: &ExpressionValue) -> Job {
        let kind = FoldKind::Sum {
            ty: ty.clone(),
            total: numbers::zero(ty),
        };
        Self::job(kind, list)
    }

    pub(super) fn member(list: &ExpressionValue, probe: ExpressionValue) -> Job {
        Self::job(FoldKind::Member { probe }, list)
    }

    fn step(&mut self, meter: &mut EvaluationMeter) -> ExpressionResult<Poll<ExpressionValue>> {
        let items = &self.list.items;
        loop {
            if let Some(comparison) = &mut self.comparison {
                let ordering = ready!(comparison.step(meter));
                self.comparison = None;
                match &mut self.kind {
                    FoldKind::Extreme { max, best } => {
                        let wanted = if *max {
                            Ordering::Greater
                        } else {
                            Ordering::Less
                        };
                        if ordering == wanted {
                            *best = self.index - 1;
                        }
                    }
                    FoldKind::Member { .. } if ordering.is_eq() => {
                        return Ok(Poll::Ready(ExpressionValue::bool(true)));
                    }
                    _ => {}
                }
            }
            if self.index == items.len() {
                return self.finish(meter).map(Poll::Ready);
            }
            let item = &items[self.index];
            match &mut self.kind {
                FoldKind::Extreme { best, .. } => {
                    meter.visit(1)?;
                    if self.index > 0 {
                        self.comparison = Some(Comparison::new(item.clone(), items[*best].clone()));
                    }
                }
                FoldKind::Sum { ty, total } => {
                    let units = if *ty == ExpressionType::Decimal {
                        DECIMAL_WORK
                    } else {
                        1
                    };
                    if meter.work(units)?.is_pending() {
                        return Ok(Poll::Pending);
                    }
                    meter.visit(1)?;
                    *total = numbers::arithmetic(BinaryOp::Add, ty, total, item)?;
                }
                FoldKind::Member { probe } => {
                    meter.visit(1)?;
                    self.comparison = Some(Comparison::new(item.clone(), probe.clone()));
                }
            }
            self.index += 1;
        }
    }

    fn finish(&mut self, meter: &mut EvaluationMeter) -> ExpressionResult<ExpressionValue> {
        Ok(match &self.kind {
            FoldKind::Extreme { .. } if self.list.items.is_empty() => ExpressionValue::none(),
            FoldKind::Extreme { best, .. } => {
                meter.allocate(16)?;
                ExpressionValue::some(self.list.items[*best].clone())
            }
            FoldKind::Sum { total, .. } => total.clone(),
            FoldKind::Member { .. } => ExpressionValue::bool(false),
        })
    }
}
