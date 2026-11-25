//! Metered target-entity Foundation `where_filter` matcher for related predicates.
use super::super::residual::{
    charge_work, evaluation, log2_ceil, operands_input_bytes, prepare_static_operands,
    row_operands::RowOperandReader, PreparedOperands,
};
use super::super::{NarrowingRead, RootPathSourceBuilder, RootSelectionWork};
use super::target_check::projection_unavailable;
use crate::domain_computation::primary_graph::application_query::read_execution::{
    relational_read_denial, WorthQueryApplicationReadExecutionDenial,
};
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use std::collections::BTreeMap;
use worth_foundational::expression_api::{ExpressionInputs, ExpressionValue};
use worth_query_installation::facade::WorthQueryInstalledWhereFilter;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

pub(super) struct TargetFilterMatcher<'plan> {
    filter: &'plan WorthQueryInstalledWhereFilter,
    static_operands: PreparedOperands,
    rows: RowOperandReader<'plan>,
    static_input_bytes: u64,
    total_operands: usize,
    input_build_work: u64,
}

impl<'plan> TargetFilterMatcher<'plan> {
    pub(super) fn new<ReadIdentity>(
        read: &NarrowingRead<'_, '_, '_, '_, ReadIdentity>,
        target_entity: &'plan str,
        filter: &'plan WorthQueryInstalledWhereFilter,
        subject: &str,
        allocation: &SourceFactsAllocation,
        work: &mut RootSelectionWork<'_>,
    ) -> Result<Self, WorthQueryApplicationReadExecutionDenial> {
        let stop = |cause| relational_read_denial(cause, subject);
        let static_operands = prepare_static_operands(read, filter, allocation, work)?;
        let rows = RowOperandReader::new_for_entity(read, target_entity, filter, allocation, work)?;
        let builder_schema_bytes =
            ExpressionInputs::scalar_builder_schema_and_conform_bytes(filter.program().schema())
                .ok_or_else(|| projection_unavailable(subject))
                .and_then(|bytes| {
                    u64::try_from(bytes).map_err(|_| stop(Cause::ChargedBytesOverflow))
                })?;
        let static_input_bytes = operands_input_bytes(&static_operands.0)
            .and_then(|n| n.checked_add(operands_input_bytes(&static_operands.1)?))
            .and_then(|n| n.checked_add(builder_schema_bytes))
            .ok_or_else(|| stop(Cause::ChargedBytesOverflow))?;
        let total_operands = static_operands
            .0
            .len()
            .checked_add(static_operands.1.len())
            .and_then(|n| n.checked_add(filter.fields().len()))
            .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
        let schema_len = filter.program().schema().operands().len();
        let per_bind_work = log2_ceil(total_operands)
            .checked_add(log2_ceil(schema_len))
            .and_then(|n| n.checked_add(2))
            .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
        let input_build_work = (total_operands as u64)
            .checked_mul(per_bind_work)
            .and_then(|n| n.checked_add(schema_len as u64))
            .and_then(|n| n.checked_add(builder_schema_bytes))
            .ok_or_else(|| stop(Cause::WorkCounterOverflow))?;
        Ok(Self {
            filter,
            static_operands,
            rows,
            static_input_bytes,
            total_operands,
            input_build_work,
        })
    }

    pub(super) fn matches<ReadIdentity>(
        &self,
        read: &NarrowingRead<'_, '_, '_, '_, ReadIdentity>,
        target_id: EntityId,
        subject: &str,
        evidence: &mut RootPathSourceBuilder,
        work: &mut RootSelectionWork<'_>,
    ) -> Result<bool, WorthQueryApplicationReadExecutionDenial> {
        let stop = |cause| relational_read_denial(cause, subject);
        let row_alloc = SourceFactsAllocation::open(read.execution).map_err(stop)?;
        let fields = self
            .rows
            .read(read, target_id, evidence, &row_alloc, work)?;
        let field_input_bytes =
            operands_input_bytes(&fields).ok_or_else(|| stop(Cause::ChargedBytesOverflow))?;
        let row_input_bytes = self
            .static_input_bytes
            .checked_add(field_input_bytes)
            .ok_or_else(|| stop(Cause::ChargedBytesOverflow))?;
        charge_work(read.execution, work, self.input_build_work)?;
        row_alloc
            .claim_tree_copy::<(Box<str>, ExpressionValue)>(read.execution, self.total_operands)
            .map_err(stop)?;
        row_alloc
            .claim_arc::<BTreeMap<Box<str>, ExpressionValue>>(read.execution)
            .map_err(stop)?;
        if row_input_bytes > 0 {
            row_alloc
                .claim(read.execution, row_input_bytes)
                .map_err(stop)?;
        }
        let mut inputs = ExpressionInputs::builder(self.filter.program().schema());
        for (name, value) in self
            .static_operands
            .0
            .iter()
            .chain(&self.static_operands.1)
            .chain(&fields)
        {
            inputs = inputs
                .bind(name, value.clone())
                .map_err(|_| projection_unavailable(subject))?;
        }
        evaluation::admits_row(read.execution, self.filter, &inputs.build(), work)
    }
}
