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
    let result = execution.in_scope(|lease| {
        ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
            lease,
            None,
            |context| {
                if basis.descriptor().runtime_instance_id() != runtime.runtime_instance_id()
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
                )
            },
        )
    });
    execution.consult().map_err(request)?;
    result.map_err(request)?.map_err(request)?.0
}

pub(super) fn select_at(
    runtime: &RelationalRuntime,
    mut next: Option<CommitId>,
    meaning: &RequiredOutputPublicationMeaning,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
) -> Result<Option<RequiredOutputRoutePosition>, Stop> {
    let bound = runtime.history.catalog_len();
    for _ in 0..bound {
        let Some(commit) = next else { return Ok(None) };
        charge(context, bound.saturating_add(1))?;
        let envelope = runtime
            .history
            .canonical_envelope(commit)
            .ok_or("cold route selection lost canonical history")?;
        require_linear_history(&envelope)?;
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
            super::codec_disposition::validate(runtime, &envelope, context, lease)?;
            return Ok(Some(RequiredOutputRoutePosition::CodecRetired(commit)));
        }
        if let Some(origin) = &envelope.required_output_origin {
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
                envelope.validate_output_membership_header()?;
                return Ok(Some(RequiredOutputRoutePosition::Published(commit)));
            }
        }
        next = envelope.commit.parents.first().copied();
    }
    if next.is_none() {
        Ok(None)
    } else {
        Err("cold route ancestry is incomplete or cyclic".into())
    }
}
