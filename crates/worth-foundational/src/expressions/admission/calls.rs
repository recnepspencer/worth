//! Call resolution: reserved intrinsics, closed builtins, and installed
//! functions by exact signature.

use crate::expressions::denial::{ExpressionDenialDetail, ExpressionResult};
use crate::expressions::functions::Builtin;
use crate::expressions::program::Op;
use crate::expressions::syntax::ast::{NodeId, QualifiedName, TypeArgument};
use crate::expressions::types::{rounding_type, ExpressionType};

use super::Checker;

impl Checker<'_> {
    pub(super) fn call(
        &mut self,
        id: NodeId,
        function: &QualifiedName,
        type_arguments: &[TypeArgument],
        arguments: &[NodeId],
    ) -> ExpressionResult<u32> {
        let Some(name) = function.single() else {
            self.no_type_arguments(id, type_arguments)?;
            return self.installed(id, function, arguments);
        };
        if let Some(result) = self.constructor(id, name, type_arguments, arguments) {
            return result;
        }
        if let Some(result) = self.width_intrinsic(id, name, type_arguments, arguments) {
            return result;
        }
        self.no_type_arguments(id, type_arguments)?;
        if let Some(result) = self.bitwise(id, name, arguments) {
            return result;
        }
        self.builtin(id, name, arguments)
    }

    /// Builtins and installed functions have exact signatures without type
    /// parameters, so a type argument cannot mean anything.
    fn no_type_arguments(
        &self,
        id: NodeId,
        type_arguments: &[TypeArgument],
    ) -> ExpressionResult<()> {
        if type_arguments.is_empty() {
            Ok(())
        } else {
            Err(self.deny(
                id,
                ExpressionDenialDetail::UnsupportedFeature("this call takes no type arguments"),
            ))
        }
    }

    fn builtin(&mut self, id: NodeId, name: &str, arguments: &[NodeId]) -> ExpressionResult<u32> {
        let children = arguments
            .iter()
            .map(|argument| self.check(*argument, None))
            .collect::<ExpressionResult<Vec<_>>>()?;
        let types: Vec<ExpressionType> = children
            .iter()
            .map(|child| self.ty(*child).clone())
            .collect();
        let Some((builtin, ty)) = builtin_signature(name, &types) else {
            let detail = if super::super::functions::is_reserved_callable(name) {
                ExpressionDenialDetail::FunctionContractMismatch {
                    function: name.to_string(),
                    reason: "no builtin signature matches the argument types",
                }
            } else {
                ExpressionDenialDetail::UnknownBinding(name.to_string())
            };
            return Err(self.deny(id, detail));
        };
        self.emit(id, Op::Builtin(builtin), ty, children)
    }

    fn installed(
        &mut self,
        id: NodeId,
        function: &QualifiedName,
        arguments: &[NodeId],
    ) -> ExpressionResult<u32> {
        let name = function.text();
        let catalog = self.context.catalog;
        let candidates: Vec<usize> = catalog
            .overloads(&name)
            .filter(|index| catalog.function(*index).parameters().len() == arguments.len())
            .collect();
        if catalog.overloads(&name).next().is_none() {
            return Err(self.deny(id, ExpressionDenialDetail::UnknownBinding(name)));
        }
        // An argument checks against its parameter type only where every
        // same-arity overload declares that type, so the denial family for a
        // mismatch depends on the declared signatures, not on their count.
        let mut children = Vec::with_capacity(arguments.len());
        for (position, argument) in arguments.iter().enumerate() {
            let mut declared = candidates
                .iter()
                .map(|index| &catalog.function(*index).parameters()[position].1);
            let first = declared.next();
            let expected = first.filter(|first| declared.all(|ty| ty == *first));
            children.push(self.check(*argument, expected)?);
        }
        let matched = candidates.into_iter().find(|index| {
            let parameters = catalog.function(*index).parameters();
            children
                .iter()
                .zip(parameters)
                .all(|(child, (_, ty))| self.ty(*child) == ty)
        });
        let Some(index) = matched else {
            return Err(self.deny(
                id,
                ExpressionDenialDetail::FunctionContractMismatch {
                    function: name,
                    reason: "no installed signature matches the argument types",
                },
            ));
        };
        self.functions.insert(index);
        let ty = catalog.function(index).result().clone();
        self.emit(id, Op::Call(index as u32), ty, children)
    }
}

/// The closed numeric, option, string, and collection builtin signatures.
fn builtin_signature(name: &str, types: &[ExpressionType]) -> Option<(Builtin, ExpressionType)> {
    use ExpressionType as T;
    let integer = |ty: &T| matches!(ty, T::Integer(_));
    let summable = |ty: &T| {
        matches!(
            ty,
            T::Integer(_) | T::Float32 | T::Float64 | T::Decimal | T::Quantity(_)
        )
    };
    let signed = |ty: &T| match ty {
        T::Integer(integer) => integer.is_signed(),
        other => summable(other),
    };
    let rounding = rounding_type();
    Some(match (name, types) {
        ("min" | "max", [T::List(element)]) if element.is_ordered() => {
            let builtin = if name == "min" {
                Builtin::MinOf
            } else {
                Builtin::MaxOf
            };
            (builtin, T::Option(element.clone()))
        }
        ("min" | "max", [a, b]) if a == b && a.is_ordered() => {
            let builtin = if name == "min" {
                Builtin::Min
            } else {
                Builtin::Max
            };
            (builtin, a.clone())
        }
        ("abs", [a]) if signed(a) => (Builtin::Abs, a.clone()),
        ("clamp", [x, low, high]) if x == low && x == high && x.is_ordered() => {
            (Builtin::Clamp, x.clone())
        }
        ("sqrt", [a]) if a.is_float() => (Builtin::Sqrt, a.clone()),
        ("sqrt", [T::Quantity(dimension)]) => {
            (Builtin::Sqrt, T::Quantity(dimension.square_root()?))
        }
        ("near", [a, b, tolerance, T::Float64])
            if a == b && a == tolerance && matches!(a, T::Float64 | T::Quantity(_)) =>
        {
            (Builtin::Near, T::Bool)
        }
        ("sum", [T::List(element)]) if summable(element) => (Builtin::Sum, (**element).clone()),
        ("decimal_div", [T::Decimal, T::Decimal, scale, mode])
            if integer(scale) && *mode == rounding =>
        {
            (Builtin::DecimalDiv, T::Decimal)
        }
        ("quantize", [T::Decimal, scale, mode]) if integer(scale) && *mode == rounding => {
            (Builtin::Quantize, T::Decimal)
        }
        ("is_some", [T::Option(_)]) => (Builtin::IsSome, T::Bool),
        ("some", [a]) => (Builtin::Some, T::option(a.clone())),
        ("unwrap", [T::Option(inner)]) => (Builtin::Unwrap, (**inner).clone()),
        (
            "length",
            [T::String | T::Bytes | T::List(_) | T::Map(..) | T::Bits(_) | T::Logic4(_)],
        ) => (Builtin::Length, T::INT64),
        ("get", [T::List(element), index]) if integer(index) => {
            (Builtin::Get, T::Option(element.clone()))
        }
        ("get", [T::Map(key, value), probe]) if **key == *probe => {
            (Builtin::Get, T::Option(value.clone()))
        }
        ("contains", [T::List(element), probe]) if **element == *probe => {
            (Builtin::Contains, T::Bool)
        }
        ("contains", [T::Map(key, _), probe]) if **key == *probe => (Builtin::Contains, T::Bool),
        ("contains", [T::String, T::String] | [T::Bytes, T::Bytes]) => (Builtin::Contains, T::Bool),
        ("entries", [T::Map(key, value)]) => (
            Builtin::Entries,
            T::list(T::MapEntry(key.clone(), value.clone())),
        ),
        ("starts_with" | "ends_with", [T::String, T::String] | [T::Bytes, T::Bytes]) => {
            let builtin = if name == "starts_with" {
                Builtin::StartsWith
            } else {
                Builtin::EndsWith
            };
            (builtin, T::Bool)
        }
        ("slice", [text @ (T::String | T::Bytes), low, high]) if integer(low) && integer(high) => {
            (Builtin::Substring, T::option(text.clone()))
        }
        _ => return None,
    })
}
