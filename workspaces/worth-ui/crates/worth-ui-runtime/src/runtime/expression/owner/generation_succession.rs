use std::collections::BTreeSet;
use std::sync::Arc;

use super::operand_binding::UiExpressionInputs;
use super::records::UiExpressionRecords;
use super::state::probe_projections;
use super::{
    UiExpressionEvaluationRecord, UiExpressionOperandFact, UiExpressionRuntimeState,
    UiExpressionWorkCounters,
};
use crate::runtime::expression::{
    UiExpressionCatalog, UiInstalledExpression, UiResolvedExpressionOperand,
};

impl UiExpressionRuntimeState {
    /// Moves this owner to the generation `inputs` names as active, against
    /// the catalog that generation installs. It is the only place the
    /// owner's generation changes.
    ///
    /// An expression whose successor installs the same program over the same
    /// operands in the same slot, and whose every operand fact is still the
    /// one its owner holds, is re-stamped: its record keeps its outcome,
    /// outcome revision and last current value, takes the new generation and
    /// the successor's span, and is not evaluated again. Any other expression
    /// is rebuilt with no prior record, and every expression is rebuilt when
    /// the successor catalog installs different slots. A rebuilt expression
    /// settles its readers through the dependency index, as any change does.
    /// Whatever the path, no reference or ticket of the prior generation is
    /// current afterwards.
    ///
    /// The work counters carry across: each operand re-proven counts one
    /// operand probe, each indexed projection slot one probe, and a rebuild
    /// counts as any settle does.
    pub(crate) fn follow(
        &mut self,
        catalog: &Arc<UiExpressionCatalog>,
        inputs: &UiExpressionInputs<'_>,
    ) {
        if self.follows(inputs.generation) && Arc::ptr_eq(&self.catalog, catalog) {
            return;
        }
        let same_slots = self.catalog.has_same_slots_as(catalog);
        let mut retained = std::mem::replace(
            &mut self.records,
            UiExpressionRecords::unsettled(catalog.slot_count()),
        );
        let mut dirty = BTreeSet::new();
        for slot in catalog.slot_count().slots() {
            let restamped = match (
                retained.take(slot),
                self.catalog.expression(slot),
                catalog.expression(slot),
            ) {
                (Some(record), Some(prior), Some(successor))
                    if same_slots && prior.installs_same_program_as(successor) =>
                {
                    restamp(record, successor, &self.records, inputs, &mut self.counters)
                }
                _ => None,
            };
            if !restamped.is_some_and(|record| self.records.admit(record)) {
                dirty.insert(slot);
            }
        }
        self.catalog = Arc::clone(catalog);
        self.generation = inputs.generation.clone();
        self.projections = probe_projections(catalog, inputs.mounted, &mut self.counters);
        self.settle_dirty(dirty, inputs);
    }
}

/// `record` as a record of the successor generation, when every operand fact
/// it read is still what its owner holds. `records` holds the successor
/// records of every upstream slot already followed.
fn restamp(
    mut record: UiExpressionEvaluationRecord,
    successor: &UiInstalledExpression,
    records: &UiExpressionRecords,
    inputs: &UiExpressionInputs<'_>,
    counters: &mut UiExpressionWorkCounters,
) -> Option<UiExpressionEvaluationRecord> {
    if successor.operands().len() != record.operands.len() {
        return None;
    }
    record.operands = successor
        .operands()
        .iter()
        .zip(&record.operands)
        .map(|(operand, read)| {
            counters.operand_probes = counters.operand_probes.saturating_add(1);
            reprove(operand.source(), read, records, inputs)
        })
        .collect::<Option<_>>()?;
    record.generation = inputs.generation.clone();
    record.span = successor.body_span().cloned();
    Some(record)
}

/// The fact `read` is in the successor generation, or `None` when its owner
/// no longer holds it. An application fact is issued per generation, so an
/// equal identity, revision and value is re-issued for the successor. An
/// upstream expression that is rebuilt instead settles this reader through
/// the dependency index when its outcome changes.
fn reprove(
    source: &UiResolvedExpressionOperand,
    read: &UiExpressionOperandFact,
    records: &UiExpressionRecords,
    inputs: &UiExpressionInputs<'_>,
) -> Option<UiExpressionOperandFact> {
    let current = match source {
        UiResolvedExpressionOperand::QueryScalar { slot, .. } => {
            let observed = inputs.mounted.current_projection_input(*slot);
            match read {
                UiExpressionOperandFact::Query(reference) => observed.as_ref() == Some(reference),
                UiExpressionOperandFact::Absent { .. } => observed.is_none(),
                UiExpressionOperandFact::Application(_)
                | UiExpressionOperandFact::Expression { .. } => false,
            }
        }
        UiResolvedExpressionOperand::Application { slot, .. } => {
            let held = inputs.facts.input_reference(*slot, inputs.generation);
            match read {
                UiExpressionOperandFact::Application(reference) => {
                    let restamped = reference.restamped(inputs.generation);
                    return (held.as_ref() == Some(&restamped))
                        .then(|| UiExpressionOperandFact::Application(restamped));
                }
                UiExpressionOperandFact::Absent { .. } => held.is_none(),
                UiExpressionOperandFact::Query(_) | UiExpressionOperandFact::Expression { .. } => {
                    false
                }
            }
        }
        UiResolvedExpressionOperand::Expression { slot, .. } => match read {
            UiExpressionOperandFact::Expression {
                slot: read_slot,
                outcome_revision,
            } => {
                read_slot == slot
                    && records
                        .get(*slot)
                        .is_none_or(|upstream| upstream.outcome_revision == *outcome_revision)
            }
            UiExpressionOperandFact::Absent { .. } => records.get(*slot).is_none(),
            UiExpressionOperandFact::Query(_) | UiExpressionOperandFact::Application(_) => false,
        },
    };
    current.then(|| read.clone())
}
