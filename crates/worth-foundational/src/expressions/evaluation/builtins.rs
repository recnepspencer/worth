//! Builtin calls. Option and collection builtins record exactly the part of
//! an operand they inspect, such as its presence, length, one element, or one
//! key. Every other builtin consumes its arguments whole, then computes
//! directly or starts a job whose work depends on their contents.

use crate::expressions::denial::{ExpressionDenial, ExpressionDenialDetail, ExpressionResult};
use crate::expressions::functions::Builtin;
use crate::expressions::operators::decimal::{self, Rounding};
use crate::expressions::operators::{float, quantity};
use crate::expressions::types::ExpressionType;

use super::apply::{parts, take};
use super::buses::bus;
use super::jobs::{Clamp, Fold, Job};
use super::machine::{Applied, Held, Machine};
use super::maps::{Entries, Lookup};
use super::numbers::{self, decimal_of, integer_of, magnitude_of};
use super::paths::{ExpressionReadKind, Origin};
use super::scan::TextJob;
use super::value::{ExpressionValue, Repr};

fn rounding(value: &ExpressionValue) -> Rounding {
    Rounding::from_variant(
        value
            .as_variant()
            .expect("admission typed this as Rounding"),
    )
}

impl Machine {
    pub(super) fn builtin(
        &mut self,
        builtin: Builtin,
        ty: &ExpressionType,
        types: &[&ExpressionType],
        arguments: Vec<Held>,
    ) -> ExpressionResult<Applied> {
        let over_map = matches!(types.first(), Some(ExpressionType::Map(..)));
        match builtin {
            Builtin::Some => {
                let [inner] = take(arguments);
                self.runtime.meter.allocate(16)?;
                let value = ExpressionValue::some(inner.value);
                let origin = parts(vec![inner.origin]);
                return Ok(Applied::Ready(Held { value, origin }));
            }
            Builtin::IsSome | Builtin::Unwrap => {
                let [option] = take(arguments);
                self.runtime.read(&option, ExpressionReadKind::Presence)?;
                let inner = option.value.as_option().flatten();
                if builtin == Builtin::IsSome {
                    let value = ExpressionValue::bool(inner.is_some());
                    return Ok(Applied::Ready(Held::computed(value)));
                }
                let inner = inner
                    .ok_or_else(|| ExpressionDenial::new(ExpressionDenialDetail::AbsentValue))?;
                let origin = match &option.origin {
                    Origin::Parts(parts) => parts[0].clone(),
                    origin => origin.clone(),
                };
                let value = inner.clone();
                return Ok(Applied::Ready(Held { value, origin }));
            }
            Builtin::Length => return self.length(arguments),
            Builtin::Get if !over_map => return self.element(arguments),
            Builtin::Get | Builtin::Contains if over_map => {
                let [map, key] = take(arguments);
                self.runtime.consume(&key)?;
                let get = builtin == Builtin::Get;
                return Ok(Applied::Job(Lookup::job(map, key.value, get)));
            }
            Builtin::Entries => {
                let [map] = take(arguments);
                self.runtime.read(&map, ExpressionReadKind::Length)?;
                return Ok(Applied::Job(Entries::job(map)));
            }
            _ => {}
        }
        for held in &arguments {
            self.runtime.consume(held)?;
        }
        let values: Vec<ExpressionValue> = arguments.into_iter().map(|held| held.value).collect();
        self.compute(builtin, ty, types, values)
    }

    fn compute(
        &mut self,
        builtin: Builtin,
        ty: &ExpressionType,
        types: &[&ExpressionType],
        values: Vec<ExpressionValue>,
    ) -> ExpressionResult<Applied> {
        let meter = &mut self.runtime.meter;
        let job = match builtin {
            Builtin::Min | Builtin::Max => {
                let [a, b] = take_values(values);
                Job::pick(builtin == Builtin::Max, a, b)
            }
            Builtin::MinOf | Builtin::MaxOf => Fold::extreme(builtin == Builtin::MaxOf, &values[0]),
            Builtin::Sum => Fold::sum(ty, &values[0]),
            Builtin::Clamp => Clamp::job(take_values(values)),
            Builtin::Contains => {
                let [collection, probe] = take_values(values);
                if matches!(types[0], ExpressionType::List(_)) {
                    Fold::member(&collection, probe)
                } else {
                    TextJob::contains(collection, probe, meter)?
                }
            }
            Builtin::StartsWith | Builtin::EndsWith => {
                let [text, affix] = take_values(values);
                TextJob::affix(text, affix, builtin == Builtin::EndsWith)
            }
            Builtin::Substring => {
                let [text, low, high] = take_values(values);
                let (low, high) = (integer_of(&low), integer_of(&high));
                let job = match &text.0 {
                    Repr::String(_) => TextJob::range(text, low, high),
                    _ => {
                        let length = text.logical_bytes() as i128;
                        (0 <= low && low <= high && high <= length)
                            .then(|| TextJob::copy(text, low as usize, high as usize))
                    }
                };
                match job {
                    Some(job) => job,
                    None => return ready(ExpressionValue::none()),
                }
            }
            _ => return ready(self.scalar(builtin, ty, values)?),
        };
        Ok(Applied::Job(job))
    }

    /// Builtins whose work the apply step already charged.
    fn scalar(
        &mut self,
        builtin: Builtin,
        ty: &ExpressionType,
        values: Vec<ExpressionValue>,
    ) -> ExpressionResult<ExpressionValue> {
        Ok(match builtin {
            Builtin::Abs => numbers::abs(ty, &values[0])?,
            Builtin::Sqrt => numbers::sqrt(ty, &values[0])?,
            Builtin::Near => {
                let [a, b, absolute, relative] = take_values(values);
                let relative = relative.as_f64().expect("admission typed this as Float64");
                let near = float::near(
                    magnitude_of(&a),
                    magnitude_of(&b),
                    magnitude_of(&absolute),
                    relative,
                )?;
                ExpressionValue::bool(near)
            }
            Builtin::DecimalDiv => {
                let [a, b, scale, mode] = take_values(values);
                let quotient = decimal::divide(
                    decimal_of(&a),
                    decimal_of(&b),
                    integer_of(&scale),
                    rounding(&mode),
                )?;
                numbers::decimal_value(quotient)
            }
            Builtin::Quantize => {
                let [value, scale, mode] = take_values(values);
                let scale = integer_of(&scale);
                numbers::decimal_value(decimal::quantize(
                    decimal_of(&value),
                    scale,
                    rounding(&mode),
                )?)
            }
            Builtin::Quantity(unit) => {
                numbers::quantity(quantity::to_canonical(magnitude_of(&values[0]), unit)?)
            }
            Builtin::Magnitude(unit) => {
                numbers::float64(quantity::in_unit(magnitude_of(&values[0]), unit)?)
            }
            Builtin::ExactCast | Builtin::RoundedCast => numbers::cast(builtin, ty, &values[0])?,
            _ => bus(builtin, ty, &values, &mut self.runtime.meter)?,
        })
    }

    /// `length`: scalars of a String, a read of a collection's length, or a
    /// bus's static width.
    fn length(&mut self, arguments: Vec<Held>) -> ExpressionResult<Applied> {
        let [value] = take(arguments);
        let length = match &value.value.0 {
            Repr::String(_) => {
                self.runtime.consume(&value)?;
                return Ok(Applied::Job(TextJob::count(value.value)));
            }
            Repr::Bits { width, .. } | Repr::Logic4 { width, .. } => i128::from(*width),
            Repr::Bytes(bytes) => bytes.len() as i128,
            Repr::List(parts) => parts.items.len() as i128,
            Repr::Map(parts) => parts.items.len() as i128 / 2,
            _ => unreachable!("admission measures only text, collections, and buses"),
        };
        if !matches!(&value.value.0, Repr::Bits { .. } | Repr::Logic4 { .. }) {
            self.runtime.read(&value, ExpressionReadKind::Length)?;
        }
        ready(ExpressionValue::integer(length))
    }

    /// List `get`: an in-range index reads a prefix long enough to hold the
    /// element, an index past the end reads the length, and a negative index
    /// reads nothing.
    fn element(&mut self, arguments: Vec<Held>) -> ExpressionResult<Applied> {
        let [list, index] = take(arguments);
        self.runtime.consume(&index)?;
        let index = integer_of(&index.value);
        let items = list
            .value
            .items()
            .expect("admission gets elements of lists");
        let Ok(position) = usize::try_from(index) else {
            return ready(ExpressionValue::none());
        };
        if position >= items.len() {
            self.runtime.read(&list, ExpressionReadKind::Length)?;
            return ready(ExpressionValue::none());
        }
        let prefix = ExpressionReadKind::Prefix(position as u64 + 1);
        self.runtime.read(&list, prefix)?;
        let meter = &mut self.runtime.meter;
        meter.visit(1)?;
        meter.read();
        let origin = match &list.origin {
            Origin::Path(path) => {
                let element = self
                    .runtime
                    .recorder
                    .element(*path, position as u64, meter)?;
                Origin::Path(element)
            }
            Origin::Parts(parts) => parts[position].clone(),
            Origin::Computed => Origin::Computed,
        };
        meter.allocate(16)?;
        let value = ExpressionValue::some(items[position].clone());
        Ok(Applied::Ready(Held {
            value,
            origin: parts(vec![origin]),
        }))
    }
}

fn ready(value: ExpressionValue) -> ExpressionResult<Applied> {
    Ok(Applied::Ready(Held::computed(value)))
}

fn take_values<const N: usize>(values: Vec<ExpressionValue>) -> [ExpressionValue; N] {
    values
        .try_into()
        .unwrap_or_else(|_| unreachable!("admission fixes each builtin's arity"))
}
