//! Opening uses the original disclosed subject set and a separately selected history root.
use super::super::super::admission::prepare_governed_access;
use super::super::{
    session, SubjectScope, WorthQueryApplicationHistoryCursor as Cursor,
    WorthQueryApplicationHistoryPageLimits as Limits,
};
use super::*;
use crate::domain_computation::{
    authorization::WorthQueryAdmittedApplicationCapabilityAccess as CapabilityAccess,
    primary_graph::WorthQueryAdvancementPhase,
};
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Both authored disclosure contracts must admit Public decisions. The captured
    /// source grants provenance only; a fresh session grants this opening's read authority.
    #[allow(clippy::too_many_arguments)]
    pub fn open_application_history<'a, History>(
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

    /// Each actual typed capability is prepared for its own purpose on the same fresh session.
    #[allow(clippy::too_many_arguments)]
    pub fn open_governed_application_history<
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
                            self,
                            history.subject_read().query(),
                            access,
                            read,
                            request,
                        )
                        .map_err(Denial::Query)?,
                    ),
                    Some(
                        prepare_governed_access(
                            self,
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
    pub fn open_application_history_with_governed_subject<
        'a,
        History,
        Capability,
        Operation,
        Input,
    >(
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
                            self,
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
    pub fn open_application_history_with_governed_metadata<
        'a,
        History,
        Capability,
        Operation,
        Input,
    >(
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
                            self,
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
        let execution = request_for(self, phase)?;
        let captured = subjects.clone();
        with_session(
            self,
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
                    self,
                    plan,
                    metadata,
                    captured,
                    limits.maximum_owned_bytes.get(),
                )
            },
        )
    }
}
