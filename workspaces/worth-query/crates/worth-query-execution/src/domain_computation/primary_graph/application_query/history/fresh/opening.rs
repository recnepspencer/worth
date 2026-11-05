//! Opening uses the original disclosed subject set and a separately selected history root.
use super::super::super::admission::prepare_governed_access;
use super::super::{
    session, SubjectScope, WorthQueryApplicationHistoryCursor as Cursor,
    WorthQueryApplicationHistoryPageLimits as Limits,
};
use super::*;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use {
    crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess as CapabilityAccess,
    crate::domain_computation::request_execution::WorthQueryAdvancementPhase,
};

pub trait OpeningOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
{
    #[allow(clippy::too_many_arguments)]
    fn open_application_history<'a, History>(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        subjects: &Subjects<Schema, History>,
        security: WorthQuerySelectedProductOperation<'a, Schema>,
        truth: &WorthQuerySelectedProductOperation<'a, Schema>,
        request: &'a WorthQueryRequestScope,
        limits: Limits,
    ) -> Result<Cursor<Schema, History>, Denial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
    {
        self.open_history_with(
            phase,
            history,
            external,
            subjects,
            security,
            truth,
            request,
            limits,
            |_| Ok((None, None)),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn open_governed_application_history<
        'a,
        History,
        ReadCapability,
        ReadOperation,
        ReadInput,
        MetadataCapability,
        MetadataOperation,
        MetadataInput,
    >(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        subjects: &Subjects<Schema, History>,
        security: WorthQuerySelectedProductOperation<'a, Schema>,
        truth: &WorthQuerySelectedProductOperation<'a, Schema>,
        read: CapabilityAccess<Schema, ReadCapability, ReadOperation, ReadInput>,
        metadata: CapabilityAccess<Schema, MetadataCapability, MetadataOperation, MetadataInput>,
        request: &'a WorthQueryRequestScope,
        limits: Limits,
    ) -> Result<Cursor<Schema, History>, Denial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
        ReadInput: ApplicationCapabilityRequest<
            Schema,
            ReadCapability,
            Scope = SubjectScope<Schema, History>,
        >,
        MetadataInput: ApplicationCapabilityRequest<
            Schema,
            MetadataCapability,
            Scope = SubjectScope<Schema, History>,
        >,
    {
        self.open_history_with(
            phase,
            history,
            external,
            subjects,
            security,
            truth,
            request,
            limits,
            |access| {
                Ok((
                    Some(
                        prepare_governed_access(
                            self.application_query_host(),
                            history.subject_read().query(),
                            access,
                            read,
                            request,
                        )
                        .map_err(Denial::Query)?,
                    ),
                    Some(
                        prepare_governed_access(
                            self.application_query_host(),
                            history.subject_read().query(),
                            access,
                            metadata,
                            request,
                        )
                        .map_err(Denial::Query)?,
                    ),
                ))
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn open_application_history_with_governed_subject<'a, History, Capability, Operation, Input>(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        subjects: &Subjects<Schema, History>,
        security: WorthQuerySelectedProductOperation<'a, Schema>,
        truth: &WorthQuerySelectedProductOperation<'a, Schema>,
        read: CapabilityAccess<Schema, Capability, Operation, Input>,
        request: &'a WorthQueryRequestScope,
        limits: Limits,
    ) -> Result<Cursor<Schema, History>, Denial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
        Input:
            ApplicationCapabilityRequest<Schema, Capability, Scope = SubjectScope<Schema, History>>,
    {
        self.open_history_with(
            phase,
            history,
            external,
            subjects,
            security,
            truth,
            request,
            limits,
            |access| {
                Ok((
                    Some(
                        prepare_governed_access(
                            self.application_query_host(),
                            history.subject_read().query(),
                            access,
                            read,
                            request,
                        )
                        .map_err(Denial::Query)?,
                    ),
                    None,
                ))
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn open_application_history_with_governed_metadata<'a, History, Capability, Operation, Input>(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        subjects: &Subjects<Schema, History>,
        security: WorthQuerySelectedProductOperation<'a, Schema>,
        truth: &WorthQuerySelectedProductOperation<'a, Schema>,
        metadata: CapabilityAccess<Schema, Capability, Operation, Input>,
        request: &'a WorthQueryRequestScope,
        limits: Limits,
    ) -> Result<Cursor<Schema, History>, Denial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
        Input:
            ApplicationCapabilityRequest<Schema, Capability, Scope = SubjectScope<Schema, History>>,
    {
        self.open_history_with(
            phase,
            history,
            external,
            subjects,
            security,
            truth,
            request,
            limits,
            |access| {
                Ok((
                    None,
                    Some(
                        prepare_governed_access(
                            self.application_query_host(),
                            history.subject_read().query(),
                            access,
                            metadata,
                            request,
                        )
                        .map_err(Denial::Query)?,
                    ),
                ))
            },
        )
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > OpeningOps<Schema> for HostRuntime
{
}

trait OpeningOwnOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<Schema>
{
    #[allow(clippy::too_many_arguments)]
    fn open_history_with<'a, History>(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        subjects: &Subjects<Schema, History>,
        security: WorthQuerySelectedProductOperation<'a, Schema>,
        truth: &WorthQuerySelectedProductOperation<'a, Schema>,
        request: &'a WorthQueryRequestScope,
        limits: Limits,
        authorize: impl for<'fresh> FnOnce(
            &SubjectAccess<'fresh, Schema, History>,
        ) -> Result<PendingDecisions, Denial>,
    ) -> Result<Cursor<Schema, History>, Denial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
    {
        let execution = request_for(self.application_query_host(), phase)?;
        let captured = subjects.clone();
        with_session(
            self.application_query_host(),
            execution,
            history,
            external,
            subjects,
            security,
            truth,
            request,
            authorize,
            |plan, metadata| {
                session::open(
                    execution,
                    self.application_query_host(),
                    plan,
                    metadata,
                    captured,
                    limits.maximum_owned_bytes.get(),
                )
            },
        )
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                Schema,
            > + ?Sized,
    > OpeningOwnOps<Schema> for HostRuntime
{
}
