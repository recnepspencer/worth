//! Query results a condition expression reads. Each operand names one
//! installed query; its single published result becomes one typed value.

use std::marker::PhantomData;

use worth_foundational::expression_api::{
    ExpressionDenial, ExpressionType, ExpressionValue, IntegerType,
};
use worth_foundational::facade::{CanonicalF32, CanonicalF64};

use crate::application_query::ApplicationQueryMarkerIdentity;
use crate::application_schema::ApplicationStructuredValueBinding;
use crate::portable_identity::WorthQueryPortableTypeIdentity;

use super::super::ApplicationWorkflowSpec;

/// A query result type a condition expression can read.
///
/// The value converts exactly; a result the language cannot represent, such
/// as a non-finite float, denies evaluation rather than reading as false.
pub trait ApplicationExpressionOperandValue: 'static {
    fn expression_type() -> ExpressionType;

    fn expression_value(&self) -> Result<ExpressionValue, ExpressionDenial>;
}

impl ApplicationExpressionOperandValue for bool {
    fn expression_type() -> ExpressionType {
        ExpressionType::Bool
    }

    fn expression_value(&self) -> Result<ExpressionValue, ExpressionDenial> {
        Ok(ExpressionValue::bool(*self))
    }
}

macro_rules! integer_operand {
    ($($value:ty => $kind:ident),* $(,)?) => {$(
        impl ApplicationExpressionOperandValue for $value {
            fn expression_type() -> ExpressionType {
                ExpressionType::Integer(IntegerType::$kind)
            }

            fn expression_value(&self) -> Result<ExpressionValue, ExpressionDenial> {
                Ok(ExpressionValue::integer(i128::from(*self)))
            }
        }
    )*};
}

integer_operand!(
    i8 => Int8, i16 => Int16, i32 => Int32, i64 => Int64,
    u8 => UInt8, u16 => UInt16, u32 => UInt32, u64 => UInt64,
);

impl ApplicationExpressionOperandValue for f32 {
    fn expression_type() -> ExpressionType {
        ExpressionType::Float32
    }

    fn expression_value(&self) -> Result<ExpressionValue, ExpressionDenial> {
        ExpressionValue::float32(CanonicalF32::from_f32(*self))
    }
}

impl ApplicationExpressionOperandValue for f64 {
    fn expression_type() -> ExpressionType {
        ExpressionType::Float64
    }

    fn expression_value(&self) -> Result<ExpressionValue, ExpressionDenial> {
        ExpressionValue::float64(CanonicalF64::from_f64(*self))
    }
}

impl ApplicationExpressionOperandValue for String {
    fn expression_type() -> ExpressionType {
        ExpressionType::String
    }

    fn expression_value(&self) -> Result<ExpressionValue, ExpressionDenial> {
        Ok(ExpressionValue::string(self.as_str()))
    }
}

impl<Value: ApplicationExpressionOperandValue> ApplicationExpressionOperandValue for Option<Value> {
    fn expression_type() -> ExpressionType {
        ExpressionType::option(Value::expression_type())
    }

    fn expression_value(&self) -> Result<ExpressionValue, ExpressionDenial> {
        match self {
            Some(value) => Ok(ExpressionValue::some(value.expression_value()?)),
            None => Ok(ExpressionValue::none()),
        }
    }
}

impl<Value: ApplicationExpressionOperandValue> ApplicationExpressionOperandValue for Vec<Value> {
    fn expression_type() -> ExpressionType {
        ExpressionType::list(Value::expression_type())
    }

    fn expression_value(&self) -> Result<ExpressionValue, ExpressionDenial> {
        self.iter()
            .map(ApplicationExpressionOperandValue::expression_value)
            .collect::<Result<_, _>>()
            .map(ExpressionValue::list)
    }
}

/// One installed query a condition may read, with its result's expression
/// type. It names the query; installation binds it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationWorkflowConditionQuery {
    identifier: &'static str,
    parameter_type: WorthQueryPortableTypeIdentity,
    result_type: WorthQueryPortableTypeIdentity,
    query_type: std::any::TypeId,
    expression_type: ExpressionType,
}

impl ApplicationWorkflowConditionQuery {
    pub fn declared<Spec, Query>() -> Self
    where
        Spec: ApplicationWorkflowSpec,
        Query: ApplicationQueryMarkerIdentity<Spec::Schema> + 'static,
        <Query::ResultBinding as ApplicationStructuredValueBinding>::Value:
            ApplicationExpressionOperandValue,
    {
        Self {
            identifier: Query::IDENTIFIER,
            parameter_type: Query::PARAMETER_TYPE_IDENTITY,
            result_type: Query::RESULT_TYPE_IDENTITY,
            query_type: std::any::TypeId::of::<Query>(),
            expression_type: <<Query::ResultBinding as ApplicationStructuredValueBinding>::Value as ApplicationExpressionOperandValue>::expression_type(),
        }
    }

    pub const fn identifier(&self) -> &'static str {
        self.identifier
    }

    pub const fn parameter_type(&self) -> &WorthQueryPortableTypeIdentity {
        &self.parameter_type
    }

    pub const fn result_type(&self) -> &WorthQueryPortableTypeIdentity {
        &self.result_type
    }

    pub const fn expression_type(&self) -> &ExpressionType {
        &self.expression_type
    }

    #[doc(hidden)]
    pub const fn query_type(&self) -> std::any::TypeId {
        self.query_type
    }
}

/// One named operand: the expression reads `name` as the result of `query`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationWorkflowConditionOperand {
    name: Box<str>,
    query: ApplicationWorkflowConditionQuery,
}

impl ApplicationWorkflowConditionOperand {
    pub fn new(name: impl Into<Box<str>>, query: ApplicationWorkflowConditionQuery) -> Self {
        Self {
            name: name.into(),
            query,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn query(&self) -> &ApplicationWorkflowConditionQuery {
        &self.query
    }
}

/// The typed operands a condition declares, each naming a query of `Spec`.
pub struct ApplicationWorkflowConditionOperands<Spec> {
    operands: Vec<ApplicationWorkflowConditionOperand>,
    spec: PhantomData<fn() -> Spec>,
}

impl<Spec: ApplicationWorkflowSpec> Default for ApplicationWorkflowConditionOperands<Spec> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Spec: ApplicationWorkflowSpec> ApplicationWorkflowConditionOperands<Spec> {
    pub const fn new() -> Self {
        Self {
            operands: Vec::new(),
            spec: PhantomData,
        }
    }

    /// Reads `Query`'s single result as the operand `name`.
    #[must_use]
    pub fn query<Query>(mut self, name: &str) -> Self
    where
        Query: ApplicationQueryMarkerIdentity<Spec::Schema> + 'static,
        <Query::ResultBinding as ApplicationStructuredValueBinding>::Value:
            ApplicationExpressionOperandValue,
    {
        self.operands.push(ApplicationWorkflowConditionOperand::new(
            name,
            ApplicationWorkflowConditionQuery::declared::<Spec, Query>(),
        ));
        self
    }

    pub fn into_operands(self) -> Vec<ApplicationWorkflowConditionOperand> {
        self.operands
    }
}
