//! Rebind every recorded outbox to its co-created original operation before service.
use super::*;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectFieldLocator, ContractValidatedAspectValueView};
use worth_relational::facade::runtime::{EntityReadRecord, VisibilityProjectionView};

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn dispatch_readmission_catalog(
        &self,
    ) -> &worth_query_installation::facade::InstalledOperationCatalog {
        &self.installed_operations
    }
    pub(in crate::domain_computation::primary_graph) fn readmit_dispatch_outboxes(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        observation: &worth_runtime_world::facade::RuntimeWorldObservationPort,
        inspection: &worth_runtime_world::facade::RuntimeWorldInspectionPort,
    ) -> Result<bool, Denial> {
        let (inbound, candidates) = self.graph.with_runtime(execution, |runtime| {
            let (_, basis) = runtime
                .observe_branch(&runtime.main_branch_identity())
                .map_err(|_| Denial::ExactCommitUnavailable)?;
            let view = runtime
                .read_truth()
                .project_observation(&basis.observation())
                .map_err(|_| Denial::ExactCommitUnavailable)?;
            self.readmit_outbox_view(execution, &view)
        })?;
        self.readmit_dispatch_correlation_index(execution, observation, inspection, candidates)?;
        Ok(inbound)
    }

    fn readmit_outbox_view(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        view: &VisibilityProjectionView,
    ) -> Result<(bool, Vec<super::recovered::RecoveredDispatchCandidate>), Denial> {
        let layout = self.graph.layout.provider_dispatch_outbox();
        let outboxes = view
            .bounded_entities_of_kind(execution, layout.entity_kind, usize::MAX)
            .map_err(read_denial)?
            .map_err(|_| Denial::Malformed)?;
        if outboxes.records().is_empty() {
            return Ok((false, Vec::new()));
        }
        let binding_layout = self.graph.layout.provider_idempotency();
        let bindings = view
            .bounded_entities_of_kind(execution, binding_layout.entity_kind, usize::MAX)
            .map_err(read_denial)?
            .map_err(|_| Denial::Malformed)?;
        let bytes = bindings
            .records()
            .len()
            .checked_mul(std::mem::size_of::<(
                u64,
                &str,
                worth_relational::facade::identity::VersionId,
            )>())
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Denial::Malformed)?;
        let _funding = execution
            .reserve_retained_memory(bytes)
            .map_err(|cause| read_denial(cause.into()))?;
        let mut by_outcome = Vec::new();
        by_outcome
            .try_reserve_exact(bindings.records().len())
            .map_err(|_| Denial::Malformed)?;
        execution
            .run_native_with_checkpoint(|context| {
                let stop =
                    |cause| read_denial(worth_execution::WorkCeilingDenial::Stopped(cause).into());
                for row in bindings.records() {
                    context.checkpoint(1).map_err(stop)?;
                    let AspectValue::UInt64(outcome) =
                        field(row, &binding_layout.outcome_identity_locator)?
                    else {
                        return Err(Denial::Malformed);
                    };
                    let AspectValue::String(intent) = field(row, &binding_layout.intent_locator)?
                    else {
                        return Err(Denial::Malformed);
                    };
                    let intent = intent.raw_text().ok_or(Denial::Malformed)?;
                    by_outcome.push((*outcome, intent, row.created_at_version));
                }
                let ordering = by_outcome
                    .len()
                    .checked_mul(by_outcome.len().max(1).ilog2() as usize + 1)
                    .and_then(|n| u64::try_from(n).ok())
                    .ok_or(Denial::Malformed)?;
                context.checkpoint(ordering).map_err(stop)?;
                by_outcome.sort_unstable_by_key(|(outcome, _, _)| *outcome);
                if by_outcome.windows(2).any(|pair| pair[0].0 == pair[1].0) {
                    return Err(Denial::RecordMismatch);
                }
                let required = required_fields(layout)?;
                let mut inbound = false;
                let mut candidates = Vec::new();
                candidates
                    .try_reserve_exact(outboxes.records().len())
                    .map_err(|_| Denial::Malformed)?;
                for row in outboxes.records() {
                    context.checkpoint(1).map_err(stop)?;
                    let aspect = row
                        .authoritative_aspect_state
                        .as_ref()
                        .and_then(|state| {
                            state.get(layout.correlation_locator.aspect().aspect_key())
                        })
                        .ok_or(Denial::Malformed)?;
                    let ContractValidatedAspectValueView::Struct(values) = aspect.view() else {
                        return Err(Denial::Malformed);
                    };
                    let fields = decode_fields(
                        required
                            .iter()
                            .map(|key| values.get(key).cloned())
                            .collect::<Option<Vec<_>>>()
                            .ok_or(Denial::Malformed)?,
                    )?;
                    let index = by_outcome
                        .binary_search_by_key(&fields.outcome_identity, |(outcome, _, _)| *outcome)
                        .map_err(|_| Denial::RecordMismatch)?;
                    let (_, intent, version) = by_outcome[index];
                    if version != row.created_at_version {
                        return Err(Denial::CommitMismatch);
                    }
                    context
                        .checkpoint(
                            u64::try_from(
                                fields
                                    .payload
                                    .len()
                                    .checked_add(intent.len())
                                    .ok_or(Denial::Malformed)?,
                            )
                            .map_err(|_| Denial::Malformed)?,
                        )
                        .map_err(stop)?;
                    let rebound = super::super::WorthQueryRecoveredDispatchOutbox::from_source_row(
                        RecordRef::Entity(row.entity_id),
                        fields,
                        intent,
                        &self.installed_operations,
                    );
                    // Do not short-circuit: a later malformed row must still refuse service.
                    inbound |= rebound.inbound()?.is_some();
                    candidates.push(super::recovered::RecoveredDispatchCandidate {
                        version: row.created_at_version,
                        entity: row.entity_id,
                        expected: rebound.readmitted_record()?,
                    });
                }
                Ok((inbound, candidates))
            })
            .map_err(|cause| read_denial(cause.into()))?
            .0
    }
}

fn field<'a>(
    row: &'a EntityReadRecord,
    locator: &AspectFieldLocator,
) -> Result<&'a AspectValue, Denial> {
    let aspect = row
        .authoritative_aspect_state
        .as_ref()
        .and_then(|state| state.get(locator.aspect().aspect_key()))
        .ok_or(Denial::Malformed)?;
    let ContractValidatedAspectValueView::Struct(fields) = aspect.view() else {
        return Err(Denial::Malformed);
    };
    fields
        .get(
            locator
                .field_path()
                .fields()
                .first()
                .ok_or(Denial::Malformed)?,
        )
        .ok_or(Denial::Malformed)
}

fn read_denial(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
) -> Denial {
    Denial::ReadExecution(
        super::super::relational_execution_denial::relational_execution_kind(cause, None),
    )
}
