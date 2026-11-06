//! Both genuine admissions are consumed before protected attempt or source inspection.
use super::super::WorthQueryDocumentHistory;
use super::*;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRef,
    application_history::{
        ApplicationHistoryBinding, ApplicationHistoryModel, ApplicationHistoryNavigationBinding,
    },
    application_schema::ApplicationOperationRef,
};
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledDocumentHistory};
use {
    crate::domain_computation::primary_graph::application_query::CompletedMetadataRead,
    crate::domain_computation::primary_graph::vocabulary::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding,
    crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation,
    crate::domain_computation::request_execution::WorthQueryAdvancementPhase,
};

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistory<'runtime, S, H>
{
    pub(super) fn validate_navigation<N: ApplicationHistoryNavigationBinding<S, H>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        read: &CompletedMetadataRead<'_, '_>,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
        admission: &WorthQueryAdmittedApplicationOperation<
            S,
            N::Operation,
            N::Input,
            ApplicationHistoryModel<S, H>,
        >,
    ) -> Result<(), WorthQueryHistoryNavigationDenial> {
        use WorthQueryHistoryNavigationDenial as Denial;
        if self.binding != *installed.binding_identity()
            || self.binding != self.application().installed_schema().binding_identity()
        {
            return Err(Denial::Document(
                super::super::WorthQueryDocumentHistoryDenial::InstalledBindingMismatch,
            ));
        }
        if !std::ptr::eq(read.phase(), phase) {
            return Err(Denial::Document(
                super::super::WorthQueryDocumentHistoryDenial::ForeignPhase,
            ));
        }
        if !installed.contains_navigation::<N>() {
            return Err(Denial::RequiresSemanticCorrection(
                WorthQueryHistoryNavigationEligibilityDenial::NotDeclared,
            ));
        }
        read.confirm(self, ()).map_err(Denial::Document)?;
        let product = admission
            .graph_work()
            .mutation_product()
            .ok_or(Denial::Document(
                super::super::WorthQueryDocumentHistoryDenial::ForeignRecording,
            ))?
            .publication_binding();
        if product.observation() != read.current_product().observation() {
            return Err(Denial::HeadChanged);
        }
        if !self.model.matches_operation(admission)
            || read.principal() != admission.operation_scope_binding().principal()
        {
            return Err(Denial::Document(
                super::super::WorthQueryDocumentHistoryDenial::ForeignModel,
            ));
        }
        let capability = self
            .application()
            .installed_schema()
            .capability(
                ApplicationCapabilityRef::<S, N::Capability>::from_declaration(),
                ApplicationOperationRef::<S, N::Operation, N::Input>::from_declaration(),
            )
            .map_err(Denial::CapabilityInstallation)?;
        if admission.installed_capability_identity() != Some(*capability.identity().bytes()) {
            return Err(Denial::Document(
                super::super::WorthQueryDocumentHistoryDenial::InstalledBindingMismatch,
            ));
        }
        admission
            .validate_current_authority()
            .map_err(Denial::Authorization)
    }

    pub(in crate::domain_computation::primary_graph) fn prepare_navigation<
        N: ApplicationHistoryNavigationBinding<S, H>,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        read: CompletedMetadataRead<'_, '_>,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
        admission: WorthQueryAdmittedApplicationOperation<
            S,
            N::Operation,
            N::Input,
            ApplicationHistoryModel<S, H>,
        >,
        expected: WorthQueryHistorySelection,
        intent: WorthQueryHistoryNavigationIntent,
        key: u128,
        output_profile: Option<WorthQueryHistoryOutputProfile>,
    ) -> Result<WorthQueryDocumentHistoryNavigation<'runtime, S>, WorthQueryHistoryNavigationDenial>
    {
        self.validate_navigation::<N>(phase, &read, installed, &admission)?;
        if output_profile
            .as_ref()
            .is_some_and(|profile| profile.model != self.model)
        {
            return Err(WorthQueryHistoryNavigationDenial::Document(
                super::super::WorthQueryDocumentHistoryDenial::ForeignModel,
            ));
        }
        let meaning =
            WorthQueryApplicationIdempotencyBinding::for_host_commit::<S, N::Operation, _, _>(
                &key,
                N::IDENTITY,
            )
            .map_err(WorthQueryHistoryNavigationDenial::Value)?
            .bind_admission(&admission);
        registration::prepare(
            phase,
            self,
            installed,
            expected,
            intent,
            meaning,
            output_profile,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn advance_navigation<
        N: ApplicationHistoryNavigationBinding<S, H>,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        read: &CompletedMetadataRead<'_, '_>,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
        admission: &WorthQueryAdmittedApplicationOperation<
            S,
            N::Operation,
            N::Input,
            ApplicationHistoryModel<S, H>,
        >,
        navigation: &WorthQueryDocumentHistoryNavigation<'runtime, S>,
        key: u128,
        output_profile: Option<&WorthQueryHistoryOutputProfile>,
        outputs: Option<WorthQueryCompletedHistoryOutputs>,
    ) -> Result<Option<WorthQueryHistoryNavigationCompletion>, WorthQueryHistoryNavigationDenial>
    {
        self.validate_navigation::<N>(phase, read, installed, admission)?;
        let meaning =
            WorthQueryApplicationIdempotencyBinding::for_host_commit::<S, N::Operation, _, _>(
                &key,
                N::IDENTITY,
            )
            .map_err(WorthQueryHistoryNavigationDenial::Value)?
            .bind_admission(admission);
        navigation.advance(phase, self, admission, meaning, output_profile, outputs)
    }
}
