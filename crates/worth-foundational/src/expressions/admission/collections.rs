//! List, map, and record literal typing.

use std::collections::BTreeMap;

use crate::expressions::denial::{ExpressionDenialDetail, ExpressionResult};
use crate::expressions::program::{Literal, Op};
use crate::expressions::syntax::ast::{NodeId, QualifiedName};
use crate::expressions::types::ExpressionType;

use super::Checker;

impl Checker<'_> {
    pub(super) fn list(
        &mut self,
        id: NodeId,
        items: &[NodeId],
        expected: Option<&ExpressionType>,
    ) -> ExpressionResult<u32> {
        let mut element = match expected {
            Some(ExpressionType::List(element)) => Some((**element).clone()),
            _ => None,
        };
        if items.is_empty() && element.is_none() {
            return Err(self.deny(id, ExpressionDenialDetail::TypeRequired("an empty list")));
        }
        let mut children = Vec::with_capacity(items.len());
        for item in items {
            let child = self.check(*item, element.as_ref())?;
            element.get_or_insert_with(|| self.ty(child).clone());
            children.push(child);
        }
        let ty = ExpressionType::list(element.expect("a nonempty list or an expected element"));
        self.emit(id, Op::List, ty, children)
    }

    pub(super) fn map(
        &mut self,
        id: NodeId,
        entries: &[(NodeId, NodeId)],
        expected: Option<&ExpressionType>,
    ) -> ExpressionResult<u32> {
        let (mut key_type, mut value_type) = match expected {
            Some(ExpressionType::Map(key, value)) => {
                (Some((**key).clone()), Some((**value).clone()))
            }
            _ => (None, None),
        };
        if entries.is_empty() && key_type.is_none() {
            return Err(self.deny(id, ExpressionDenialDetail::TypeRequired("an empty map")));
        }
        let mut children = Vec::with_capacity(entries.len() * 2);
        for (key, value) in entries {
            let key = self.check(*key, key_type.as_ref())?;
            if !self.ty(key).is_map_key() {
                return Err(self.expected_kind(id, "a String, integer, or ID map key", key));
            }
            key_type.get_or_insert_with(|| self.ty(key).clone());
            let value = self.check(*value, value_type.as_ref())?;
            value_type.get_or_insert_with(|| self.ty(value).clone());
            children.extend([key, value]);
        }
        let ty = ExpressionType::map(
            key_type.expect("a nonempty map or an expected key"),
            value_type.expect("a nonempty map or an expected value"),
        );
        self.emit(id, Op::Map, ty, children)
    }

    /// Fields evaluate in schema order; omitted Option fields are `none`.
    pub(super) fn record(
        &mut self,
        id: NodeId,
        type_name: &QualifiedName,
        fields: &[(Box<str>, NodeId)],
    ) -> ExpressionResult<u32> {
        let name = type_name.text();
        let declaration =
            self.schema().record(&name).cloned().ok_or_else(|| {
                self.deny(id, ExpressionDenialDetail::UnknownBinding(name.clone()))
            })?;
        // Field lookup is linear in the declaration; charge the matching work.
        self.meter
            .charge((fields.len() as u64).saturating_mul(declaration.fields().len() as u64))?;
        let mut authored = BTreeMap::new();
        for (field, value) in fields {
            if authored.insert(&**field, *value).is_some() {
                return Err(self.deny(
                    id,
                    ExpressionDenialDetail::AmbiguousBinding(field.to_string()),
                ));
            }
        }
        if let Some(unknown) = authored
            .keys()
            .find(|field| declaration.field(field).is_none())
        {
            return Err(self.deny(
                id,
                ExpressionDenialDetail::UnknownBinding(unknown.to_string()),
            ));
        }
        let mut children = Vec::with_capacity(declaration.fields().len());
        for (field, ty) in declaration.fields() {
            let child = match authored.get(field) {
                Some(value) => self.check(*value, Some(ty))?,
                None if matches!(ty, ExpressionType::Option(_)) => {
                    self.literal(id, Literal::None, ty.clone())?
                }
                None => {
                    return Err(self.deny(
                        id,
                        ExpressionDenialDetail::MissingOperand(field.to_string()),
                    ));
                }
            };
            children.push(child);
        }
        let ty = ExpressionType::Record(declaration.name().clone());
        self.emit(id, Op::Record, ty, children)
    }
}
