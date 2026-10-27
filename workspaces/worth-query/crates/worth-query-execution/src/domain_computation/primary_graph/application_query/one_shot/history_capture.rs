//! Captured history subjects leave only through the ordinary successful read completion.
use worth_query_declaration::facade::{
    application_query::ApplicationHistoryQueryBinding, application_schema::ApplicationSchema,
};

use super::{
    admit_request, denial, map_authorized_read_denial, refresh_governed_authorization,
    validate_one_shot_plan, WorthQueryApplicationOneShotDenialKind,
    WorthQueryApplicationOneShotResult,
};
use crate::domain_computation::primary_graph::{
    application_query::{
        authorized_read::{
            execute_authorized_read_with_access, WorthQueryAuthorizedApplicationReadDenial,
        },
        history::{
            prepare_capture, SubjectQuery, SubjectResult,
            WorthQueryAdmittedApplicationHistoryCapture,
            WorthQueryApplicationHistoryDenial as Denial,
            WorthQueryCapturedApplicationHistorySubjects,
        },
        read_execution::{project_non_live_kernel_with_decode, read_bounded_root_rows},
        WorthQueryApplicationProjection,
    },
    value_decode::ValueDecodeState,
    WorthQueryAdvancementPhase, WorthQueryPrimaryGraphApplicationRuntime,
};

/// The ordinary disclosed result and the exact generations captured from that result.
pub struct WorthQueryApplicationHistoryCaptureResult<
    Schema: ApplicationSchema,
    History: ApplicationHistoryQueryBinding<Schema>,
> {
    read: WorthQueryApplicationOneShotResult<
        SubjectQuery<Schema, History>,
        SubjectResult<Schema, History>,
    >,
    subjects: WorthQueryCapturedApplicationHistorySubjects<Schema, History>,
}

impl<Schema: ApplicationSchema, History: ApplicationHistoryQueryBinding<Schema>>
    WorthQueryApplicationHistoryCaptureResult<Schema, History>
{
    pub fn into_parts(
        self,
    ) -> (
        WorthQueryApplicationOneShotResult<
            SubjectQuery<Schema, History>,
            SubjectResult<Schema, History>,
        >,
        WorthQueryCapturedApplicationHistorySubjects<Schema, History>,
    ) {
        (self.read, self.subjects)
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn execute_application_history_capture<History>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        capture: WorthQueryAdmittedApplicationHistoryCapture<'_, Schema, History>,
    ) -> Result<WorthQueryApplicationHistoryCaptureResult<Schema, History>, Denial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
        SubjectResult<Schema, History>:
            WorthQueryApplicationProjection<Schema, SubjectQuery<Schema, History>>,
    {
        let request = phase.execution_request_for(&self.product_runtime).map_err(|_| {
            Denial::Read(super::authorization_denial(
                crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial::new(
                    crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::Refused(
                        crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind::ForeignPhase,
                    ),
                    "phase-custody",
                ),
                capture.plan.query.name(),
            ))
        })?;
        self.execute_application_history_capture_with_request(&request, capture)
    }

    pub(in crate::domain_computation) fn execute_application_history_capture_with_request<History>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        capture: WorthQueryAdmittedApplicationHistoryCapture<'_, Schema, History>,
    ) -> Result<WorthQueryApplicationHistoryCaptureResult<Schema, History>, Denial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
        SubjectResult<Schema, History>:
            WorthQueryApplicationProjection<Schema, SubjectQuery<Schema, History>>,
    {
        let WorthQueryAdmittedApplicationHistoryCapture {
            history,
            mut plan,
            metadata,
            limits,
        } = capture;
        validate_one_shot_plan(execution, self, &plan)?;
        refresh_governed_authorization(execution, self, &mut plan)
            .map_err(|cause| Denial::Read(map_authorized_read_denial(cause, plan.query.name())))?;
        let (raw, authorization_work, read_proof) = execute_authorized_read_with_access(
            execution,
            self,
            &plan,
            |runtime, layout, security, plan| {
                metadata.validate(execution, self, runtime, security, plan)?;
                read_bounded_root_rows(runtime, execution, layout, plan, None)
                    .map_err(WorthQueryAuthorizedApplicationReadDenial::Read)
                    .map_err(Denial::from)
            },
        )?;
        let decode = ValueDecodeState::new(self.installed_schema().value_bindings().clone());
        let completed = super::outcome::complete_one_shot_with_capture(
            execution,
            self,
            &decode,
            plan,
            raw,
            // A successful completion necessarily ran the capture callback's
            // second genuine security resolution; failed captures issue no receipt.
            authorization_work.with_execution_security_product_resolution(),
            read_proof,
            |kernel, governance, request, query| {
                project_non_live_kernel_with_decode::<
                    Schema,
                    SubjectQuery<Schema, History>,
                    SubjectResult<Schema, History>,
                    _,
                >(
                    execution,
                    &decode,
                    kernel,
                    governance,
                    || admit_request(request, query),
                    |projection| {
                        denial(
                            WorthQueryApplicationOneShotDenialKind::Projection(projection.kind()),
                            query,
                            projection.subject(),
                        )
                    },
                )
            },
            |disclosed, plan| {
                // The same live graph session issues this callback's actual security root.
                // Completion still consumes the genuine proof from the ordinary bounded read.
                execute_authorized_read_with_access(
                    execution,
                    self,
                    plan,
                    |runtime, layout, security, plan| {
                        prepare_capture(
                            *execution, self, runtime, layout, security, history, plan, &metadata,
                            disclosed, limits,
                        )
                    },
                )
                .map(|(prepared, _, _)| prepared)
            },
        )?;
        let subjects = completed
            .capture
            .bind(*execution, completed.read.result_set_observation())?;
        Ok(WorthQueryApplicationHistoryCaptureResult {
            read: completed.read,
            subjects,
        })
    }
}
