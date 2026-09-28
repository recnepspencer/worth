//! Digital-logic builtins over `Bits<N>` and `Logic4<N>`. Binary operations
//! require equal widths; nothing coerces between buses, integers, and Bool.

use crate::expressions::denial::{ExpressionDenialDetail, ExpressionResource, ExpressionResult};
use crate::expressions::functions::Builtin;
use crate::expressions::profile::check_limit;
use crate::expressions::program::Op;
use crate::expressions::syntax::ast::NodeId;
use crate::expressions::types::ExpressionType;

use super::Checker;

impl Checker<'_> {
    pub(super) fn bitwise(
        &mut self,
        id: NodeId,
        name: &str,
        arguments: &[NodeId],
    ) -> Option<ExpressionResult<u32>> {
        let builtin = match name {
            "bit_and" => Builtin::BitAnd,
            "bit_or" => Builtin::BitOr,
            "bit_xor" => Builtin::BitXor,
            "bit_not" => Builtin::BitNot,
            "shift_left" => Builtin::ShiftLeft,
            "shift_right" => Builtin::ShiftRight,
            "concat" => Builtin::Concat,
            "bits_add" => Builtin::BitsAdd,
            "wrapping_add" => Builtin::WrappingAdd,
            "logic_eq" => Builtin::LogicEq,
            "case_equal" => Builtin::CaseEqual,
            "mux" => Builtin::Mux,
            _ => return None,
        };
        let children = arguments
            .iter()
            .map(|argument| self.check(*argument, None))
            .collect::<ExpressionResult<Vec<_>>>();
        Some(children.and_then(|children| self.bitwise_signature(id, name, builtin, children)))
    }

    fn bitwise_signature(
        &mut self,
        id: NodeId,
        name: &str,
        builtin: Builtin,
        children: Vec<u32>,
    ) -> ExpressionResult<u32> {
        use ExpressionType as T;
        let types: Vec<&T> = children.iter().map(|child| self.ty(*child)).collect();
        let bus = |ty: &T| matches!(ty, T::Bits(_) | T::Logic4(_));
        let ty = match (builtin, types.as_slice()) {
            (Builtin::BitAnd | Builtin::BitOr | Builtin::BitXor, [a, b]) if a == b && bus(a) => {
                (*a).clone()
            }
            (Builtin::BitNot, [a]) if bus(a) => (*a).clone(),
            (Builtin::ShiftLeft | Builtin::ShiftRight, [a @ T::Bits(_), T::Integer(_)]) => {
                (*a).clone()
            }
            (Builtin::BitsAdd | Builtin::WrappingAdd, [a @ T::Bits(_), b]) if a == b => {
                (*a).clone()
            }
            (Builtin::LogicEq, [a @ T::Logic4(_), b]) if a == b => T::Logic4(1),
            (Builtin::CaseEqual, [a @ T::Logic4(_), b]) if a == b => T::Bool,
            (Builtin::Mux, [T::Logic4(1), a @ T::Logic4(_), b]) if a == b => (*a).clone(),
            (Builtin::Concat, [T::Bits(high), T::Bits(low)]) => {
                T::Bits(self.concat_width(*high, *low)?)
            }
            (Builtin::Concat, [T::Logic4(high), T::Logic4(low)]) => {
                T::Logic4(self.concat_width(*high, *low)?)
            }
            _ => {
                return Err(self.deny(
                    id,
                    ExpressionDenialDetail::FunctionContractMismatch {
                        function: name.to_string(),
                        reason: "no digital-logic signature matches the argument types",
                    },
                ));
            }
        };
        self.emit(id, Op::Builtin(builtin), ty, children)
    }

    /// `concat(high, low)`: the first argument supplies the high bits.
    fn concat_width(&self, high: u32, low: u32) -> ExpressionResult<u32> {
        let width = u64::from(high) + u64::from(low);
        check_limit(self.context.profile, ExpressionResource::BitWidth, width)?;
        Ok(width as u32)
    }
}
