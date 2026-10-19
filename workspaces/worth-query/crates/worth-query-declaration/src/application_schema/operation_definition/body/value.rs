//! One authored value may feed many candidate sinks without repeated calculation.
mod authoring;
mod validation;
use super::{
    ApplicationOperationBodyExpression as Expression,
    ApplicationOperationBodyRequirementDenial as Denial,
    ApplicationOperationBodyRequirementOperand as Operand,
    ApplicationOperationBodyRequirementOperands,
};
use crate::{
    application_query::ApplicationQueryExpressionSnapshot,
    application_schema::ApplicationScalarValueBinding,
    portable_identity::WorthQueryPortableTypeIdentity,
};
use std::{marker::PhantomData, sync::Arc};
pub(super) use validation::validate_values;
use worth_foundational::{
    expression_api::{ExpressionDraft, ExpressionType, ExpressionValue, IntegerType},
    facade::ScalarAspectType,
};

pub struct ApplicationOperationBodyValue<Schema, Operation, Binding> {
    program: ApplicationOperationBodyValueProgram,
    marker: PhantomData<fn() -> (Schema, Operation, Binding)>,
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodyValueProgram {
    identity: String,
    binding: WorthQueryPortableTypeIdentity,
    operands: Vec<Operand>,
    expression: Expression,
}
impl<S, O, B: ApplicationScalarValueBinding> ApplicationOperationBodyValue<S, O, B> {
    pub fn admit(
        identity: &str,
        operands: ApplicationOperationBodyRequirementOperands<S, O>,
        expression: ExpressionDraft,
    ) -> Result<Self, Denial> {
        if B::SCALAR_FAMILY != ScalarAspectType::UInt64 || B::UNIT.is_some() || B::FRAME.is_some() {
            return Err(Denial::UnsupportedScalar);
        }
        Ok(Self {
            program: ApplicationOperationBodyValueProgram::from_untrusted(
                identity.to_owned(),
                B::IDENTITY,
                operands.into_operands(),
                expression.encode(),
            )?,
            marker: PhantomData,
        })
    }
}
impl ApplicationOperationBodyValueProgram {
    /// Pure transported meaning; this grants neither source nor mutation authority.
    pub fn from_untrusted(
        identity: String,
        binding: WorthQueryPortableTypeIdentity,
        mut operands: Vec<Operand>,
        draft: Vec<u8>,
    ) -> Result<Self, Denial> {
        if identity.is_empty() || identity.trim() != identity {
            return Err(Denial::InvalidName);
        }
        operands.sort_by(|a, b| a.name().cmp(b.name()));
        if operands.windows(2).any(|p| p[0].name() == p[1].name()) {
            return Err(Denial::DuplicateOperand);
        }
        let scope = operands.first().map(|o| o.source().scope_entity());
        if operands
            .iter()
            .any(|o| Some(o.source().scope_entity()) != scope)
        {
            return Err(Denial::ForeignScope);
        }
        if operands
            .iter()
            .filter(|o| {
                matches!(
                    o.source(),
                    super::ApplicationOperationBodyRequirementSource::QueryContext { .. }
                )
            })
            .count()
            > 1
        {
            return Err(Denial::MultipleQueryContexts);
        }
        let expression = Expression::decode(
            &draft,
            &operands,
            &ExpressionType::Integer(IntegerType::UInt64),
        )?;
        if operands.iter().any(|o| !expression.reads(o.name())) {
            return Err(Denial::UnusedOperand);
        }
        Ok(Self {
            identity,
            binding,
            operands,
            expression,
        })
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn binding(&self) -> &WorthQueryPortableTypeIdentity {
        &self.binding
    }
    pub fn operands(&self) -> &[Operand] {
        &self.operands
    }
    pub fn expression(&self) -> &Expression {
        &self.expression
    }
    pub(crate) fn canonical_record(&self) -> String {
        use std::fmt::Write;
        let mut result = String::new();
        let mut part =
            |value: &str| write!(&mut result, "{}:{value}", value.len()).expect("String write");
        part("operation-body-value-v1");
        part(self.identity());
        part(self.binding().as_str());
        part(&self.operands.len().to_string());
        super::requirement::encoding::append_operands(&mut part, self.operands());
        part(self.expression.semantic_material());
        result
    }
}

/// Descriptive prepared values. Public construction cannot supply a native decision.
#[derive(Clone, Debug)]
pub struct ApplicationOperationBodyValueSnapshot(Arc<[(String, ExpressionValue)]>);
impl ApplicationOperationBodyValueSnapshot {
    pub fn from_untrusted_values(values: Vec<(String, ExpressionValue)>) -> Self {
        Self(values.into())
    }
    pub fn value(&self, name: &str) -> Option<&ExpressionValue> {
        self.0
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value)
    }
}
/// Observed query data and computed decision values remain explicitly distinct.
#[derive(Clone, Debug)]
pub struct ApplicationOperationBodyEvaluationSnapshot {
    query: ApplicationQueryExpressionSnapshot,
    values: ApplicationOperationBodyValueSnapshot,
}
impl ApplicationOperationBodyEvaluationSnapshot {
    pub fn from_untrusted_parts(
        query: ApplicationQueryExpressionSnapshot,
        values: ApplicationOperationBodyValueSnapshot,
    ) -> Self {
        Self { query, values }
    }
    pub fn query(&self) -> &ApplicationQueryExpressionSnapshot {
        &self.query
    }
    pub fn value(&self, name: &str) -> Option<&ExpressionValue> {
        self.values.value(name)
    }
}
