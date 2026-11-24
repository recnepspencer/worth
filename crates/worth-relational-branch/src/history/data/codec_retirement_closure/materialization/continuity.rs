//! Retained suspension custody rejoins matching canonical producer groups on restoration.
use super::capture::stop;
use super::*;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::branch::materialization::RelationalMaterializationCustody;
use crate::history::data::{
    canonical_commit_envelope::OutputOriginHistory, CanonicalRequiredOutputMaterialization as Event,
};
use crate::storage::overlay::WorkingState;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling};

pub fn prepare_materialization_continuity<'r>(
    execution: &ExecutionRequest<'_, '_>,
    runtime: impl Into<
        crate::history::data::codec_retirement_closure::history_request::RetirementHistoryInputs<
            'r,
        >,
    >,
    basis: &AdmittedRelationalBranchBasis,
    custody: &RelationalMaterializationCustody,
    candidate: &WorkingState,
) -> Result<Option<Arc<Event>>, TransactionCommitError> {
    let runtime = &crate::history::data::codec_retirement_closure::history_request::RuntimeHistoryRequest::new(runtime, execution);
    let captured = custody.data();
    if captured.producer_publications.is_empty() {
        return Ok(None);
    }
    let result = execution
        .in_scope(|lease| {
            ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
                lease,
                None,
                |context| {
                    let _groups_memory = super::super::reserve(
                        lease,
                        captured
                            .producer_records
                            .len()
                            .checked_mul(std::mem::size_of::<
                                Arc<crate::history::data::CanonicalRequiredOutputOrigin>,
                            >())
                            .ok_or_else(super::super::memory_overflow)?,
                    )?;
                    let mut groups: Vec<Arc<crate::history::data::CanonicalRequiredOutputOrigin>> =
                        Vec::with_capacity(captured.producer_records.len());
                    let mut history = OutputOriginHistory::default();
                    let mut publications = Vec::with_capacity(captured.producer_publications.len());
                    for publication in &captured.producer_publications {
                        charge(context, 1)?;
                        let original = history
                            .by_commit(
                                *publication,
                                &mut |id| runtime.commit(id),
                                &runtime.execution(),
                                context,
                            )?
                            .ok_or("materialization original producer history absent")?;
                        for origin in original.output_origins() {
                            charge(
                                context,
                                captured.producer_records.len().saturating_mul(
                                    origin.members.len() + origin.relations.len() + 1,
                                ),
                            )?;
                            if !captured
                                .producer_records
                                .iter()
                                .any(|record| header::owns(origin, &record.record))
                            {
                                continue;
                            }
                            if payload::candidate_matches(
                                runtime,
                                basis,
                                candidate,
                                &original,
                                &origin.meaning,
                                &captured.producer_records,
                                context,
                                lease,
                            )? {
                                if !publications.contains(publication) {
                                    publications.push(*publication);
                                }
                                groups.push(Arc::clone(origin));
                            }
                        }
                    }
                    if publications.is_empty() {
                        return Ok(None);
                    }
                    let mut records = Vec::with_capacity(captured.producer_records.len());
                    for record in &captured.producer_records {
                        charge(context, publications.len() + 1)?;
                        let mut owned = false;
                        for origin in &groups {
                            charge(context, origin.members.len() + origin.relations.len() + 1)?;
                            if header::owns(origin, &record.record) {
                                owned = true;
                                break;
                            }
                        }
                        if owned {
                            records.push(record.clone());
                        }
                    }
                    Ok::<_, Stop>(Some(Arc::new(Event {
                        source_commit: captured.source_commit,
                        suspension_commit: captured.suspension_commit,
                        producer_publications: publications,
                        records,
                    })))
                },
            )
        })
        .map_err(|cause| stop(Stop::Request(cause.into())))?
        .map_err(|cause| stop(Stop::Request(cause.into())))?
        .0
        .map_err(stop)?;
    execution
        .consult()
        .map_err(|cause| stop(Stop::Request(cause.into())))?;
    Ok(result)
}
