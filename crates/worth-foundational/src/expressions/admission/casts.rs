//! Width- and type-parameterized intrinsics: explicit casts and static bit
//! width changes. Every conversion is spelled; nothing converts implicitly.

use crate::expressions::denial::{ExpressionDenialDetail, ExpressionResource, ExpressionResult};
use crate::expressions::functions::Builtin;
use crate::expressions::profile::check_limit;
use crate::expressions::program::Op;
use crate::expressions::syntax::ast::{NodeId, TypeArgument};
use crate::expressions::types::{resolve_type, ExpressionType};

use super::Checker;

impl Checker<'_> {
    pub(super) fn width_intrinsic(
        &mut self,
        id: NodeId,
        name: &str,
        type_arguments: &[TypeArgument],
        arguments: &[NodeId],
    ) -> Option<ExpressionResult<u32>> {
        let result = match (name, type_arguments, arguments) {
            ("exact_cast" | "rounded_cast", [TypeArgument::Type(target)], [value]) => {
                let rounded = name == "rounded_cast";
                resolve_type(self.schema(), target, id.0)
                    .and_then(|target| self.cast(id, rounded, &target, *value))
            }
            ("truncate" | "extend", [TypeArgument::Width(width)], [value]) => {
                self.resize(id, name == "extend", *width, *value)
            }
            ("to_bits", [TypeArgument::Width(width)], [value]) => self.to_bits(id, *width, *value),
            ("slice", [TypeArgument::Width(low), TypeArgument::Width(high)], [value]) => {
                self.bit_slice(id, *low, *high, *value)
            }
            ("slice", [], _) => return None,
            ("exact_cast" | "rounded_cast" | "truncate" | "extend" | "to_bits" | "slice", ..) => {
                Err(self.deny(id, ExpressionDenialDetail::TypeRequired(generic_form(name))))
            }
            _ => return None,
        };
        Some(result)
    }

    fn cast(
        &mut self,
        id: NodeId,
        rounded: bool,
        target: &ExpressionType,
        value: NodeId,
    ) -> ExpressionResult<u32> {
        let value = self.check(value, None)?;
        let source = self.ty(value);
        let allowed = if rounded {
            rounded_cast_allowed(source, target)
        } else {
            exact_cast_allowed(source, target)
        };
        if !allowed {
            let function = if rounded {
                "rounded_cast"
            } else {
                "exact_cast"
            };
            return Err(self.deny(
                id,
                ExpressionDenialDetail::FunctionContractMismatch {
                    function: function.to_string(),
                    reason: "no conversion between these types",
                },
            ));
        }
        let builtin = if rounded {
            Builtin::RoundedCast
        } else {
            Builtin::ExactCast
        };
        self.emit(id, Op::Builtin(builtin), target.clone(), vec![value])
    }

    /// `truncate` drops high bits; `extend` zero-fills them. Both change the
    /// width strictly, so a no-op resize denies as a likely mistake.
    fn resize(
        &mut self,
        id: NodeId,
        extend: bool,
        width: u32,
        value: NodeId,
    ) -> ExpressionResult<u32> {
        check_limit(
            self.context.profile,
            ExpressionResource::BitWidth,
            u64::from(width),
        )?;
        let value = self.check(value, None)?;
        let (ty, current) = match *self.ty(value) {
            ExpressionType::Bits(current) => (ExpressionType::Bits(width), current),
            ExpressionType::Logic4(current) => (ExpressionType::Logic4(width), current),
            _ => return Err(self.expected_kind(id, "Bits or Logic4", value)),
        };
        let valid = if extend {
            width > current
        } else {
            width < current && width > 0
        };
        if !valid {
            let reason = if extend {
                "extend widens to a strictly greater width"
            } else {
                "truncate narrows to a strictly smaller nonzero width"
            };
            return Err(self.deny(id, ExpressionDenialDetail::Bounds(reason)));
        }
        let builtin = if extend {
            Builtin::Extend
        } else {
            Builtin::Truncate
        };
        self.emit(id, Op::Builtin(builtin), ty, vec![value])
    }

    /// Checked: a negative value or one that needs more than `width` bits
    /// denies at evaluation.
    fn to_bits(&mut self, id: NodeId, width: u32, value: NodeId) -> ExpressionResult<u32> {
        if width == 0 {
            return Err(self.deny(
                id,
                ExpressionDenialDetail::Bounds("bit widths are at least one"),
            ));
        }
        check_limit(
            self.context.profile,
            ExpressionResource::BitWidth,
            u64::from(width),
        )?;
        let value = self.check(value, None)?;
        if !matches!(self.ty(value), ExpressionType::Integer(_)) {
            return Err(self.expected_kind(id, "an integer type", value));
        }
        self.emit(
            id,
            Op::Builtin(Builtin::ToBits),
            ExpressionType::Bits(width),
            vec![value],
        )
    }

    /// Half-open `[low, high)`, checked statically against the input width.
    fn bit_slice(
        &mut self,
        id: NodeId,
        low: u32,
        high: u32,
        value: NodeId,
    ) -> ExpressionResult<u32> {
        let value = self.check(value, None)?;
        let (width, logic) = match *self.ty(value) {
            ExpressionType::Bits(width) => (width, false),
            ExpressionType::Logic4(width) => (width, true),
            _ => return Err(self.expected_kind(id, "Bits or Logic4", value)),
        };
        if low >= high || high > width {
            return Err(self.deny(
                id,
                ExpressionDenialDetail::Bounds("bit slice bounds are outside the width"),
            ));
        }
        let ty = if logic {
            ExpressionType::Logic4(high - low)
        } else {
            ExpressionType::Bits(high - low)
        };
        self.emit(id, Op::Builtin(Builtin::BitSlice { low }), ty, vec![value])
    }
}

fn generic_form(name: &str) -> &'static str {
    match name {
        "exact_cast" => "exact_cast<T>(value)",
        "rounded_cast" => "rounded_cast<T>(value)",
        "truncate" => "truncate<N>(bits)",
        "extend" => "extend<N>(bits)",
        "to_bits" => "to_bits<N>(integer)",
        _ => "slice<low, high>(bits)",
    }
}

/// Exact casts check range, integrality, and precision at evaluation.
fn exact_cast_allowed(source: &ExpressionType, target: &ExpressionType) -> bool {
    use ExpressionType as T;
    let numeric = |ty: &T| matches!(ty, T::Integer(_) | T::Float32 | T::Float64 | T::Decimal);
    match (source, target) {
        _ if source == target => false,
        (T::Bits(_), T::Integer(integer)) => !integer.is_signed(),
        _ => numeric(source) && numeric(target),
    }
}

/// Rounded casts round once, nearest-even, into a binary float.
fn rounded_cast_allowed(source: &ExpressionType, target: &ExpressionType) -> bool {
    use ExpressionType as T;
    matches!(
        (source, target),
        (T::Integer(_) | T::Decimal, T::Float32 | T::Float64) | (T::Float64, T::Float32)
    )
}
