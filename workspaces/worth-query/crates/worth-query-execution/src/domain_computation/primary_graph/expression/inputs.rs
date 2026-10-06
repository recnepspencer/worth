use std::marker::PhantomData;

use worth_foundational::expression_api::{ExpressionDenial, ExpressionType, ExpressionValue};
use worth_query_declaration::facade::{
    application_program::ApplicationExpressionOperandValue,
    application_query::{ApplicationQueryBinding, ApplicationQueryMarkerIdentity},
    application_schema::ApplicationStructuredValueBinding,
};
use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::primary_graph::{
    workflow::definition::WorkflowConditionOperand, RequiredWorkflowCondition,
    WorthQueryApplicationOutputDemandSource, WorthQueryObservedSource,
};

/// The published query results a workflow condition reads, one per named
/// operand. Each result must be exactly one row with its source; the
/// condition settles only when the operands match the installed requirement.
pub struct WorthQueryWorkflowConditionSources<Schema> {
    operands: Vec<ConditionOperandSource>,
    schema: PhantomData<fn() -> Schema>,
}

/// One supplied operand: the installed contract it claims and its single
/// observed value, or `None` when the result was not exactly one sourced row.
pub(in crate::domain_computation::primary_graph) struct ConditionOperandSource {
    pub(in crate::domain_computation::primary_graph) name: Box<str>,
    pub(in crate::domain_computation::primary_graph) query: &'static str,
    pub(in crate::domain_computation::primary_graph) parameter_type: String,
    pub(in crate::domain_computation::primary_graph) result_type: String,
    pub(in crate::domain_computation::primary_graph) binding: &'static str,
    pub(in crate::domain_computation::primary_graph) expression_type: ExpressionType,
    pub(in crate::domain_computation::primary_graph) observed: Option<(
        Result<ExpressionValue, ExpressionDenial>,
        WorthQueryObservedSource<()>,
    )>,
}

impl<Schema: ApplicationSchema> Default for WorthQueryWorkflowConditionSources<Schema> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Schema: ApplicationSchema> WorthQueryWorkflowConditionSources<Schema> {
    pub const fn new() -> Self {
        Self {
            operands: Vec::new(),
            schema: PhantomData,
        }
    }

    /// Supplies operand `name` from `Binding`'s published result. Supplying a
    /// name twice leaves the set unable to settle.
    pub fn operand<Binding, Value>(
        mut self,
        name: &str,
        source: WorthQueryApplicationOutputDemandSource<Binding::Query, Value>,
    ) -> Self
    where
        Binding: ApplicationQueryBinding<Schema>,
        Binding::Query: ApplicationQueryMarkerIdentity<Schema>,
        <Binding::Query as ApplicationQueryMarkerIdentity<Schema>>::ResultBinding:
            ApplicationStructuredValueBinding<Value = Value>,
        Value: ApplicationExpressionOperandValue,
    {
        let observed = source
            .into_single_source()
            .map(|(value, source)| (value.expression_value(), source.retyped()));
        let operand = ConditionOperandSource {
            name: name.into(),
            query: Binding::Query::IDENTIFIER,
            parameter_type: Binding::Query::PARAMETER_TYPE_IDENTITY.as_str().to_owned(),
            result_type: Binding::Query::RESULT_TYPE_IDENTITY.as_str().to_owned(),
            binding: Binding::IDENTITY,
            expression_type: Value::expression_type(),
            observed,
        };
        let index = self
            .operands
            .partition_point(|supplied| supplied.name <= operand.name);
        self.operands.insert(index, operand);
        self
    }

    /// Whether these are exactly the operands `required` reads: the same
    /// names, each from the installed query and binding it declares.
    pub fn supplies(&self, required: &RequiredWorkflowCondition) -> bool {
        supplies(&self.operands, required.operands())
    }

    /// Operands in name order.
    pub(in crate::domain_computation::primary_graph) fn operands(
        &self,
    ) -> &[ConditionOperandSource] {
        &self.operands
    }

    pub(in crate::domain_computation::primary_graph) fn into_operands(
        self,
    ) -> Vec<ConditionOperandSource> {
        self.operands
    }
}

/// Supplied operands match required ones pairwise, in name order; a repeated
/// or missing name leaves the lengths or names unequal.
pub(in crate::domain_computation::primary_graph) fn supplies(
    supplied: &[ConditionOperandSource],
    required: &[WorkflowConditionOperand],
) -> bool {
    supplied.len() == required.len()
        && supplied.iter().zip(required).all(|(supplied, required)| {
            *supplied.name == *required.name()
                && supplied.query == required.query()
                && supplied.parameter_type == required.parameter_type()
                && supplied.result_type == required.result_type()
                && supplied.binding == required.binding()
        })
}
