//! Fresh READ, metadata, NAV and source control precede the SDK's registered lookup.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::{
        execution_validation::validate_execution_lifetimes, HistoryOutputControlPlan,
    },
    product_operation::document_history::{HistoryOutputLookup, HistoryOutputLookupPhase},
    WorthQueryCompletedHistoryOutputs, WorthQueryHistoryOutputNavigationAdvance as Step,
    WorthQueryHistoryOutputNavigationDenial as Denial, WorthQueryHistoryOutputProfile,
    WorthQueryHistoryOutputReadDenial,
};

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    fn validate_navigation_source<H, N, B>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        read: &CompletedMetadataRead<'_, '_>,
        admission: &NavigationAdmission<S, H, N>,
        profile: &WorthQueryHistoryOutputProfile,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        control: &HistoryOutputControlPlan<'_, S, B>,
    ) -> Result<(), WorthQueryHistoryOutputReadDenial>
    where
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryNavigationBinding<S, H>,
        B: ApplicationQueryBinding<S>,
    {
        use WorthQueryHistoryOutputReadDenial as ReadDenial;
        let execution = phase
            .checked_execution_request_for(&self.product_runtime)
            .map_err(|cause| ReadDenial::Authority(NavigationDenial::Stopped(cause)))?;
        profile
            .source_intent()
            .require_control(self, &execution, binding, control)?;
        let request = admission.retain_admitted_request();
        if !request.same_request(control.controls.request_scope())
            || control.security_product.observation() != read.current_product().observation()
            || control.basis.selected_product().observation() != read.current_product().observation()
            || !control.controls.is_current_product_one_shot()
            || crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(control.principal.principal_entity_id()) != read.principal()
        { return Err(ReadDenial::Authority(NavigationDenial::ForeignDocument)); }
        read.confirm(document, ())
            .map_err(|cause| ReadDenial::Authority(NavigationDenial::Document(cause)))?;
        validate_execution_lifetimes(self, &control.controls, control.principal)
            .map_err(super::super::output_read::control_denial)
    }

    pub fn prepare_document_history_navigation_with_outputs<'runtime, H, N, B>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'runtime, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        expected: WorthQueryHistorySelection,
        requested: WorthQueryHistoryNodeId,
        key: u128,
        profile: WorthQueryHistoryOutputProfile,
        source_binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        source: HistoryOutputControlPlan<'_, S, B>,
    ) -> Result<WorthQueryDocumentHistoryNavigation<'runtime, S>, Denial>
    where
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryNavigationBinding<S, H>,
        B: ApplicationQueryBinding<S>,
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        let installed = read.history;
        let (read, admission) = read
            .admit_navigation::<N>(self, phase, access, document, N::input(history_access))
            .map_err(Denial::Navigation)?;
        self.validate_navigation_source::<H, N, B>(
            phase,
            document,
            &read,
            &admission,
            &profile,
            source_binding,
            &source,
        )
        .map_err(Denial::Source)?;
        document
            .prepare_navigation::<N>(
                phase,
                read,
                installed,
                admission,
                expected,
                requested,
                key,
                Some(profile),
            )
            .map_err(Denial::Navigation)
    }

    /// The SDK callback owns slot allocation, tree progress and refused results.
    /// No SDK work runs while a Native selection or attempt guard is held.
    pub fn advance_document_history_navigation_with_outputs<'runtime, 'scope, H, N, B, E>(
        &self,
        phase: &WorthQueryAdvancementPhase<'scope>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'runtime, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        navigation: &WorthQueryDocumentHistoryNavigation<'runtime, S>,
        key: u128,
        profile: &WorthQueryHistoryOutputProfile,
        source_binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        source: HistoryOutputControlPlan<'_, S, B>,
        callback: impl for<'authority> FnOnce(
            [u8; 32],
            Step<'authority, 'scope>,
        )
            -> Result<Option<WorthQueryCompletedHistoryOutputs>, E>,
    ) -> Result<Result<Option<WorthQueryHistoryNavigationCompletion>, E>, Denial>
    where
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryNavigationBinding<S, H>,
        B: ApplicationQueryBinding<S>,
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        let installed = read.history;
        let (read, admission) = read
            .admit_navigation::<N>(self, phase, access, document, N::input(history_access))
            .map_err(Denial::Navigation)?;
        self.validate_navigation_source::<H, N, B>(
            phase,
            document,
            &read,
            &admission,
            profile,
            source_binding,
            &source,
        )
        .map_err(Denial::Source)?;
        let lookup = document
            .inspect_output_navigation::<N>(
                phase, &read, installed, &admission, navigation, key, profile,
            )
            .map_err(Denial::Navigation)?;
        let outputs = match lookup {
            HistoryOutputLookup::Pending { key } => callback(key, Step::Pending),
            HistoryOutputLookup::Selected { key, completed } => {
                callback(key, Step::Selected(completed))
            }
            HistoryOutputLookup::Source {
                key,
                phase: state,
                branch,
                source,
                profile,
            } => {
                let original = std::sync::Arc::clone(&source);
                document
                    .with_output_authority::<N, _>(
                        phase,
                        &read,
                        installed,
                        &admission,
                        &profile,
                        &original,
                        |authority| {
                            callback(
                                key,
                                match state {
                                    HistoryOutputLookupPhase::AwaitingOutputs => Step::Outputs {
                                        branch,
                                        source,
                                        authority,
                                    },
                                    HistoryOutputLookupPhase::Selecting => Step::Selecting {
                                        branch,
                                        source,
                                        authority,
                                    },
                                },
                            )
                        },
                    )
                    .map_err(Denial::Navigation)?
            }
        };
        let outputs = match outputs {
            Ok(outputs) => outputs,
            Err(error) => return Ok(Err(error)),
        };
        // A stopped callback leaves its SDK owners in the document registry.
        // The actual Native transition remains authoritative if its delivery lapses.
        validate_execution_lifetimes(self, &source.controls, source.principal)
            .map_err(|cause| Denial::Source(super::super::output_read::control_denial(cause)))?;
        read.confirm(document, ())
            .map_err(|cause| Denial::Navigation(NavigationDenial::Document(cause)))?;
        document
            .advance_navigation::<N>(
                phase,
                &read,
                installed,
                &admission,
                navigation,
                key,
                Some(profile),
                outputs,
            )
            .map(Ok)
            .map_err(Denial::Navigation)
    }
}
