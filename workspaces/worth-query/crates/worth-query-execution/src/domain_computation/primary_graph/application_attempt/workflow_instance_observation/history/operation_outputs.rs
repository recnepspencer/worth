//! Reconstruct native correspondence closure without consulting today's business selection.
use super::{budget::HistoryReconstructionCharge, denial, WorthQueryApplicationAttemptDenial};
use crate::domain_computation::primary_graph::{
    application_attempt::{
        observe_adjacency, observe_field_value,
        WorthQueryApplicationAdjacencyDirection as Direction,
    },
    workflow::schema::WorthQueryWorkflowLayout,
};
use std::collections::BTreeSet;
use worth_foundational::facade::{AspectFieldLocator, AspectValue, InternedString};
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::RelationalRuntime,
    snapshots::SnapshotHandle,
};
#[cfg(test)]
mod tests;

pub(super) fn observe(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    transition: EntityId,
    charge: &mut HistoryReconstructionCharge,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    OutputContextReader {
        execution,
        runtime,
        snapshot,
        layout,
    }
    .validate(transition, charge)
}

/// One pinned native read context, not a cache or a second output authority.
struct OutputContextReader<'a, 'scope, 'lease> {
    execution: &'a worth_execution::ExecutionRequest<'scope, 'lease>,
    runtime: &'a RelationalRuntime,
    snapshot: &'a SnapshotHandle,
    layout: &'a WorthQueryWorkflowLayout,
}

impl OutputContextReader<'_, '_, '_> {
    fn validate(
        &self,
        transition: EntityId,
        charge: &mut HistoryReconstructionCharge,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let count = self.count(transition)?;
        if count > charge.record_limit() {
            return Err(super::budget::inventory_exhausted());
        }
        charge.reserve_inventory(count)?;
        let members = observe_adjacency(
            self.runtime,
            self.execution,
            self.snapshot,
            self.layout.operation_output.membership,
            transition,
            Direction::Outgoing,
            count
                .checked_add(1)
                .and_then(|count| count.checked_mul(2))
                .ok_or_else(|| denial("workflow output count overflows"))?,
        )?
        .ok_or_else(|| denial("workflow operation outputs are unavailable"))?;
        if members.len() != count {
            return Err(denial(
                "workflow operation output count disagrees with membership",
            ));
        }
        let mut roles = BTreeSet::new();
        for member in members {
            let role = self.entry(member.to, charge)?;
            if !roles.insert(role) {
                return Err(denial("workflow operation output role is duplicated"));
            }
        }
        Ok(())
    }

    fn count(&self, transition: EntityId) -> Result<usize, WorthQueryApplicationAttemptDenial> {
        let layout = &self.layout.transition;
        let read = |field| {
            observe_field_value(
                self.runtime,
                self.execution,
                self.snapshot,
                transition,
                layout.entity_kind,
                field,
            )
        };
        match (
            read(&layout.protocol_version)?,
            read(&layout.output_binding)?,
            read(&layout.output_count)?,
        ) {
            // Old missing context is never backfilled from today's selection.
            (Some(AspectValue::UInt64(2)), None, None) => Ok(0),
            (
                Some(AspectValue::UInt64(3)),
                Some(AspectValue::String(InternedString::Raw(binding))),
                Some(AspectValue::UInt64(count)),
            ) if !binding.is_empty() => {
                usize::try_from(count).map_err(|_| denial("workflow output count overflows"))
            }
            _ => Err(denial("workflow operation output context is malformed")),
        }
    }

    fn entry(
        &self,
        entry: EntityId,
        charge: &mut HistoryReconstructionCharge,
    ) -> Result<String, WorthQueryApplicationAttemptDenial> {
        let output = &self.layout.operation_output;
        charge.reserve_record(
            self.execution,
            self.runtime,
            self.snapshot,
            entry,
            output.entity_kind,
            &output.role,
        )?;
        let role = self.text(entry, &output.role)?;
        self.text(entry, &output.entity)?;
        let posture = observe_field_value(
            self.runtime,
            self.execution,
            self.snapshot,
            entry,
            output.entity_kind,
            &output.posture,
        )?;
        if !matches!(posture, Some(AspectValue::UInt64(1..=3))) {
            return Err(denial("workflow operation output posture is unsupported"));
        }
        let targets = crate::domain_computation::primary_graph::application_attempt::fact::observe_audit_outgoing_adjacency(
            self.runtime, self.execution, self.snapshot, output.target, entry, 4,
        )?.ok_or_else(|| denial("workflow output target is unavailable"))?;
        if targets.len() != 1 {
            return Err(denial(
                "workflow output requires exactly one historical target",
            ));
        }
        // Retire or later deletion does not reclassify historical correspondence.
        Ok(role)
    }

    fn text(
        &self,
        entity: EntityId,
        field: &AspectFieldLocator,
    ) -> Result<String, WorthQueryApplicationAttemptDenial> {
        text(
            self.execution,
            self.runtime,
            self.snapshot,
            entity,
            self.layout.operation_output.entity_kind,
            field,
        )
    }
}

fn text(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    entity: EntityId,
    kind: KindId,
    field: &AspectFieldLocator,
) -> Result<String, WorthQueryApplicationAttemptDenial> {
    match observe_field_value(runtime, execution, snapshot, entity, kind, field)? {
        Some(AspectValue::String(InternedString::Raw(value))) if !value.is_empty() => Ok(value),
        _ => Err(denial("workflow output text is malformed")),
    }
}
