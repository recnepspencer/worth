//! An admitted SubjectRead and its independent history governance share one genuine session.
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_query::ApplicationHistoryQueryBinding, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationHistoryQuery;

use super::{
    HistoryMetadataAdmission, SubjectPlan, SubjectScope, WorthQueryApplicationHistoryCaptureLimits,
    WorthQueryApplicationHistoryDenial as Denial,
};
use crate::domain_computation::{
    authorization::WorthQueryAdmittedApplicationCapabilityAccess,
    primary_graph::{
        application_query::{
            admission::prepare_governed_access, WorthQueryApplicationQueryAccessContext,
        },
        WorthQueryPrimaryGraphApplicationRuntime,
    },
};

/// A capture can only consume the SubjectRead plan whose session admitted both decisions.
pub struct WorthQueryAdmittedApplicationHistoryCapture<
    'a,
    Schema: ApplicationSchema,
    History: ApplicationHistoryQueryBinding<Schema>,
> {
    pub(in crate::domain_computation::primary_graph::application_query) history:
        &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
    pub(in crate::domain_computation::primary_graph::application_query) plan:
        SubjectPlan<'a, Schema, History>,
    pub(in crate::domain_computation::primary_graph::application_query) metadata:
        HistoryMetadataAdmission<History>,
    pub(in crate::domain_computation::primary_graph::application_query) limits:
        WorthQueryApplicationHistoryCaptureLimits,
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// The authored history contract must admit Public metadata; absence of a capability
    /// is not an implicit permission. SubjectRead retains its own admission and governance.
    pub fn admit_application_history_capture<'a, History>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        mut plan: SubjectPlan<'a, Schema, History>,
        limits: WorthQueryApplicationHistoryCaptureLimits,
    ) -> Result<WorthQueryAdmittedApplicationHistoryCapture<'a, Schema, History>, Denial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
    {
        let metadata =
            self.admit_application_history_metadata(execution, history, &mut plan, None)?;
        Ok(WorthQueryAdmittedApplicationHistoryCapture {
            history,
            plan,
            metadata,
            limits,
        })
    }

    /// The typed capability admits only the independent history contract on this session.
    pub fn admit_governed_application_history_capture<'a, History, Capability, Operation, Input>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        mut plan: SubjectPlan<'a, Schema, History>,
        capability: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
        limits: WorthQueryApplicationHistoryCaptureLimits,
    ) -> Result<WorthQueryAdmittedApplicationHistoryCapture<'a, Schema, History>, Denial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
        Input:
            ApplicationCapabilityRequest<Schema, Capability, Scope = SubjectScope<Schema, History>>,
    {
        let access = WorthQueryApplicationQueryAccessContext::new(plan.principal, plan.scope);
        let pending = prepare_governed_access(
            self,
            plan.query,
            &access,
            capability,
            plan.controls.request_scope(),
        )
        .map_err(Denial::Query)?;
        let metadata =
            self.admit_application_history_metadata(execution, history, &mut plan, Some(pending))?;
        Ok(WorthQueryAdmittedApplicationHistoryCapture {
            history,
            plan,
            metadata,
            limits,
        })
    }
}
