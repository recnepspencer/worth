//! Native producer association survives only authenticated semantic maintenance.
use super::{charge, CodecRetirementClosureStop as Stop};
use crate::history::data::{CanonicalCommitEnvelope, CommitId};
use crate::identity::data::VersionId;
use crate::transactions::data::RecordRef;
use std::sync::Arc;
use worth_execution::MapKernelContext;

mod capture;
mod header;
mod payload;
pub(crate) use capture::{capture_materialization_producers, prepare_materialization_continuity};
pub(super) use header::validate as validate_header;

/// A native write fence may advance only through matching physical maintenance.
#[allow(clippy::too_many_arguments)]
pub(crate) fn write_matches(
    record: &RecordRef,
    publication: CommitId,
    expected: VersionId,
    mut actual: VersionId,
    lookup_commit: &mut impl FnMut(CommitId) -> Result<Option<Arc<CanonicalCommitEnvelope>>, Stop>,
    lookup_version: &mut impl FnMut(
        VersionId,
        &mut MapKernelContext<'_, '_>,
    ) -> Result<Option<Arc<CanonicalCommitEnvelope>>, Stop>,
    maximum_hops: usize,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<bool, Stop> {
    for _ in 0..=maximum_hops {
        if actual == expected {
            return Ok(true);
        }
        if actual < expected {
            return Ok(false);
        }
        charge(context, 1)?;
        let envelope =
            lookup_version(actual, context)?.ok_or("maintenance write position absent")?;
        let Some(event) = envelope.required_output_materialization.as_ref() else {
            return Ok(false);
        };
        validate_header(&envelope, lookup_commit, maximum_hops, execution, context)?;
        if !event.producer_publications.contains(&publication) {
            return Ok(false);
        }
        let Some(member) = event.records.iter().find(|member| &member.record == record) else {
            return Ok(false);
        };
        if member.before_write >= actual {
            return Err("maintenance write does not advance its captured predecessor".into());
        }
        actual = member.before_write;
    }
    Err("maintenance write ancestry is incomplete or cyclic".into())
}

pub(super) fn validate_semantics(
    runtime: &crate::history::data::codec_retirement_closure::history_request::RuntimeHistoryRequest<'_, '_, '_>,
    envelope: &CanonicalCommitEnvelope,
    context: &mut MapKernelContext<'_, '_>,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    schema: &crate::branch::RelationalBranchRootSchemaAuthority,
) -> Result<(), super::PublicationHistoryFailure> {
    validate_header(
        envelope,
        &mut |id| runtime.commit(id),
        runtime.runtime.history.catalog_len(),
        &runtime.execution(),
        context,
    )?;
    payload::validate(runtime, envelope, context, lease, schema)
}
