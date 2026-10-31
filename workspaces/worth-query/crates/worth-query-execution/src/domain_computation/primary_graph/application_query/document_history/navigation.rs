//! Complete current READ/metadata before admitting the independent NAV operation.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryDocumentHistoryNavigation,
    WorthQueryHistoryNavigationCompletion, WorthQueryHistoryNavigationDenial as NavigationDenial,
    WorthQueryHistoryNavigationEligibilityDenial, WorthQueryHistoryNodeId,
    WorthQueryHistorySelection,
};
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRef,
    application_history::ApplicationHistoryNavigationBinding,
    application_schema::ApplicationOperationRef,
};

type NavigationAdmission<S, H, N> = WorthQueryAdmittedApplicationOperation<
    S,
    <N as ApplicationHistoryNavigationBinding<S, H>>::Operation,
    <N as ApplicationHistoryNavigationBinding<S, H>>::Input,
    ApplicationHistoryModel<S, H>,
>;

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistoryRead<'_, S, H>
{
    fn admit_navigation<'phase, 'scope, N: ApplicationHistoryNavigationBinding<S, H>>(
        self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        phase: &'phase WorthQueryAdvancementPhase<'scope>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        input: N::Input,
    ) -> Result<
        (
            CompletedMetadataRead<'phase, 'scope>,
            NavigationAdmission<S, H, N>,
        ),
        NavigationDenial,
    >
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        if !self.history.contains_navigation::<N>() {
            return Err(NavigationDenial::RequiresSemanticCorrection(
                WorthQueryHistoryNavigationEligibilityDenial::NotDeclared,
            ));
        }
        // These references remain owned by the original binding-aware READ
        // callback. No principal or admitted query is retained by the attempt.
        let principal = self.plan.principal;
        let request = self.plan.controls.request_scope().clone();
        let completed = self
            .complete_metadata_read(runtime, phase, document, access)
            .map_err(NavigationDenial::Document)?;
        let selected = document
            .application()
            .on_branch(completed.source().identity().product_branch())
            .select(phase)
            .map_err(NavigationDenial::ProductAdmission)?;
        if selected.product().observation() != completed.source().product().observation() {
            return Err(NavigationDenial::HeadChanged);
        }
        let schema = runtime.installed_schema();
        let reference = ApplicationOperationRef::<S, N::Operation, N::Input>::from_declaration();
        let capability = schema
            .capability(
                ApplicationCapabilityRef::<S, N::Capability>::from_declaration(),
                reference.clone(),
            )
            .map_err(NavigationDenial::CapabilityInstallation)?;
        let capability = selected
            .admit_capability_access(phase, principal, &capability, input, &request)
            .map_err(NavigationDenial::Authorization)?;
        let operation = schema
            .installed_operation(reference)
            .map_err(NavigationDenial::OperationInstallation)?;
        let admission = runtime
            .authorize_capability_operation(phase, capability, &operation, Default::default())
            .map_err(NavigationDenial::Authorization)?;
        Ok((completed, admission))
    }
}

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn prepare_document_history_navigation<'runtime, H, N>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'runtime, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        expected: WorthQueryHistorySelection,
        requested: WorthQueryHistoryNodeId,
        key: u128,
    ) -> Result<WorthQueryDocumentHistoryNavigation<'runtime, S>, NavigationDenial>
    where
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryNavigationBinding<S, H>,
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        let installed = read.history;
        let (read, admission) =
            read.admit_navigation::<N>(self, phase, access, document, N::input(history_access))?;
        document
            .prepare_navigation::<N>(phase, read, installed, admission, expected, requested, key)
    }

    pub fn advance_document_history_navigation<'runtime, H, N>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'runtime, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        navigation: &WorthQueryDocumentHistoryNavigation<'runtime, S>,
        key: u128,
    ) -> Result<Option<WorthQueryHistoryNavigationCompletion>, NavigationDenial>
    where
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryNavigationBinding<S, H>,
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        let installed = read.history;
        let (read, admission) =
            read.admit_navigation::<N>(self, phase, access, document, N::input(history_access))?;
        document.advance_navigation::<N>(phase, read, installed, admission, navigation, key)
    }
}
