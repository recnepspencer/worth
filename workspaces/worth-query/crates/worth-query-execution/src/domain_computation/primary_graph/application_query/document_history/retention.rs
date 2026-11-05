//! Fresh ordinary READ/metadata precedes each independently admitted policy step.
use super::*;
use crate::domain_computation::primary_graph::{
    product_operation::document_history::HistoryRetentionChange,
    WorthQueryAdmittedApplicationOperation, WorthQueryHistoryNodeId,
    WorthQueryHistoryRetentionStatus,
};
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRef,
    application_history::{
        ApplicationHistoryBookmarkBinding, ApplicationHistoryRecentSteps,
        ApplicationHistoryRetentionBinding, ApplicationHistoryWindowBinding,
    },
    application_schema::ApplicationOperationRef,
};

type RetentionAdmission<S, H, N> = WorthQueryAdmittedApplicationOperation<
    S,
    <N as ApplicationHistoryRetentionBinding<S, H>>::Operation,
    <N as ApplicationHistoryRetentionBinding<S, H>>::Input,
    ApplicationHistoryModel<S, H>,
>;

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistoryRead<'_, S, H>
{
    fn admit_retention<'phase, 'scope, N: ApplicationHistoryRetentionBinding<S, H>>(
        self,
        runtime: &dyn crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<S>,
        phase: &'phase WorthQueryAdvancementPhase<'scope>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        input: N::Input,
    ) -> Result<
        (
            CompletedMetadataRead<'phase, 'scope>,
            RetentionAdmission<S, H, N>,
        ),
        Denial,
    >
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        let principal = self.plan.principal;
        let request = self.plan.controls.request_scope().clone();
        let completed = self.complete_metadata_read(runtime, phase, document, access)?;
        let selected = runtime
            .application_runtime()
            .on_branch(completed.source().identity().product_branch())
            .select(phase)
            .map_err(Denial::CandidateSelection)?;
        if selected.product().observation() != completed.source().product().observation() {
            return Err(Denial::HeadDiverged);
        }
        let schema = runtime.installed_schema();
        let reference = ApplicationOperationRef::<S, N::Operation, N::Input>::from_declaration();
        let capability = schema
            .capability(
                ApplicationCapabilityRef::<S, N::Capability>::from_declaration(),
                reference,
            )
            .map_err(Denial::MetadataCapabilityInstallation)?;
        let capability = selected
            .admit_capability_access(phase, principal, &capability, input, &request)
            .map_err(Denial::OperationAdmission)?;
        let operation = schema
            .installed_operation(reference)
            .map_err(Denial::RetentionOperationInstallation)?;
        let admission = runtime
            .application_runtime()
            .authorize_capability_operation(phase, capability, &operation, Default::default())
            .map_err(Denial::OperationAdmission)?;
        Ok((completed, admission))
    }
}

pub trait RetentionOps<S: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<S>
{
    fn change_document_history_window<H, N>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        window: ApplicationHistoryRecentSteps,
    ) -> Result<WorthQueryHistoryRetentionStatus, Denial>
    where
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryWindowBinding<S, H>,
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        if !read.history.contains_window::<N>() {
            return Err(Denial::RetentionPolicyNotDeclared);
        }
        let (read, admission) = read.admit_retention::<N>(
            self.application_query_host(),
            phase,
            access,
            document,
            N::input(history_access),
        )?;
        document.apply_retention::<N>(
            phase,
            &read,
            &admission,
            HistoryRetentionChange::Window(window),
        )
    }

    fn change_document_history_bookmark<H, N>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        node: WorthQueryHistoryNodeId,
        pin: bool,
    ) -> Result<WorthQueryHistoryRetentionStatus, Denial>
    where
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryBookmarkBinding<S, H>,
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        if !read.history.contains_bookmarks::<N>() {
            return Err(Denial::RetentionPolicyNotDeclared);
        }
        let (read, admission) = read.admit_retention::<N>(
            self.application_query_host(),
            phase,
            access,
            document,
            N::input(history_access),
        )?;
        document.apply_retention::<N>(
            phase,
            &read,
            &admission,
            HistoryRetentionChange::Bookmark { node, pin },
        )
    }

    fn read_document_history_retention<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
    ) -> Result<WorthQueryHistoryRetentionStatus, Denial>
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        let completed =
            read.complete_metadata_read(self.application_query_host(), phase, document, access)?;
        let (status, generation) = {
            let selected = document.selection.lock();
            (
                selected.journal.retention_status(),
                selected.journal.view_generation(),
            )
        };
        completed.confirm_generation(document, generation, status)
    }
}

impl<
        S: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::application_query::host::ApplicationQueryHost<
                S,
            > + ?Sized,
    > RetentionOps<S> for HostRuntime
{
}
