//! Latest complete native publication or governed codec disposition on a route.
use super::{charge, request, require_linear_history, CodecRetirementClosureStop as Stop};
use crate::{
    branch::AdmittedRelationalBranchBasis,
    history::data::{CommitId, RequiredOutputRoutePosition},
    mvcc::RequiredOutputPublicationMeaning,
    runtime::RelationalRuntime,
};
use worth_execution::{
    ExecutionRequest, ExecutionResourceLease, ExecutionWorkCeiling, MapKernelContext,
};

pub(crate) fn readmit_required_output_route(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    meaning: &RequiredOutputPublicationMeaning,
) -> Result<Option<RequiredOutputRoutePosition>, Stop> {
    let runtime = &crate::history::data::codec_retirement_closure::history_request::RuntimeHistoryRequest::new(runtime, execution);
    let result = execution.in_scope(|lease| {
        ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
            lease,
            None,
            |context| {
                if basis.descriptor().runtime_instance_id() != runtime.runtime.runtime_instance_id()
                    || !basis.is_current()
                {
                    return Err(
                        "cold route selection requires the exact current native basis".into(),
                    );
                }
                select_at(
                    runtime,
                    basis.inner.root.commit_id(),
                    meaning,
                    context,
                    lease,
                    basis.inner.root.schema_authority(),
                )
            },
        )
    });
    execution.consult().map_err(request)?;
    result.map_err(request)?.map_err(request)?.0
}

pub(super) fn select_at(
    runtime: &crate::history::data::codec_retirement_closure::history_request::RuntimeHistoryRequest<'_, '_, '_>,
    mut next: Option<CommitId>,
    meaning: &RequiredOutputPublicationMeaning,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
    schema: &crate::branch::RelationalBranchRootSchemaAuthority,
) -> Result<Option<RequiredOutputRoutePosition>, Stop> {
    let bound = runtime.runtime.history.catalog_len();
    for _ in 0..bound {
        let Some(commit) = next else { return Ok(None) };
        charge(context, bound.saturating_add(1))?;
        let envelope = runtime
            .commit(commit)?
            .ok_or("cold route selection lost canonical history")?;
        require_linear_history(&envelope)?;
        for absence in envelope.historical_absent_routes() {
            charge(
                context,
                absence.meaning.family.len() + meaning.family.len() + 65,
            )?;
            if absence.meaning.same_route(meaning) {
                envelope.validate_historical_selection_header(context)?;
                return Ok(Some(RequiredOutputRoutePosition::HistoricalAbsence(commit)));
            }
        }
        let mut event = None;
        for candidate in &envelope.required_output_codec_retirements {
            charge(
                context,
                candidate
                    .meaning
                    .family
                    .len()
                    .saturating_add(meaning.family.len())
                    .saturating_add(64),
            )?;
            if candidate.meaning.same_route(meaning) {
                if event.is_some() {
                    return Err("duplicate complete codec disposition for one route".into());
                }
                event = Some(candidate);
            }
        }
        if event.is_some() {
            super::codec_disposition::validate(runtime, &envelope, context, lease, schema)?;
            return Ok(Some(RequiredOutputRoutePosition::CodecRetired(commit)));
        }
        if let Some(restored) = &envelope.authored_cascade_restoration {
            charge(context, bound.saturating_add(1))?;
            let retired = runtime
                .commit(restored.retirement)?
                .ok_or("restoration retirement absent")?;
            for origin in retired.output_origins() {
                charge(
                    context,
                    origin.meaning.family.len() + meaning.family.len() + 64,
                )?;
                if origin.meaning.same_route(meaning) {
                    retired
                        .validate_output_origins(context)
                        .and_then(|origins| origins.output_origin_for_route(meaning, context))?;
                    envelope.validate_restoration_header(context)?;
                    return Ok(Some(RequiredOutputRoutePosition::Published(commit)));
                }
            }
        }
        for origin in envelope.output_origins() {
            charge(
                context,
                origin
                    .meaning
                    .family
                    .len()
                    .saturating_add(meaning.family.len())
                    .saturating_add(64),
            )?;
            if origin.meaning.same_route(meaning) {
                envelope
                    .validate_output_origins(context)
                    .and_then(|origins| origins.output_origin_for_route(meaning, context))?;
                return Ok(Some(RequiredOutputRoutePosition::Published(commit)));
            }
        }
        if let Some(event) = &envelope.required_output_materialization {
            super::materialization::validate_semantics(runtime, &envelope, context, lease, schema)?;
            next = Some(event.source_commit);
        } else {
            next = envelope.commit.parents.first().copied();
        }
    }
    if next.is_none() {
        Ok(None)
    } else {
        Err("cold route ancestry is incomplete or cyclic".into())
    }
}
