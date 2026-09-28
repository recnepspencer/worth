//! Resumable structural comparison.
//!
//! One comparison orders two values of the same type: numerically for
//! numbers and quantities, bytewise for String, Bytes, and IDs, and
//! lexicographically then by length for options and composites. Equality is
//! an `Equal` ordering; only ordered types use the direction. Scalars cost one
//! unit, buses one per limb and plane, byte strings one per started 8-byte
//! word, and each composite element one visit, so a comparison suspends
//! between any two charged steps.

use std::cmp::Ordering;
use std::sync::Arc;
use std::task::Poll;

use crate::expressions::denial::ExpressionResult;
use crate::expressions::operators::decimal;
use crate::expressions::operators::text::WORD;

use super::meter::EvaluationMeter;
use super::value::{Composite, ExpressionValue, Repr};

#[derive(Debug, Clone)]
enum Task {
    Pair(ExpressionValue, ExpressionValue),
    Items {
        a: Arc<Composite>,
        b: Arc<Composite>,
        next: usize,
    },
    Bytes {
        a: ExpressionValue,
        b: ExpressionValue,
        offset: usize,
    },
}

#[derive(Debug, Clone)]
pub(super) struct Comparison {
    tasks: Vec<Task>,
    started: bool,
}

/// The bytes of a String, Bytes, or ID value.
fn bytes(value: &ExpressionValue) -> &[u8] {
    match &value.0 {
        Repr::String(text) => text.as_bytes(),
        Repr::Bytes(bytes) | Repr::Id(bytes) => bytes,
        _ => unreachable!("byte comparisons compare byte strings"),
    }
}

/// Work units a pair costs before it is split into further tasks.
fn pair_cost(value: &ExpressionValue) -> u64 {
    match &value.0 {
        Repr::Bits { limbs, .. } => limbs.len() as u64,
        Repr::Logic4 { value, .. } => value.len() as u64 * 2,
        _ => 1,
    }
}

impl Comparison {
    pub(super) fn new(a: ExpressionValue, b: ExpressionValue) -> Self {
        Self {
            tasks: vec![Task::Pair(a, b)],
            started: false,
        }
    }

    /// Advances until the ordering is known or the slice ends.
    pub(super) fn step(&mut self, meter: &mut EvaluationMeter) -> ExpressionResult<Poll<Ordering>> {
        if !self.started {
            self.started = true;
            meter.compare();
        }
        while let Some(task) = self.tasks.last_mut() {
            let ordering = match task {
                Task::Pair(a, _) => {
                    if meter.work(pair_cost(a))?.is_pending() {
                        return Ok(Poll::Pending);
                    }
                    let Some(Task::Pair(a, b)) = self.tasks.pop() else {
                        unreachable!("the top task is a pair");
                    };
                    self.split(a, b)
                }
                Task::Items { a, b, next } => {
                    let common = a.items.len().min(b.items.len());
                    if *next == common {
                        let ordering = a.items.len().cmp(&b.items.len());
                        self.tasks.pop();
                        ordering
                    } else {
                        meter.visit(1)?;
                        let pair = Task::Pair(a.items[*next].clone(), b.items[*next].clone());
                        *next += 1;
                        self.tasks.push(pair);
                        Ordering::Equal
                    }
                }
                Task::Bytes { a, b, offset } => {
                    if meter.work(1)?.is_pending() {
                        return Ok(Poll::Pending);
                    }
                    let (a, b) = (bytes(a), bytes(b));
                    let common = a.len().min(b.len());
                    let end = (*offset + WORD).min(common);
                    let ordering = a[*offset..end].cmp(&b[*offset..end]);
                    let lengths = a.len().cmp(&b.len());
                    *offset = end;
                    if ordering.is_ne() || end == common {
                        self.tasks.pop();
                        ordering.then(lengths)
                    } else {
                        Ordering::Equal
                    }
                }
            };
            if ordering.is_ne() {
                self.tasks.clear();
                return Ok(Poll::Ready(ordering));
            }
        }
        Ok(Poll::Ready(Ordering::Equal))
    }

    /// Orders a paid pair directly, or queues the work it needs.
    fn split(&mut self, a: ExpressionValue, b: ExpressionValue) -> Ordering {
        match (&a.0, &b.0) {
            (Repr::Bool(x), Repr::Bool(y)) => x.cmp(y),
            (Repr::Integer(x), Repr::Integer(y)) => x.cmp(y),
            (Repr::Enum(x), Repr::Enum(y)) => x.cmp(y),
            (Repr::Float32(x), Repr::Float32(y)) => {
                float(f64::from(f32::from_bits(*x)), f64::from(f32::from_bits(*y)))
            }
            (Repr::Float64(x), Repr::Float64(y)) | (Repr::Quantity(x), Repr::Quantity(y)) => {
                float(f64::from_bits(*x), f64::from_bits(*y))
            }
            (
                Repr::Decimal {
                    coefficient: ca,
                    scale: sa,
                },
                Repr::Decimal {
                    coefficient: cb,
                    scale: sb,
                },
            ) => decimal::compare((*ca, *sa), (*cb, *sb)),
            (Repr::String(_), Repr::String(_))
            | (Repr::Bytes(_), Repr::Bytes(_))
            | (Repr::Id(_), Repr::Id(_)) => {
                self.tasks.push(Task::Bytes { a, b, offset: 0 });
                Ordering::Equal
            }
            (Repr::Bits { limbs: x, .. }, Repr::Bits { limbs: y, .. }) => x.cmp(y),
            (
                Repr::Logic4 {
                    value: xv,
                    unknown: xu,
                    ..
                },
                Repr::Logic4 {
                    value: yv,
                    unknown: yu,
                    ..
                },
            ) => xv.cmp(yv).then(xu.cmp(yu)),
            (Repr::None, Repr::None) => Ordering::Equal,
            (Repr::None, Repr::Some(_)) => Ordering::Less,
            (Repr::Some(_), Repr::None) => Ordering::Greater,
            (Repr::Some(x), Repr::Some(y)) => {
                self.tasks.push(Task::Pair((**x).clone(), (**y).clone()));
                Ordering::Equal
            }
            (Repr::List(x), Repr::List(y))
            | (Repr::Map(x), Repr::Map(y))
            | (Repr::Record(x), Repr::Record(y))
            | (Repr::Entry(x), Repr::Entry(y)) => {
                // Always structural: a shared allocation costs what an equal,
                // separately built value costs.
                self.tasks.push(Task::Items {
                    a: x.clone(),
                    b: y.clone(),
                    next: 0,
                });
                Ordering::Equal
            }
            _ => unreachable!("admission compares only values of one type"),
        }
    }
}

/// Finite floats are totally ordered by value; both zeros are positive.
fn float(a: f64, b: f64) -> Ordering {
    a.partial_cmp(&b).expect("expression floats are finite")
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use super::Comparison;
    use crate::expressions::evaluation::meter::EvaluationMeter;
    use crate::expressions::evaluation::value::ExpressionValue;
    use crate::expressions::profile::ExpressionProfile;

    /// The ordering, the work charged, and the slices it took at `quantum`.
    fn compare(a: &ExpressionValue, b: &ExpressionValue, quantum: u64) -> (Ordering, u64, u64) {
        let mut meter = EvaluationMeter::new(&ExpressionProfile::interactive());
        let mut comparison = Comparison::new(a.clone(), b.clone());
        loop {
            meter.open_slice(quantum);
            if let std::task::Poll::Ready(ordering) = comparison.step(&mut meter).unwrap() {
                let cost = meter.cost();
                let work = cost.used(crate::expressions::ExpressionResource::SemanticWork);
                return (ordering, work, cost.slices());
            }
        }
    }

    fn text(value: &str) -> ExpressionValue {
        ExpressionValue::string(value)
    }

    #[test]
    fn strings_order_bytewise_one_word_per_unit() {
        let long = "x".repeat(40);
        assert_eq!(
            compare(&text(&long), &text(&long), 4096),
            (Ordering::Equal, 6, 1)
        );
        let shorter = &long[..39];
        assert_eq!(
            compare(&text(shorter), &text(&long), 4096).0,
            Ordering::Less
        );
        assert_eq!(
            compare(&text("b"), &text("abc"), 4096),
            (Ordering::Greater, 2, 1)
        );
        assert_eq!(compare(&text("é"), &text("z"), 4096).0, Ordering::Greater);
    }

    #[test]
    fn composites_compare_structurally_at_the_same_cost_under_any_quantum() {
        let list = |items: &[&str]| ExpressionValue::list(items.iter().map(|s| text(s)).collect());
        let a = list(&["alpha", "beta", &"g".repeat(100)]);
        let b = list(&["alpha", "beta", &"g".repeat(100)]);
        let whole = compare(&a, &b, 4096);
        assert_eq!(whole.0, Ordering::Equal);
        let sliced = compare(&a, &b, 1);
        assert_eq!((sliced.0, sliced.1), (whole.0, whole.1));
        assert!(sliced.2 > 1);
        let c = list(&["alpha", "bets"]);
        assert_eq!(compare(&a, &c, 4096).0, Ordering::Less);
        let some = ExpressionValue::some(ExpressionValue::integer(1));
        assert_eq!(
            compare(&ExpressionValue::none(), &some, 4096).0,
            Ordering::Less
        );
    }
}
