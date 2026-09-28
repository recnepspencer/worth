//! `Bits<N>` and `Logic4<N>` builtins over evaluation values. Their work is
//! the static per-limb cost the apply step already charged; each allocates
//! its result planes.

use crate::expressions::denial::ExpressionResult;
use crate::expressions::functions::Builtin;
use crate::expressions::operators::digital::{self, Plane};
use crate::expressions::types::ExpressionType;

use super::cost::planes;
use super::meter::EvaluationMeter;
use super::numbers::integer_of;
use super::value::{ExpressionValue, Repr};

/// `(width, value plane, unknown plane)`; Bits have no unknown plane.
fn split(value: &ExpressionValue) -> (u32, &[u64], Option<&[u64]>) {
    match &value.0 {
        Repr::Bits { width, limbs } => (*width, limbs, None),
        Repr::Logic4 {
            width,
            value,
            unknown,
        } => (*width, value, Some(unknown)),
        _ => unreachable!("admission applies bus builtins only to buses"),
    }
}

fn logic(value: &ExpressionValue) -> digital::Logic<'_> {
    match split(value) {
        (_, value, Some(unknown)) => (value, unknown),
        _ => unreachable!("admission typed this operand as Logic4"),
    }
}

/// Applies `plane` to the value plane and, for Logic4, the unknown plane.
fn each(value: &ExpressionValue, plane: impl Fn(&[u64]) -> Plane) -> (Plane, Option<Plane>) {
    let (_, value, unknown) = split(value);
    (plane(value), unknown.map(plane))
}

pub(super) fn bus(
    builtin: Builtin,
    ty: &ExpressionType,
    values: &[ExpressionValue],
    meter: &mut EvaluationMeter,
) -> ExpressionResult<ExpressionValue> {
    if builtin == Builtin::CaseEqual {
        // Equal widths, value planes, and unknown planes.
        return Ok(ExpressionValue::bool(values[0] == values[1]));
    }
    let (width, is_logic) = match ty {
        ExpressionType::Bits(width) => (*width, false),
        ExpressionType::Logic4(width) => (*width, true),
        _ => unreachable!("admission gives bus builtins bus results"),
    };
    meter.allocate(8 * planes(ty))?;
    let (value, unknown) = match builtin {
        Builtin::ToBits => (digital::to_bits(integer_of(&values[0]), width)?, None),
        Builtin::Truncate => each(&values[0], |plane| digital::slice(plane, 0, width)),
        Builtin::Extend => each(&values[0], |plane| digital::extend(plane, width)),
        Builtin::BitSlice { low } => each(&values[0], |plane| digital::slice(plane, low, width)),
        Builtin::Concat => {
            let ((low_width, low, low_unknown), (_, high, high_unknown)) =
                (split(&values[1]), split(&values[0]));
            let join = |high, low| digital::concat(high, low, low_width, width);
            (
                join(high, low),
                high_unknown.zip(low_unknown).map(|(h, l)| join(h, l)),
            )
        }
        Builtin::ShiftLeft | Builtin::ShiftRight => {
            let (_, plane, _) = split(&values[0]);
            let count = integer_of(&values[1]);
            let shifted = if builtin == Builtin::ShiftLeft {
                digital::shift_left(width, plane, count)?
            } else {
                digital::shift_right(width, plane, count)?
            };
            (shifted, None)
        }
        Builtin::BitsAdd | Builtin::WrappingAdd => {
            let ((_, a, _), (_, b, _)) = (split(&values[0]), split(&values[1]));
            let wrapping = builtin == Builtin::WrappingAdd;
            (digital::add(width, a, b, wrapping)?, None)
        }
        Builtin::LogicEq => {
            let (value, unknown) = digital::logic_eq(logic(&values[0]), logic(&values[1]));
            (vec![value], Some(vec![unknown]))
        }
        Builtin::Mux => {
            let (select_value, select_unknown) = logic(&values[0]);
            let select = (select_value[0], select_unknown[0]);
            let (value, unknown) =
                digital::mux(width, select, logic(&values[1]), logic(&values[2]));
            (value, Some(unknown))
        }
        Builtin::BitNot if is_logic => {
            let (value, unknown) = digital::logic_not(width, logic(&values[0]));
            (value, Some(unknown))
        }
        Builtin::BitNot => (digital::not(width, split(&values[0]).1), None),
        Builtin::BitAnd | Builtin::BitOr | Builtin::BitXor if is_logic => {
            let (a, b) = (logic(&values[0]), logic(&values[1]));
            let (value, unknown) = match builtin {
                Builtin::BitAnd => digital::logic_and(width, a, b),
                Builtin::BitOr => digital::logic_or(width, a, b),
                _ => digital::logic_xor(width, a, b),
            };
            (value, Some(unknown))
        }
        Builtin::BitAnd | Builtin::BitOr | Builtin::BitXor => {
            let (a, b) = (split(&values[0]).1, split(&values[1]).1);
            let plane = match builtin {
                Builtin::BitAnd => digital::and(a, b),
                Builtin::BitOr => digital::or(a, b),
                _ => digital::xor(a, b),
            };
            (plane, None)
        }
        _ => unreachable!("not a bus builtin"),
    };
    Ok(ExpressionValue(match unknown {
        Some(unknown) => Repr::Logic4 {
            width,
            value: value.into(),
            unknown: unknown.into(),
        },
        None => Repr::Bits {
            width,
            limbs: value.into(),
        },
    }))
}
