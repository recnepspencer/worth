use std::num::NonZeroUsize;
use worth_query_declaration::facade::{
    application_query::ApplicationHistoryQueryBinding, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationHistoryQuery;
use worth_relational::facade::history::{
    RelationalRecordHistorySelectionLimits, RelationalRecordHistorySubject,
};

use super::super::super::projection::WorthQueryApplicationDisclosedProjectionTree;
use super::super::{
    admission::HistoryMetadataAdmission,
    bounds::WorthQueryApplicationHistoryCaptureLimits as Limits,
    denial::WorthQueryApplicationHistoryDenial as Denial, request_stage, SubjectPlan,
};
use super::{walk::CaptureWalk, PreparedHistoryCapture};
use crate::domain_computation::primary_graph::{
    application_query::{
        authorized_read::WorthQueryAuthorizedApplicationReadDenial,
        read_execution::relational_read_denial,
    },
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrimaryGraphLayout,
};

/// Called only from the genuine authorized-read callback at its admitted security snapshot.
#[allow(clippy::too_many_arguments)]
pub(in crate::domain_computation::primary_graph::application_query) fn prepare<Schema, History>(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    layout: &WorthQueryPrimaryGraphLayout,
    security: &worth_relational::facade::snapshots::SnapshotHandle,
    history: &WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
    plan: &SubjectPlan<'_, Schema, History>,
    metadata: &HistoryMetadataAdmission<History>,
    disclosed: &WorthQueryApplicationDisclosedProjectionTree,
    limits: Limits,
) -> Result<PreparedHistoryCapture<Schema, History>, Denial>
where
    Schema: ApplicationSchema,
    History: ApplicationHistoryQueryBinding<Schema>,
{
    metadata.validate(&execution, application, runtime, security, plan)?;
    if history.binding_identity() != plan.query.binding_identity()
        || history.subject_read().query().identity() != plan.query.identity()
    {
        return Err(Denial::SubjectMismatch);
    }
    if history.selected_aspects().len() > limits.maximum_aspects.get() {
        return Err(Denial::CaptureLimitExceeded);
    }
    request_stage::run(execution, |context| {
        request_stage::checkpoint(context, 1)?;
        let capacity = limits.maximum_subjects.get();
        let temporary_bytes = capacity
            .checked_mul(size_of::<RelationalRecordHistorySubject>())
            .ok_or(Denial::SizeOverflow)?;
        let subject_bytes = capacity
            .checked_mul(RelationalRecordHistorySubject::owned_backing_bytes())
            .ok_or(Denial::SizeOverflow)?;
        let owner_bytes = PreparedHistoryCapture::<Schema, History>::owner_bytes();
        let reserved_peak = temporary_bytes
            .checked_add(subject_bytes)
            .and_then(|n| n.checked_add(owner_bytes))
            .ok_or(Denial::SizeOverflow)?;
        if reserved_peak >= limits.maximum_owned_bytes.get() {
            return Err(Denial::StorageLimitExceeded);
        }
        let memory = execution
            .reserve_retained_memory(owner_bytes as u64)
            .map_err(Denial::Request)?;
        let mut prepared = PreparedHistoryCapture::identity(plan, memory);
        let projection = runtime
            .read_truth()
            .project_snapshot_in_request(plan.basis.snapshot_handle(), &execution)
            .map_err(|cause| {
                Denial::from(WorthQueryAuthorizedApplicationReadDenial::Read(
                    relational_read_denial(cause, History::IDENTITY),
                ))
            })?
            .ok_or(Denial::SubjectMismatch)?;
        let mut walk = CaptureWalk {
            execution,
            runtime,
            projection: &projection,
            layout,
            contract: plan.query.read_graph(),
            target: history.target(),
            basis: plan.basis.selected_product().relational_basis(),
            subjects: worth_execution::ExecutionRequestArrayBuilder::allocate(execution, capacity)?,
            maximum_visits: limits.maximum_source_visits.get(),
            visits: 0,
        };
        walk.nodes(
            context,
            disclosed.raw_rows(),
            plan.query.read_graph().root_entity(),
        )?;
        if !walk.subjects.is_empty() {
            let remaining = limits
                .maximum_owned_bytes
                .get()
                .checked_sub(reserved_peak)
                .and_then(NonZeroUsize::new)
                .ok_or(Denial::StorageLimitExceeded)?;
            prepared.native = Some(
                runtime
                    .retain_record_history_selection(
                        execution,
                        walk.subjects.elements(),
                        history.selected_aspects(),
                        RelationalRecordHistorySelectionLimits {
                            maximum_subjects: limits.maximum_subjects,
                            maximum_aspects: limits.maximum_aspects,
                            maximum_owned_bytes: remaining,
                        },
                    )
                    .map_err(Denial::Native)?,
            );
        }
        Ok(prepared)
    })
}
