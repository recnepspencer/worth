//! Name resolution, field access, and lexical binders.
//!
//! Resolution order: innermost binder, then operand or parameter, then enum
//! variant. A name that is both an operand and an enum variant is ambiguous.

use crate::expressions::denial::{ExpressionDenialDetail, ExpressionResult};
use crate::expressions::functions::is_reserved_callable;
use crate::expressions::program::{Literal, Op};
use crate::expressions::syntax::ast::{ComprehensionKind, NodeId, QualifiedName};
use crate::expressions::syntax::GENERIC_INTRINSICS;
use crate::expressions::types::{rounding_type, ExpressionType, ROUNDING_NAME, ROUNDING_VARIANTS};

use super::{Checker, Scope};

impl Checker<'_> {
    pub(super) fn name(&mut self, id: NodeId, name: &QualifiedName) -> ExpressionResult<u32> {
        if let Some(single) = name.single() {
            self.meter.charge(self.binders.len() as u64)?;
            if let Some(position) = self
                .binders
                .iter()
                .rposition(|(binder, _)| &**binder == single)
            {
                let ty = self.binders[position].1.clone();
                let index = (self.binders.len() - 1 - position) as u32;
                return self.emit(id, Op::Local(index), ty, Vec::new());
            }
        }
        let text = name.text();
        self.meter.charge(text.len() as u64)?;
        let operand = self.lookup_operand(&text)?;
        let variant = self.lookup_variant(name)?;
        match (operand, variant) {
            (Some(_), Some(_)) => {
                Err(self.deny(id, ExpressionDenialDetail::AmbiguousBinding(text)))
            }
            (Some((index, ty)), None) => {
                self.operands.insert(index);
                self.emit(id, Op::Operand(index as u32), ty, Vec::new())
            }
            (None, Some((variant, ty))) => self.literal(id, Literal::Enum(variant), ty),
            (None, None) => Err(self.deny(id, ExpressionDenialDetail::UnknownBinding(text))),
        }
    }

    fn lookup_operand(&mut self, name: &str) -> ExpressionResult<Option<(usize, ExpressionType)>> {
        match self.scope {
            Scope::Operands => {
                self.probe(self.schema().operands().len())?;
                let schema = self.schema();
                Ok(schema
                    .operand_index(name)
                    .and_then(|index| Some((index, schema.operand_at(index)?.1.clone()))))
            }
            Scope::Parameters(parameters) => {
                self.meter.charge(parameters.len() as u64)?;
                Ok(parameters
                    .iter()
                    .position(|(parameter, _)| &**parameter == name)
                    .map(|index| (index, parameters[index].1.clone())))
            }
        }
    }

    fn lookup_variant(
        &mut self,
        name: &QualifiedName,
    ) -> ExpressionResult<Option<(u32, ExpressionType)>> {
        let Some((variant, prefix)) = name.0.split_last() else {
            return Ok(None);
        };
        if prefix.is_empty() {
            return Ok(None);
        }
        let enumeration = prefix.join("::");
        self.probe(self.schema().nominal_count())?;
        if let Some(declaration) = self.schema().enumeration(&enumeration) {
            self.probe(declaration.variants().len())?;
        }
        Ok(self.resolve_variant(variant, &enumeration))
    }

    fn resolve_variant(&self, variant: &str, enumeration: &str) -> Option<(u32, ExpressionType)> {
        if enumeration == ROUNDING_NAME {
            let index = ROUNDING_VARIANTS
                .iter()
                .position(|known| *known == variant)?;
            return Some((index as u32, rounding_type()));
        }
        let declaration = self.schema().enumeration(enumeration)?;
        let index = declaration.variant(variant)?;
        Some((
            index as u32,
            ExpressionType::Enum(declaration.name().clone()),
        ))
    }

    pub(super) fn field(&mut self, id: NodeId, base: NodeId, field: &str) -> ExpressionResult<u32> {
        let base = self.check(base, None)?;
        let resolved = match self.ty(base) {
            ExpressionType::Record(name) => {
                let record = self.schema().record(name.name());
                self.probe(self.schema().nominal_count())?;
                self.probe(record.map_or(0, |record| record.fields().len()))?;
                record
                    .and_then(|record| record.field(field))
                    .map(|(index, ty)| (index as u32, ty.clone()))
            }
            ExpressionType::MapEntry(key, value) => match field {
                "key" => Some((0, (**key).clone())),
                "value" => Some((1, (**value).clone())),
                _ => None,
            },
            other => {
                return Err(self.deny(
                    id,
                    ExpressionDenialDetail::TypeMismatch {
                        expected: "a record".to_string(),
                        found: other.to_string(),
                    },
                ));
            }
        };
        let (index, ty) = resolved.ok_or_else(|| {
            self.deny(
                id,
                ExpressionDenialDetail::UnknownBinding(field.to_string()),
            )
        })?;
        self.emit(id, Op::Field(index), ty, vec![base])
    }

    pub(super) fn comprehension(
        &mut self,
        id: NodeId,
        base: NodeId,
        kind: ComprehensionKind,
        binder: &str,
        body: NodeId,
    ) -> ExpressionResult<u32> {
        self.check_binder(id, binder)?;
        let base = self.check(base, None)?;
        let ExpressionType::List(element) = self.ty(base).clone() else {
            return Err(self.deny(
                id,
                ExpressionDenialDetail::TypeMismatch {
                    expected: "a List".to_string(),
                    found: self.ty(base).to_string(),
                },
            ));
        };
        let body_expected = match kind {
            ComprehensionKind::Map => None,
            _ => Some(ExpressionType::Bool),
        };
        self.binders.push((binder.into(), (*element).clone()));
        let body = self.check(body, body_expected.as_ref());
        self.binders.pop();
        let body = body?;
        let ty = match kind {
            ComprehensionKind::Map => ExpressionType::list(self.ty(body).clone()),
            ComprehensionKind::Filter => ExpressionType::List(element),
            ComprehensionKind::All | ComprehensionKind::Any => ExpressionType::Bool,
        };
        self.emit(id, Op::Comprehension(kind), ty, vec![base, body])
    }

    pub(super) fn let_binding(
        &mut self,
        id: NodeId,
        binder: &str,
        value: NodeId,
        body: NodeId,
        expected: Option<&ExpressionType>,
    ) -> ExpressionResult<u32> {
        self.check_binder(id, binder)?;
        let value = self.check(value, None)?;
        self.binders.push((binder.into(), self.ty(value).clone()));
        let body = self.check(body, expected);
        self.binders.pop();
        let body = body?;
        let ty = self.ty(body).clone();
        self.emit(id, Op::Let, ty, vec![value, body])
    }

    /// Binders are nonshadowing: they cannot reuse an in-scope binder, an
    /// operand or parameter, or a reserved callable name.
    fn check_binder(&mut self, id: NodeId, binder: &str) -> ExpressionResult<()> {
        self.meter.charge(self.binders.len() as u64)?;
        let shadows = self
            .binders
            .iter()
            .any(|(existing, _)| &**existing == binder)
            || self.lookup_operand(binder)?.is_some()
            || is_reserved_callable(binder)
            || GENERIC_INTRINSICS.contains(&binder);
        if shadows {
            Err(self.deny(
                id,
                ExpressionDenialDetail::AmbiguousBinding(binder.to_string()),
            ))
        } else {
            Ok(())
        }
    }
}
