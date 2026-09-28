//! Map construction, lookup, and enumeration as resumable jobs.
//!
//! A map literal sorts its entries by a bottom-up merge sort that charges one
//! unit per placed entry plus its key comparisons. Equal keys end adjacent in
//! sorted order and a comparison sort compares every adjacent pair, so a
//! duplicate key always meets its twin and denies. Lookup is a binary search
//! over the canonical order. A map read from an operand records the presence
//! of each key it looked up or enumerated, and hashing a key into its path
//! pays one unit per word of the key.

use std::cmp::Ordering;
use std::sync::Arc;
use std::task::Poll;

use crate::expressions::denial::{ExpressionDenial, ExpressionDenialDetail, ExpressionResult};

use super::apply::parts;
use super::compare::Comparison;
use super::cost::words;
use super::jobs::{ready, Job};
use super::machine::{Held, Runtime};
use super::meter::EvaluationMeter;
use super::paths::{ExpressionReadKind, Origin};
use super::value::{Composite, ExpressionValue, Repr};

#[derive(Debug)]
pub(super) struct MapSort {
    keys: Vec<ExpressionValue>,
    values: Vec<Held>,
    /// Entry indices sorted in runs of `width`.
    order: Vec<usize>,
    /// The next pass's runs of `2 * width`.
    merged: Vec<usize>,
    width: usize,
    /// The left run of the current merge starts here.
    start: usize,
    left: usize,
    right: usize,
    placed: usize,
    /// Whether the entry being placed is paid for.
    paid: bool,
    comparison: Option<Comparison>,
}

impl MapSort {
    /// `items` alternate key and value in source order.
    pub(super) fn new(items: Vec<Held>) -> Self {
        let (mut keys, mut values) = (Vec::new(), Vec::new());
        let mut items = items.into_iter();
        while let (Some(key), Some(value)) = (items.next(), items.next()) {
            keys.push(key.value);
            values.push(value);
        }
        let entries = keys.len();
        Self {
            keys,
            values,
            order: (0..entries).collect(),
            merged: vec![0; entries],
            width: 1,
            start: 0,
            left: 0,
            right: 1.min(entries),
            placed: 0,
            paid: false,
            comparison: None,
        }
    }

    pub(super) fn step(&mut self, meter: &mut EvaluationMeter) -> ExpressionResult<Poll<Held>> {
        let entries = self.keys.len();
        loop {
            if self.width >= entries {
                return self.finish(meter).map(Poll::Ready);
            }
            if self.start >= entries {
                std::mem::swap(&mut self.order, &mut self.merged);
                self.width *= 2;
                self.start = 0;
                (self.left, self.right, self.placed) = (0, self.width.min(entries), 0);
                continue;
            }
            let middle = (self.start + self.width).min(entries);
            let end = (self.start + 2 * self.width).min(entries);
            if self.placed == end {
                self.start = end;
                self.left = end;
                self.right = (end + self.width).min(entries);
                continue;
            }
            if !self.paid {
                if meter.work(1)?.is_pending() {
                    return Ok(Poll::Pending);
                }
                self.paid = true;
            }
            let take_left = if self.left == middle {
                false
            } else if self.right == end {
                true
            } else {
                let (left, right) = (self.order[self.left], self.order[self.right]);
                let comparison = self.comparison.get_or_insert_with(|| {
                    Comparison::new(self.keys[left].clone(), self.keys[right].clone())
                });
                let ordering = ready!(comparison.step(meter));
                self.comparison = None;
                if ordering == Ordering::Equal {
                    return Err(ExpressionDenial::new(ExpressionDenialDetail::InvalidValue(
                        "map keys are unique",
                    )));
                }
                ordering == Ordering::Less
            };
            let from = if take_left {
                &mut self.left
            } else {
                &mut self.right
            };
            self.merged[self.placed] = self.order[*from];
            *from += 1;
            self.placed += 1;
            self.paid = false;
        }
    }

    /// The canonical map; its origin keeps each value's, in entry order.
    fn finish(&mut self, meter: &mut EvaluationMeter) -> ExpressionResult<Held> {
        meter.allocate(16 + 16 * self.keys.len() as u64)?;
        let (mut items, mut origins) = (Vec::new(), Vec::new());
        for index in &self.order {
            let value = &self.values[*index];
            items.extend([self.keys[*index].clone(), value.value.clone()]);
            origins.extend([Origin::Computed, value.origin.clone()]);
        }
        Ok(Held {
            value: ExpressionValue(Repr::Map(Composite::new(items))),
            origin: parts(origins),
        })
    }
}

/// The path of `key` under the map's operand path, paid for by `owed`.
fn key_path(
    runtime: &mut Runtime,
    map: &Held,
    key: &ExpressionValue,
    owed: &mut Option<u64>,
) -> ExpressionResult<Poll<Origin>> {
    let Some(path) = map.origin.path() else {
        return Ok(Poll::Ready(Origin::Computed));
    };
    let owed = owed.get_or_insert_with(|| words(key.logical_bytes()));
    ready!(runtime.meter.prepay(owed));
    let entry = runtime
        .recorder
        .key(path, key.clone(), &mut runtime.meter)?;
    let presence = ExpressionReadKind::Presence;
    runtime
        .recorder
        .read(entry, presence, key, &mut runtime.meter)?;
    Ok(Poll::Ready(Origin::Path(entry)))
}

/// The origin of entry `index`'s value: its recorded path under an operand
/// map, or the part a constructed map kept for it.
fn value_origin(map: &Held, index: usize, path: Origin) -> Origin {
    match &map.origin {
        Origin::Parts(parts) => parts[2 * index + 1].clone(),
        _ => path,
    }
}

/// Map `get` or `contains`.
#[derive(Debug)]
pub(super) struct Lookup {
    map: Held,
    items: Arc<Composite>,
    key: ExpressionValue,
    get: bool,
    low: usize,
    high: usize,
    comparison: Option<Comparison>,
    /// `Some` once the search ends: the found entry, if any.
    found: Option<Option<usize>>,
    owed: Option<u64>,
}

impl Lookup {
    pub(super) fn job(map: Held, key: ExpressionValue, get: bool) -> Job {
        let Repr::Map(items) = &map.value.0 else {
            unreachable!("admission looks keys up only in maps");
        };
        let items = items.clone();
        Job::Lookup(Box::new(Self {
            high: items.items.len() / 2,
            map,
            items,
            key,
            get,
            low: 0,
            comparison: None,
            found: None,
            owed: None,
        }))
    }

    pub(super) fn step(&mut self, runtime: &mut Runtime) -> ExpressionResult<Poll<Held>> {
        while self.found.is_none() {
            let middle = (self.low + self.high) / 2;
            if let Some(comparison) = &mut self.comparison {
                let ordering = ready!(comparison.step(&mut runtime.meter));
                self.comparison = None;
                match ordering {
                    Ordering::Less => self.low = middle + 1,
                    Ordering::Greater => self.high = middle,
                    Ordering::Equal => self.found = Some(Some(middle)),
                }
            } else if self.low >= self.high {
                self.found = Some(None);
            } else {
                runtime.meter.visit(1)?;
                let probe = self.items.items[2 * middle].clone();
                self.comparison = Some(Comparison::new(probe, self.key.clone()));
            }
        }
        let origin = ready!(key_path(runtime, &self.map, &self.key, &mut self.owed));
        let found = self.found.flatten();
        if !self.get {
            let contained = ExpressionValue::bool(found.is_some());
            return Ok(Poll::Ready(Held::computed(contained)));
        }
        let Some(index) = found else {
            return Ok(Poll::Ready(Held::computed(ExpressionValue::none())));
        };
        runtime.meter.allocate(16)?;
        let value = ExpressionValue::some(self.items.items[2 * index + 1].clone());
        let origin = value_origin(&self.map, index, origin);
        Ok(Poll::Ready(Held {
            value,
            origin: parts(vec![origin]),
        }))
    }
}

/// `entries(map)`: `[key, value]` records in canonical key order.
#[derive(Debug)]
pub(super) struct Entries {
    map: Held,
    items: Arc<Composite>,
    next: usize,
    /// Whether the entry being produced is paid for.
    paid: bool,
    owed: Option<u64>,
    output: Vec<ExpressionValue>,
    origins: Vec<Origin>,
}

impl Entries {
    pub(super) fn job(map: Held) -> Job {
        let Repr::Map(items) = &map.value.0 else {
            unreachable!("admission enumerates only maps");
        };
        let items = items.clone();
        Job::Entries(Box::new(Self {
            map,
            items,
            next: 0,
            paid: false,
            owed: None,
            output: Vec::new(),
            origins: Vec::new(),
        }))
    }

    pub(super) fn step(&mut self, runtime: &mut Runtime) -> ExpressionResult<Poll<Held>> {
        while 2 * self.next < self.items.items.len() {
            let key = &self.items.items[2 * self.next];
            if !self.paid {
                if runtime.meter.work(1)?.is_pending() {
                    return Ok(Poll::Pending);
                }
                self.paid = true;
            }
            let origin = ready!(key_path(runtime, &self.map, key, &mut self.owed));
            (self.paid, self.owed) = (false, None);
            runtime.meter.visit(1)?;
            runtime.meter.allocate(32)?;
            runtime.meter.scratch(8)?;
            let value = self.items.items[2 * self.next + 1].clone();
            self.output.push(ExpressionValue::entry(key.clone(), value));
            let origin = value_origin(&self.map, self.next, origin);
            self.origins.push(parts(vec![Origin::Computed, origin]));
            self.next += 1;
        }
        runtime.meter.allocate(16)?;
        let origin = parts(std::mem::take(&mut self.origins));
        let value = ExpressionValue::list(std::mem::take(&mut self.output));
        Ok(Poll::Ready(Held { value, origin }))
    }
}
